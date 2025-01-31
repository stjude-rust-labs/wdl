//! Implementation of task execution backends.

use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::path::PathBuf;
use std::path::absolute;
use std::sync::Arc;

use anyhow::Context;
use anyhow::Result;
use indexmap::IndexMap;
use tokio::sync::oneshot;
use tokio::sync::oneshot::Receiver;

use crate::Value;

pub mod local;

/// Represents constraints applied to a task's execution.
pub struct TaskExecutionConstraints {
    /// The container the task will run in.
    ///
    /// A value of `None` indicates the task will run on the host.
    pub container: Option<String>,
    /// The allocated number of CPUs; must be greater than 0.
    pub cpu: f64,
    /// The allocated memory in bytes; must be greater than 0.
    pub memory: i64,
    /// A list with one specification per allocated GPU.
    ///
    /// The specification is execution engine-specific.
    ///
    /// If no GPUs were allocated, then the value must be an empty list.
    pub gpu: Vec<String>,
    /// A list with one specification per allocated FPGA.
    ///
    /// The specification is execution engine-specific.
    ///
    /// If no FPGAs were allocated, then the value must be an empty list.
    pub fpga: Vec<String>,
    /// A map with one entry for each disk mount point.
    ///
    /// The key is the mount point and the value is the initial amount of disk
    /// space allocated, in bytes.
    ///
    /// The execution engine must, at a minimum, provide one entry for each disk
    /// mount point requested, but may provide more.
    ///
    /// The amount of disk space available for a given mount point may increase
    /// during the lifetime of the task (e.g., autoscaling volumes provided by
    /// some cloud services).
    pub disks: IndexMap<String, i64>,
}

/// Represents the root directory of a task execution.
#[derive(Debug)]
pub struct TaskExecutionRoot {
    /// The path to the working directory for the execution.
    work_dir: PathBuf,
    /// The path to the temp directory for the execution.
    temp_dir: PathBuf,
    /// The path to the command file.
    command: PathBuf,
    /// The path to the stdout file.
    stdout: PathBuf,
    /// The path to the stderr file.
    stderr: PathBuf,
}

impl TaskExecutionRoot {
    /// Creates a task execution root for the given path.
    pub fn new(path: &Path) -> Result<Self> {
        let path = absolute(path).with_context(|| {
            format!(
                "failed to determine absolute path of `{path}`",
                path = path.display()
            )
        })?;

        // Create the temp directory now as it may be needed for task evaluation
        let temp_dir = path.join("tmp");
        fs::create_dir_all(&temp_dir).with_context(|| {
            format!(
                "failed to create directory `{path}`",
                path = temp_dir.display()
            )
        })?;

        Ok(Self {
            work_dir: path.join("work"),
            temp_dir,
            command: path.join("command"),
            stdout: path.join("stdout"),
            stderr: path.join("stderr"),
        })
    }

    /// Gets the working directory for the given attempt number.
    ///
    /// The working directory will be created upon spawning the task.
    pub fn work_dir(&self, attempt: u64) -> PathBuf {
        self.work_dir.join(format!("{attempt}"))
    }

    /// Gets the temporary directory path for the task's execution.
    ///
    /// The temporary directory is created before spawning the task so that it
    /// is available for task evaluation.
    pub fn temp_dir(&self) -> &Path {
        &self.temp_dir
    }

    /// Gets the command file path.
    ///
    /// The command file is created upon spawning the task.
    pub fn command(&self) -> &Path {
        &self.command
    }

    /// Gets the stdout file path.
    ///
    /// The stdout file is created upon spawning the task.
    pub fn stdout(&self) -> &Path {
        &self.stdout
    }

    /// Gets the stderr file path.
    ///
    /// The stderr file is created upon spawning the task.
    pub fn stderr(&self) -> &Path {
        &self.stderr
    }
}

/// Represents a request to spawn a task.
#[derive(Debug)]
pub struct TaskSpawnRequest {
    /// The task's execution root.
    root: TaskExecutionRoot,
    /// The task's evaluated command script.
    command: String,
    /// The task's requirements.
    requirements: HashMap<String, Value>,
    /// The task's hints.
    hints: HashMap<String, Value>,
    /// The task's environment variables.
    env: HashMap<String, String>,
    /// The path mapping from host path to guest path.
    ///
    /// This is only populated for backends that have a container root.
    path_mapping: HashMap<String, String>,
}

impl TaskSpawnRequest {
    /// Creates a new task spawn request with the given root directory.
    pub fn new(root: &Path) -> Result<Self> {
        Ok(Self {
            root: TaskExecutionRoot::new(root)?,
            command: Default::default(),
            requirements: Default::default(),
            hints: Default::default(),
            env: Default::default(),
            path_mapping: Default::default(),
        })
    }

    /// Gets the task execution root for the request.
    pub fn root(&self) -> &TaskExecutionRoot {
        &self.root
    }

    /// Gets the task's evaluated command script.
    pub fn command(&self) -> &str {
        &self.command
    }

    /// Gets a mutable reference to the task's evaluated command script.
    pub fn command_mut(&mut self) -> &mut String {
        &mut self.command
    }

    /// Gets the task's evaluated requirements.
    pub fn requirements(&self) -> &HashMap<String, Value> {
        &self.requirements
    }

    /// Gets a mutable reference to the task's evaluated requirements.
    pub fn requirements_mut(&mut self) -> &mut HashMap<String, Value> {
        &mut self.requirements
    }

    /// Gets the task's evaluated hints.
    pub fn hints(&self) -> &HashMap<String, Value> {
        &self.hints
    }

    /// Gets a mutable reference to the task's evaluated hints.
    pub fn hints_mut(&mut self) -> &mut HashMap<String, Value> {
        &mut self.hints
    }

    /// Gets the task's environment variables.
    pub fn env(&self) -> &HashMap<String, String> {
        &self.env
    }

    /// Gets a mutable reference to the task's environment variables.
    pub fn env_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.env
    }

    /// Gets the task's path mapping from host path to guest path.
    pub fn path_mapping(&self) -> &HashMap<String, String> {
        &self.path_mapping
    }

    /// Gets a mutable reference to the task's path mapping from host path to
    /// guest path.
    pub fn path_mapping_mut(&mut self) -> &mut HashMap<String, String> {
        &mut self.path_mapping
    }
}

/// Represents a task execution backend.
pub trait TaskExecutionBackend: Send + Sync {
    /// Gets the maximum concurrent tasks supported by the backend.
    fn max_concurrency(&self) -> u64;

    /// Gets the execution constraints given a task's requirements and hints.
    ///
    /// Returns an error if the task cannot be constrained for the execution
    /// environment or if the task specifies invalid requirements.
    fn constraints(
        &self,
        requirements: &HashMap<String, Value>,
        hints: &HashMap<String, Value>,
    ) -> Result<TaskExecutionConstraints>;

    /// Gets the container root directory for the backend (e.g. `/mnt/task`)
    ///
    /// Returns `None` if the task execution does not use a container.
    fn container_root(&self) -> Option<&Path>;

    /// Spawns a task with the execution backend.
    ///
    /// The provided channel will be sent a message when the task is spawned.
    ///
    /// Upon success, returns a receiver that will receive the task's exit code.
    fn spawn(
        &self,
        request: Arc<TaskSpawnRequest>,
        attempt: u64,
        spawned: oneshot::Sender<()>,
    ) -> Result<Receiver<Result<i32>>>;
}
