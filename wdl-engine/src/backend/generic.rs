//! Implementation of the GENERIC backend.

use std::borrow::Cow;
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::File;
use std::fs::{self};
use std::path::Path;
use std::process::Stdio;
use std::sync::Arc;
use std::sync::Mutex;

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use anyhow::bail;
use crankshaft::config::backend;
use crankshaft::engine::Task;
use crankshaft::engine::service::name::GeneratorIterator;
use crankshaft::engine::service::name::UniqueAlphanumeric;
use crankshaft::engine::service::runner::Backend;
use crankshaft::engine::service::runner::backend::TaskRunError;
use crankshaft::engine::service::runner::backend::generic;
use crankshaft::engine::task::Execution;
use crankshaft::engine::task::Input;
use crankshaft::engine::task::Output;
use crankshaft::engine::task::Resources;
use crankshaft::engine::task::input::Contents;
use crankshaft::engine::task::input::Type as InputType;
use crankshaft::engine::task::output::Type as OutputType;
use futures::FutureExt;
use futures::future::BoxFuture;
use nonempty::NonEmpty;
use tokio::process::Command;
use tokio::select;
use tokio::sync::oneshot;
use tokio_util::sync::CancellationToken;
use tracing::debug;
use tracing::info;
use tracing::trace;
use wdl_ast::v1::TASK_REQUIREMENT_DISKS;

use super::TaskExecutionBackend;
use super::TaskExecutionConstraints;
use super::TaskExecutionEvents;
use super::TaskExecutionResult;
use super::TaskManager;
use super::TaskManagerRequest;
use super::TaskSpawnRequest;
use crate::COMMAND_FILE_NAME;
use crate::InputKind;
use crate::InputTrie;
use crate::ONE_GIBIBYTE;
use crate::PrimitiveValue;
use crate::STDERR_FILE_NAME;
use crate::STDOUT_FILE_NAME;
use crate::Value;
use crate::WORK_DIR_NAME;
use crate::config::Config;
use crate::config::DEFAULT_TASK_SHELL;
use crate::config::GenericBackendConfig;
use crate::config::TaskResourceLimitBehavior;
use crate::http::HttpDownloader;
use crate::http::rewrite_url;
use crate::path::EvaluationPath;
use crate::v1::DEFAULT_TASK_REQUIREMENT_DISKS;
use crate::v1::container;
use crate::v1::cpu;
use crate::v1::disks;
use crate::v1::max_cpu;
use crate::v1::max_memory;
use crate::v1::memory;
use crate::v1::preemptible;

/// The number of initial expected task names.
///
/// This controls the initial size of the bloom filter and how many names are
/// prepopulated into the name generator.
const INITIAL_EXPECTED_NAMES: usize = 1000;

/// The root guest path for inputs.
const GUEST_INPUTS_DIR: &str = "/mnt/task/inputs";

/// The guest working directory.
const GUEST_WORK_DIR: &str = "/mnt/task/work";

/// The guest path for the command file.
const GUEST_COMMAND_PATH: &str = "/mnt/task/command";

/// The path to the container's stdout.
const GUEST_STDOUT_PATH: &str = "/mnt/task/stdout";

/// The path to the container's stderr.
const GUEST_STDERR_PATH: &str = "/mnt/task/stderr";

/// The default poll interval, in seconds, for the TES backend.
const DEFAULT_GENERIC_INTERVAL: u64 = 60;

/// Represents a generic task request.
///
/// This request contains the requested cpu and memory reservations for the task
/// as well as the result receiver channel.
#[derive(Debug)]
struct GenericTaskRequest {
    /// The engine configuration.
    config: Arc<Config>,
    /// The backend configuration.
    backend_config: Arc<GenericBackendConfig>,
    /// The inner task spawn request.
    inner: TaskSpawnRequest,
    /// The Crankshaft Generic backend to use.
    backend: Arc<generic::Backend>,
    /// The name of the task.
    name: String,
    /// The requested container for the task.
    container: String,
    /// The requested CPU reservation for the task.
    cpu: f64,
    /// The requested memory reservation for the task, in bytes.
    memory: u64,
    /// The requested maximum CPU limit for the task.
    max_cpu: Option<f64>,
    /// The requested maximum memory limit for the task, in bytes.
    max_memory: Option<u64>,
    /// The number of preemptible task retries to do before using a
    /// non-preemptible task.
    ///
    /// If this value is 0, no preemptible tasks are requested from the TES
    /// server.
    preemptible: i64,
    /// The cancellation token for the request.
    token: CancellationToken,
}

impl GenericTaskRequest {
    /// Gets the TES disk resource for the request.
    fn disk_resource(&self) -> Result<f64> {
        let disks = disks(self.inner.requirements(), self.inner.hints())?;
        if disks.len() > 1 {
            bail!(
                "TES backend does not support more than one disk specification for the \
                 `{TASK_REQUIREMENT_DISKS}` task requirement"
            );
        }

        if let Some(mount_point) = disks.keys().next() {
            if *mount_point != "/" {
                bail!(
                    "TES backend does not support a disk mount point other than `/` for the \
                     `{TASK_REQUIREMENT_DISKS}` task requirement"
                );
            }
        }

        Ok(disks
            .values()
            .next()
            .map(|d| d.size as f64)
            .unwrap_or(DEFAULT_TASK_REQUIREMENT_DISKS))
    }
}

impl TaskManagerRequest for GenericTaskRequest {
    fn cpu(&self) -> f64 {
        self.cpu
    }

    fn memory(&self) -> u64 {
        self.memory
    }

    async fn run(self, spawned: oneshot::Sender<()>) -> Result<TaskExecutionResult> {
        // Create the working directory
        let work_dir = self.inner.attempt_dir().join(WORK_DIR_NAME);
        fs::create_dir_all(&work_dir).with_context(|| {
            format!(
                "failed to create directory `{path}`",
                path = work_dir.display()
            )
        })?;

        // Write the evaluated command to disk
        let command_path = self.inner.attempt_dir().join(COMMAND_FILE_NAME);
        fs::write(&command_path, self.inner.command()).with_context(|| {
            format!(
                "failed to write command contents to `{path}`",
                path = command_path.display()
            )
        })?;

        // Create a file for the stdout
        let stdout_path = self.inner.attempt_dir().join(STDOUT_FILE_NAME);
        let stdout = File::create(&stdout_path).with_context(|| {
            format!(
                "failed to create stdout file `{path}`",
                path = stdout_path.display()
            )
        })?;

        // Create a file for the stderr
        let stderr_path = self.inner.attempt_dir().join(STDERR_FILE_NAME);
        let stderr = File::create(&stderr_path).with_context(|| {
            format!(
                "failed to create stderr file `{path}`",
                path = stderr_path.display()
            )
        })?;

        let mut command = Command::new(
            self.config
                .task
                .shell
                .as_deref()
                .unwrap_or(DEFAULT_TASK_SHELL),
        );
        command
            .current_dir(&work_dir)
            .arg("-C")
            .arg(command_path)
            .stdin(Stdio::null())
            .stdout(stdout)
            .stderr(stderr)
            .envs(
                self.inner
                    .env()
                    .iter()
                    .map(|(k, v)| (OsStr::new(k), OsStr::new(v))),
            )
            .kill_on_drop(true);

        // Set the PATH variable for the child on Windows to get consistent PATH
        // searching. See: https://github.com/rust-lang/rust/issues/122660
        #[cfg(windows)]
        if let Ok(path) = std::env::var("PATH") {
            command.env("PATH", path);
        }

        let mut child = command.spawn().context("failed to spawn `bash`")?;

        // Notify that the process has spawned
        spawned.send(()).ok();

        let id = child.id().expect("should have id");
        info!("spawned local `bash` process {id} for task execution");

        select! {
            // Poll the cancellation token before the child future
            biased;

            _ = self.token.cancelled() => {
                bail!("task was cancelled");
            }
            status = child.wait() => {
                let status = status.with_context(|| {
                    format!("failed to wait for termination of task child process {id}")
                })?;

                #[cfg(unix)]
                {
                    use std::os::unix::process::ExitStatusExt;
                    if let Some(signal) = status.signal() {
                        tracing::warn!("task process {id} has terminated with signal {signal}");

                        bail!(
                            "task child process {id} has terminated with signal {signal}; see stderr file \
                            `{path}` for more details",
                            path = stderr_path.display()
                        );
                    }
                }

                let exit_code = status.code().expect("process should have exited");
                info!("task process {id} has terminated with status code {exit_code}");
                Ok(TaskExecutionResult {
                    inputs: self.inner.info.inputs,
                    exit_code,
                    work_dir: EvaluationPath::Local(work_dir),
                    stdout: PrimitiveValue::new_file(stdout_path.into_os_string().into_string().expect("path should be UTF-8")).into(),
                    stderr: PrimitiveValue::new_file(stderr_path.into_os_string().into_string().expect("path should be UTF-8")).into(),
                })
            }
        }
    }
}

/// Represents the Task Execution Service (TES) backend.
pub struct GenericBackend {
    /// The engine configuration.
    config: Arc<Config>,
    /// The backend configuration.
    backend_config: Arc<GenericBackendConfig>,
    /// The underlying Crankshaft backend.
    inner: Arc<generic::Backend>,
    /// The maximum amount of concurrency supported.
    max_concurrency: u64,
    /// The maximum CPUs for any of one node.
    max_cpu: u64,
    /// The maximum memory for any of one node.
    max_memory: u64,
    /// The task manager for the backend.
    manager: TaskManager<GenericTaskRequest>,
    /// The name generator for tasks.
    generator: Arc<Mutex<GeneratorIterator<UniqueAlphanumeric>>>,
}

impl GenericBackend {
    /// Constructs a new TES task execution backend with the given
    /// configuration.
    ///
    /// The provided configuration is expected to have already been validated.
    pub async fn new(config: Arc<Config>, backend_config: &GenericBackendConfig) -> Result<Self> {
        info!("initializing generic backend");

        // There's no way to ask the TES service for its limits, so use the maximums
        // allowed
        let max_cpu = u64::MAX;
        let max_memory = u64::MAX;
        let manager = TaskManager::new_unlimited(max_cpu, max_memory);

        let backend =
            generic::Backend::initialize(backend::generic::Config::default(), None).await?;

        // TODO ACF 2025-07-30: this is an example per-user queue limit on a cluster. in
        // the long run we probably need to bake in a concurrency limit probe to
        // the backend config
        let max_concurrency = 3000;

        Ok(Self {
            config,
            backend_config: Arc::new(backend_config.clone()),
            inner: Arc::new(backend),
            max_concurrency,
            max_cpu,
            max_memory,
            manager,
            generator: Arc::new(Mutex::new(GeneratorIterator::new(
                UniqueAlphanumeric::default_with_expected_generations(INITIAL_EXPECTED_NAMES),
                INITIAL_EXPECTED_NAMES,
            ))),
        })
    }
}

impl TaskExecutionBackend for GenericBackend {
    fn max_concurrency(&self) -> u64 {
        self.max_concurrency
    }

    fn constraints(
        &self,
        requirements: &HashMap<String, Value>,
        hints: &HashMap<String, Value>,
    ) -> Result<TaskExecutionConstraints> {
        let mut cpu = cpu(requirements);
        let mut memory = memory(requirements)?;

        Ok(TaskExecutionConstraints {
            container: None,
            cpu,
            memory,
            gpu: Default::default(),
            fpga: Default::default(),
            disks: Default::default(),
        })
    }

    fn guest_work_dir(&self) -> Option<&Path> {
        // Local execution does not use a container
        None
    }

    fn localize_inputs<'a, 'b, 'c, 'd>(
        &'a self,
        _: &'b HttpDownloader,
        inputs: &'c mut [crate::eval::Input],
    ) -> BoxFuture<'d, Result<()>>
    where
        'a: 'd,
        'b: 'd,
        'c: 'd,
        Self: 'd,
    {
        async move {
            let mut downloads = JoinSet::new();

            for (idx, input) in inputs.iter_mut().enumerate() {
                match input.path() {
                    EvaluationPath::Local(path) => {
                        let location = Location::Path(path.clone().into());
                        let guest_path = location
                            .to_str()
                            .with_context(|| {
                                format!("path `{path}` is not UTF-8", path = path.display())
                            })?
                            .to_string();
                        input.set_location(location.into_owned());
                        input.set_guest_path(guest_path);
                    }
                    EvaluationPath::Remote(url) => {
                        let downloader = downloader.clone();
                        let url = url.clone();
                        downloads.spawn(async move {
                            let location_result = downloader.download(&url).await;

                            match location_result {
                                Ok(location) => Ok((idx, location.into_owned())),
                                Err(e) => bail!("failed to localize `{url}`: {e:?}"),
                            }
                        });
                    }
                }
            }

            while let Some(result) = downloads.join_next().await {
                match result {
                    Ok(Ok((idx, location))) => {
                        let guest_path = location
                            .to_str()
                            .with_context(|| {
                                format!(
                                    "downloaded path `{path}` is not UTF-8",
                                    path = location.display()
                                )
                            })?
                            .to_string();

                        let input = inputs.get_mut(idx).expect("index should be valid");
                        input.set_location(location);
                        input.set_guest_path(guest_path);
                    }
                    Ok(Err(e)) => {
                        // Futures are aborted when the `JoinSet` is dropped.
                        bail!(e);
                    }
                    Err(e) => {
                        // Futures are aborted when the `JoinSet` is dropped.
                        bail!("download task failed: {e}");
                    }
                }
            }

            Ok(())
        }
        .boxed()
    }

    fn spawn(
        &self,
        request: TaskSpawnRequest,
        token: CancellationToken,
    ) -> Result<TaskExecutionEvents> {
        let (spawned_tx, spawned_rx) = oneshot::channel();
        let (completed_tx, completed_rx) = oneshot::channel();

        let requirements = request.requirements();
        let mut cpu = cpu(requirements);
        if let TaskResourceLimitBehavior::TryWithMax = self.config.task.cpu_limit_behavior {
            cpu = std::cmp::min(cpu.ceil() as u64, self.cpu) as f64;
        }
        let mut memory = memory(requirements)? as u64;
        if let TaskResourceLimitBehavior::TryWithMax = self.config.task.memory_limit_behavior {
            memory = std::cmp::min(memory, self.memory);
        }

        self.manager.send(
            GenericTaskRequest {
                backend_config: self.backend_config.clone(),
                backend: todo!(),
                name: request.id,
                container: todo!(),
                max_cpu: todo!(),
                max_memory: todo!(),
                preemptible: todo!(),
                config: self.config.clone(),
                inner: todo!(),
                cpu,
                memory,
                token,
            },
            spawned_tx,
            completed_tx,
        );

        Ok(TaskExecutionEvents {
            spawned: spawned_rx,
            completed: completed_rx,
        })
    }
}
