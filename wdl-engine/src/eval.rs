//! Module for evaluation.

use std::borrow::Cow;
use std::collections::HashMap;
use std::fs;
use std::io::BufRead;
use std::path::Component;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use anyhow::bail;
use indexmap::IndexMap;
use itertools::Itertools;
use path_clean::PathClean;
use rev_buf_reader::RevBufReader;
use url::Url;
use wdl_analysis::Document;
use wdl_analysis::document::Task;
use wdl_analysis::types::Type;
use wdl_ast::Diagnostic;
use wdl_ast::Span;
use wdl_ast::SupportedVersion;
use wdl_ast::v1::TASK_REQUIREMENT_RETURN_CODES;
use wdl_ast::v1::TASK_REQUIREMENT_RETURN_CODES_ALIAS;

use crate::CompoundValue;
use crate::Outputs;
use crate::PrimitiveValue;
use crate::TaskExecutionResult;
use crate::Value;
use crate::http::Downloader;
use crate::http::Location;
use crate::path;
use crate::path::EvaluationPath;
use crate::stdlib::download_file;

pub mod v1;

/// The maximum number of stderr lines to display in error messages.
const MAX_STDERR_LINES: usize = 10;

/// Represents the location of a call in an evaluation error.
#[derive(Debug, Clone)]
pub struct CallLocation {
    /// The document containing the call statement.
    pub document: Document,
    /// The span of the call statement.
    pub span: Span,
}

/// Represents an error that originates from WDL source.
#[derive(Debug)]
pub struct SourceError {
    /// The document originating the diagnostic.
    pub document: Document,
    /// The evaluation diagnostic.
    pub diagnostic: Diagnostic,
    /// The call backtrace for the error.
    ///
    /// An empty backtrace denotes that the error was encountered outside of
    /// a call.
    ///
    /// The call locations are stored as most recent to least recent.
    pub backtrace: Vec<CallLocation>,
}

/// Represents an error that may occur when evaluating a workflow or task.
#[derive(Debug)]
pub enum EvaluationError {
    /// The error came from WDL source evaluation.
    Source(Box<SourceError>),
    /// The error came from another source.
    Other(anyhow::Error),
}

impl EvaluationError {
    /// Creates a new evaluation error from the given document and diagnostic.
    pub fn new(document: Document, diagnostic: Diagnostic) -> Self {
        Self::Source(Box::new(SourceError {
            document,
            diagnostic,
            backtrace: Default::default(),
        }))
    }

    /// Helper for tests for converting an evaluation error to a string.
    #[cfg(feature = "codespan-reporting")]
    #[allow(clippy::inherent_to_string)]
    pub fn to_string(&self) -> String {
        use codespan_reporting::diagnostic::Label;
        use codespan_reporting::diagnostic::LabelStyle;
        use codespan_reporting::files::SimpleFiles;
        use codespan_reporting::term::Config;
        use codespan_reporting::term::termcolor::Buffer;
        use codespan_reporting::term::{self};
        use wdl_ast::AstNode;

        match self {
            Self::Source(e) => {
                let mut files = SimpleFiles::new();
                let mut map = HashMap::new();

                let file_id = files.add(e.document.path(), e.document.root().text().to_string());

                let diagnostic =
                    e.diagnostic
                        .to_codespan(file_id)
                        .with_labels_iter(e.backtrace.iter().map(|l| {
                            let id = l.document.id();
                            let file_id = *map.entry(id).or_insert_with(|| {
                                files.add(l.document.path(), l.document.root().text().to_string())
                            });

                            Label {
                                style: LabelStyle::Secondary,
                                file_id,
                                range: l.span.start()..l.span.end(),
                                message: "called from this location".into(),
                            }
                        }));

                let mut buffer = Buffer::no_color();
                term::emit(&mut buffer, &Config::default(), &files, &diagnostic)
                    .expect("failed to emit diagnostic");

                String::from_utf8(buffer.into_inner()).expect("should be UTF-8")
            }
            Self::Other(e) => format!("{e:?}"),
        }
    }
}

impl From<anyhow::Error> for EvaluationError {
    fn from(e: anyhow::Error) -> Self {
        Self::Other(e)
    }
}

/// Represents a result from evaluating a workflow or task.
pub type EvaluationResult<T> = Result<T, EvaluationError>;

/// Represents context to an expression evaluator.
pub trait EvaluationContext: Send + Sync {
    /// Gets the supported version of the document being evaluated.
    fn version(&self) -> SupportedVersion;

    /// Gets the value of the given name in scope.
    fn resolve_name(&self, name: &str, span: Span) -> Result<Value, Diagnostic>;

    /// Resolves a type name to a type.
    fn resolve_type_name(&self, name: &str, span: Span) -> Result<Type, Diagnostic>;

    /// Gets the working directory for the evaluation.
    ///
    /// Returns `None` if we're not evaluating a task's outputs section.
    fn work_dir(&self) -> Option<&EvaluationPath>;

    /// Gets the temp directory for the evaluation.
    ///
    /// Returns the host path of the temp directory and its guest path, if there
    /// is one.
    fn temp_dir(&self) -> (&Path, Option<&str>);

    /// Gets the value to return for a call to the `stdout` function.
    ///
    /// This is `Some` only when evaluating a task's outputs section.
    fn stdout(&self) -> Option<&Value>;

    /// Gets the value to return for a call to the `stderr` function.
    ///
    /// This is `Some` only when evaluating a task's outputs section.
    fn stderr(&self) -> Option<&Value>;

    /// Gets the task associated with the evaluation context.
    ///
    /// This is only `Some` when evaluating task hints sections.
    fn task(&self) -> Option<&Task>;

    /// Gets the downloader to use for evaluating expressions.
    fn downloader(&self) -> &dyn Downloader;

    /// Translates a guest path to a host path.
    fn host_path<'a>(&'a self, path: &'a str) -> anyhow::Result<Cow<'a, str>>;
}

/// Represents an index of a scope in a collection of scopes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ScopeIndex(usize);

impl ScopeIndex {
    /// Constructs a new scope index from a raw index.
    pub const fn new(index: usize) -> Self {
        Self(index)
    }
}

impl From<usize> for ScopeIndex {
    fn from(index: usize) -> Self {
        Self(index)
    }
}

impl From<ScopeIndex> for usize {
    fn from(index: ScopeIndex) -> Self {
        index.0
    }
}

/// Represents an evaluation scope in a WDL document.
#[derive(Default, Debug)]
pub struct Scope {
    /// The index of the parent scope.
    ///
    /// This is `None` for the root scopes.
    parent: Option<ScopeIndex>,
    /// The map of names in scope to their values.
    names: IndexMap<String, Value>,
}

impl Scope {
    /// Creates a new scope given the parent scope.
    pub fn new(parent: ScopeIndex) -> Self {
        Self {
            parent: Some(parent),
            names: Default::default(),
        }
    }

    /// Inserts a name into the scope.
    pub fn insert(&mut self, name: impl Into<String>, value: impl Into<Value>) {
        let prev = self.names.insert(name.into(), value.into());
        assert!(prev.is_none(), "conflicting name in scope");
    }

    /// Iterates over the local names and values in the scope.
    pub fn local(&self) -> impl Iterator<Item = (&str, &Value)> + use<'_> {
        self.names.iter().map(|(k, v)| (k.as_str(), v))
    }

    /// Gets a mutable reference to an existing name in scope.
    pub(crate) fn get_mut(&mut self, name: &str) -> Option<&mut Value> {
        self.names.get_mut(name)
    }

    /// Clears the scope.
    pub(crate) fn clear(&mut self) {
        self.parent = None;
        self.names.clear();
    }

    /// Sets the scope's parent.
    pub(crate) fn set_parent(&mut self, parent: ScopeIndex) {
        self.parent = Some(parent);
    }
}

impl From<Scope> for IndexMap<String, Value> {
    fn from(scope: Scope) -> Self {
        scope.names
    }
}

/// Represents a reference to a scope.
#[derive(Debug, Clone, Copy)]
pub struct ScopeRef<'a> {
    /// The reference to the scopes collection.
    scopes: &'a [Scope],
    /// The index of the scope in the collection.
    index: ScopeIndex,
}

impl<'a> ScopeRef<'a> {
    /// Creates a new scope reference given the scope index.
    pub fn new(scopes: &'a [Scope], index: impl Into<ScopeIndex>) -> Self {
        Self {
            scopes,
            index: index.into(),
        }
    }

    /// Gets the parent scope.
    ///
    /// Returns `None` if there is no parent scope.
    pub fn parent(&self) -> Option<Self> {
        self.scopes[self.index.0].parent.map(|p| Self {
            scopes: self.scopes,
            index: p,
        })
    }

    /// Gets all of the name and values available at this scope.
    pub fn names(&self) -> impl Iterator<Item = (&str, &Value)> + use<'_> {
        self.scopes[self.index.0]
            .names
            .iter()
            .map(|(n, name)| (n.as_str(), name))
    }

    /// Iterates over each name and value visible to the scope and calls the
    /// provided callback.
    ///
    /// Stops iterating and returns an error if the callback returns an error.
    pub fn for_each(&self, mut cb: impl FnMut(&str, &Value) -> Result<()>) -> Result<()> {
        let mut current = Some(self.index);

        while let Some(index) = current {
            for (n, v) in self.scopes[index.0].local() {
                cb(n, v)?;
            }

            current = self.scopes[index.0].parent;
        }

        Ok(())
    }

    /// Gets the value of a name local to this scope.
    ///
    /// Returns `None` if a name local to this scope was not found.
    pub fn local(&self, name: &str) -> Option<&Value> {
        self.scopes[self.index.0].names.get(name)
    }

    /// Lookups a name in the scope.
    ///
    /// Returns `None` if the name is not available in the scope.
    pub fn lookup(&self, name: &str) -> Option<&Value> {
        let mut current = Some(self.index);

        while let Some(index) = current {
            if let Some(name) = self.scopes[index.0].names.get(name) {
                return Some(name);
            }

            current = self.scopes[index.0].parent;
        }

        None
    }
}

/// Represents an evaluated task.
#[derive(Debug)]
pub struct EvaluatedTask {
    /// The task attempt directory.
    attempt_dir: PathBuf,
    /// The task execution result.
    result: TaskExecutionResult,
    /// The evaluated outputs of the task.
    ///
    /// This is `Ok` when the task executes successfully and all of the task's
    /// outputs evaluated without error.
    ///
    /// Otherwise, this contains the error that occurred while attempting to
    /// evaluate the task's outputs.
    outputs: EvaluationResult<Outputs>,
}

impl EvaluatedTask {
    /// Constructs a new evaluated task.
    ///
    /// Returns an error if the stdout or stderr paths are not UTF-8.
    fn new(attempt_dir: PathBuf, result: TaskExecutionResult) -> anyhow::Result<Self> {
        Ok(Self {
            result,
            attempt_dir,
            outputs: Ok(Default::default()),
        })
    }

    /// Gets the exit code of the evaluated task.
    pub fn exit_code(&self) -> i32 {
        self.result.exit_code
    }

    /// Gets the attempt directory of the task.
    pub fn attempt_dir(&self) -> &Path {
        &self.attempt_dir
    }

    /// Gets the working directory of the evaluated task.
    pub fn work_dir(&self) -> &EvaluationPath {
        &self.result.work_dir
    }

    /// Gets the stdout value of the evaluated task.
    pub fn stdout(&self) -> &Value {
        &self.result.stdout
    }

    /// Gets the stderr value of the evaluated task.
    pub fn stderr(&self) -> &Value {
        &self.result.stderr
    }

    /// Gets the outputs of the evaluated task.
    ///
    /// This is `Ok` when the task executes successfully and all of the task's
    /// outputs evaluated without error.
    ///
    /// Otherwise, this contains the error that occurred while attempting to
    /// evaluate the task's outputs.
    pub fn outputs(&self) -> &EvaluationResult<Outputs> {
        &self.outputs
    }

    /// Converts the evaluated task into an evaluation result.
    ///
    /// Returns `Ok(_)` if the task outputs were evaluated.
    ///
    /// Returns `Err(_)` if the task outputs could not be evaluated.
    pub fn into_result(self) -> EvaluationResult<Outputs> {
        self.outputs
    }

    /// Handles the exit of a task execution.
    ///
    /// Returns an error if the task failed.
    async fn handle_exit(
        &self,
        requirements: &HashMap<String, Value>,
        downloader: &dyn Downloader,
    ) -> anyhow::Result<()> {
        let mut error = true;
        if let Some(return_codes) = requirements
            .get(TASK_REQUIREMENT_RETURN_CODES)
            .or_else(|| requirements.get(TASK_REQUIREMENT_RETURN_CODES_ALIAS))
        {
            match return_codes {
                Value::Primitive(PrimitiveValue::String(s)) if s.as_ref() == "*" => {
                    error = false;
                }
                Value::Primitive(PrimitiveValue::String(s)) => {
                    bail!(
                        "invalid return code value `{s}`: only `*` is accepted when the return \
                         code is specified as a string"
                    );
                }
                Value::Primitive(PrimitiveValue::Integer(ok)) => {
                    if self.result.exit_code == i32::try_from(*ok).unwrap_or_default() {
                        error = false;
                    }
                }
                Value::Compound(CompoundValue::Array(codes)) => {
                    error = !codes.as_slice().iter().any(|v| {
                        v.as_integer()
                            .map(|i| i32::try_from(i).unwrap_or_default() == self.result.exit_code)
                            .unwrap_or(false)
                    });
                }
                _ => unreachable!("unexpected return codes value"),
            }
        } else {
            error = self.result.exit_code != 0;
        }

        if error {
            // Read the last `MAX_STDERR_LINES` number of lines from stderr
            // If there's a problem reading stderr, don't output it
            let stderr = download_file(downloader, None, self.stderr().as_file().unwrap())
                .await
                .ok()
                .and_then(|l| {
                    fs::File::open(l).ok().map(|f| {
                        // Buffer the last N number of lines
                        let reader = RevBufReader::new(f);
                        let lines: Vec<_> = reader
                            .lines()
                            .take(MAX_STDERR_LINES)
                            .map_while(|l| l.ok())
                            .collect();

                        // Iterate the lines in reverse order as we read them in reverse
                        lines
                            .iter()
                            .rev()
                            .format_with("\n", |l, f| f(&format_args!("  {l}")))
                            .to_string()
                    })
                })
                .unwrap_or_default();

            // If the work directory is remote,
            bail!(
                "process terminated with exit code {code}: see `{stdout_path}` and \
                 `{stderr_path}` for task output and the related files in \
                 `{dir}`{header}{stderr}{trailer}",
                code = self.result.exit_code,
                dir = self.attempt_dir().display(),
                stdout_path = self.stdout().as_file().expect("must be file"),
                stderr_path = self.stderr().as_file().expect("must be file"),
                header = if stderr.is_empty() {
                    Cow::Borrowed("")
                } else {
                    format!("\n\ntask stderr output (last {MAX_STDERR_LINES} lines):\n\n").into()
                },
                trailer = if stderr.is_empty() { "" } else { "\n" }
            );
        }

        Ok(())
    }
}

/// Gets the kind of an input.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InputKind {
    /// The input is a single file.
    File,
    /// The input is a directory.
    Directory,
}

impl From<InputKind> for crankshaft::engine::task::input::Type {
    fn from(value: InputKind) -> Self {
        match value {
            InputKind::File => Self::File,
            InputKind::Directory => Self::Directory,
        }
    }
}

/// Represents a `File` or `Directory` input to a task.
#[derive(Debug, Clone)]
pub struct Input {
    /// The input kind.
    kind: InputKind,
    /// The path for the input.
    path: EvaluationPath,
    /// The guest path for the input.
    ///
    /// This is `None` when the backend isn't mapping input paths.
    guest_path: Option<String>,
    /// The download location for the input.
    ///
    /// This is `Some` if the input has been downloaded to a known location.
    location: Option<Location<'static>>,
}

impl Input {
    /// Creates a new input with the given path and access.
    fn new(kind: InputKind, path: EvaluationPath, guest_path: Option<String>) -> Self {
        Self {
            kind,
            path,
            guest_path,
            location: None,
        }
    }

    /// Gets the kind of the input.
    pub fn kind(&self) -> InputKind {
        self.kind
    }

    /// Gets the path to the input.
    pub fn path(&self) -> &EvaluationPath {
        &self.path
    }

    /// Gets the guest path for the input.
    pub fn guest_path(&self) -> Option<&str> {
        self.guest_path.as_deref()
    }

    /// Gets the location of the input if it has been downloaded.
    ///
    /// Returns `None` if the input has not been downloaded or is not remote.
    pub fn location(&self) -> Option<&Path> {
        self.location.as_deref()
    }

    /// Sets the location of the input.
    pub fn set_location(&mut self, location: Location<'static>) {
        self.location = Some(location);
    }
}

/// Represents a node in an input trie.
#[derive(Debug)]
struct InputTrieNode {
    /// The children of this node.
    children: HashMap<String, Self>,
    /// The identifier of the node in the trie.
    ///
    /// A node's identifier is used when formatting guest paths of children.
    id: usize,
    /// The index into the trie's `inputs` collection.
    ///
    /// This is `Some` only for terminal nodes in the trie.
    index: Option<usize>,
}

impl InputTrieNode {
    /// Constructs a new input trie node with the given id.
    fn new(id: usize) -> Self {
        Self {
            children: Default::default(),
            id,
            index: None,
        }
    }
}

/// Represents a prefix trie based on input paths.
///
/// This is used to determine guest paths for inputs.
///
/// From the root to a terminal node represents a unique input.
#[derive(Debug)]
pub struct InputTrie {
    /// The guest inputs root directory.
    ///
    /// This is `None` for backends that don't use containers.
    guest_inputs_dir: Option<&'static str>,
    /// The URL path children of the tree.
    ///
    /// The key in the map is the scheme of each URL.
    urls: HashMap<String, InputTrieNode>,
    /// The local path children of the tree.
    ///
    /// The key in the map is the first component of each path.
    paths: HashMap<String, InputTrieNode>,
    /// The inputs in the trie.
    inputs: Vec<Input>,
    /// The next node identifier.
    next_id: usize,
}

impl InputTrie {
    /// Constructs a new inputs trie with the given guest inputs directory.
    pub fn new(guest_inputs_dir: &'static str) -> Self {
        Self {
            guest_inputs_dir: Some(guest_inputs_dir),
            ..Default::default()
        }
    }

    /// Inserts a new input into the trie.
    ///
    /// Returns `Ok(Some(_))` if an input was added.
    ///
    /// Returns `Ok(None)` if the provided path was already a guest input path
    /// or if the path was relative (relative paths are relative to the working
    /// directory).
    ///
    /// Returns an error for an invalid input path.
    pub fn insert(&mut self, kind: InputKind, path: &str) -> Result<Option<&Input>> {
        let path: EvaluationPath = path.parse()?;
        match path {
            EvaluationPath::Local(path) => {
                // Check to see if the path being inserted is already a guest path
                if let Some(dir) = self.guest_inputs_dir
                    && path.starts_with(dir)
                {
                    return Ok(None);
                }

                // Don't add an input for relative paths; they will be treated as relative to
                // the working directory and therefore not an input
                if path.is_relative() {
                    return Ok(None);
                }

                self.insert_path(kind, path).map(Some)
            }
            EvaluationPath::Remote(url) => Ok(Some(self.insert_url(kind, url))),
        }
    }

    /// Gets the inputs of the trie as a slice.
    pub fn as_slice(&self) -> &[Input] {
        &self.inputs
    }

    /// Gets the inputs of the trie as a mutable slice.
    pub fn as_slice_mut(&mut self) -> &mut [Input] {
        &mut self.inputs
    }

    /// Inserts an input with a local path into the trie.
    fn insert_path(&mut self, kind: InputKind, path: PathBuf) -> Result<&Input> {
        let mut components = path.components();

        let component = components
            .next()
            .context("input path cannot be empty")?
            .as_os_str()
            .to_str()
            .with_context(|| format!("input path `{path}` is not UTF-8", path = path.display()))?;

        let mut parent_id = 0;
        let mut node = self.paths.entry(component.to_string()).or_insert_with(|| {
            let node = InputTrieNode::new(self.next_id);
            self.next_id += 1;
            node
        });

        let mut last_component = None;
        for component in components {
            match component {
                Component::CurDir | Component::ParentDir => {
                    bail!(
                        "input path `{path}` may not contain `.` or `..`",
                        path = path.display()
                    );
                }
                _ => {}
            }

            let component = component.as_os_str().to_str().with_context(|| {
                format!("input path `{path}` is not UTF-8", path = path.display())
            })?;

            parent_id = node.id;

            node = node
                .children
                .entry(component.to_string())
                .or_insert_with(|| {
                    let node = InputTrieNode::new(self.next_id);
                    self.next_id += 1;
                    node
                });

            last_component = Some(component);
        }

        // Check to see if the input already exists in the trie
        if let Some(index) = node.index {
            return Ok(&self.inputs[index]);
        }

        let guest_path = self.guest_inputs_dir.map(|d| {
            format!(
                "{d}/{parent_id}/{last}",
                last = last_component.unwrap_or(".root")
            )
        });

        let index = self.inputs.len();
        self.inputs
            .push(Input::new(kind, EvaluationPath::Local(path), guest_path));
        node.index = Some(index);
        Ok(&self.inputs[index])
    }

    /// Determines the host path of a given guest path.
    ///
    /// If the backend doesn't use containers or if the path is a non-file URL,
    /// the given path is returned.
    ///
    /// Returns an error if the guest path cannot be mapped to a host path.
    pub fn host_path<'a>(
        &'a self,
        path: &'a str,
        guest_work_dir: &str,
        host_work_dir: Option<&EvaluationPath>,
    ) -> Result<Cow<'a, str>> {
        // It's a file scheme'd URL, treat it as an absolute guest path
        // Otherwise, if it is any other URL, return it as-is
        // If it's not a URL, join it with the guest working directory
        let guest = if path::is_file_url(path) {
            path::parse_url(path)
                .and_then(|u| u.to_file_path().ok())
                .ok_or_else(|| anyhow!("guest path `{path}` is not a valid file URI"))?
        } else if path::is_url(path) {
            // Path is a URL, return it as is
            return Ok(path.into());
        } else {
            Path::new(guest_work_dir).join(path)
        }
        .clean();

        // If the path is prefixed with the guest working directory, join it with the
        // host
        if let Ok(stripped) = guest.strip_prefix(guest_work_dir) {
            match host_work_dir {
                Some(host_work_dir) => Ok(host_work_dir
                    .join(
                        stripped
                            .to_str()
                            .with_context(|| format!("guest path `{path}` is not UTF-8"))?,
                    )?
                    .into_string()
                    .with_context(|| format!("guest path `{path}` is not UTF-8"))?
                    .into()),
                _ => bail!("guest path `{path}` can only be used in a task output section"),
            }
        } else {
            // Search for an input with the longest prefix match (i.e. strips to the minimal
            // size)
            self.inputs
                .iter()
                .filter_map(|i| Some((i.path(), guest.strip_prefix(i.guest_path()?).ok()?)))
                .min_by(|(_, a), (_, b)| a.as_os_str().len().cmp(&b.as_os_str().len()))
                .and_then(|(path, stripped)| {
                    if stripped.as_os_str().is_empty() {
                        return Some(Cow::Borrowed(path.to_str()?));
                    }

                    Some(path.join(stripped.to_str()?).ok()?.into_string()?.into())
                })
                .ok_or_else(|| {
                    anyhow!(
                        "guest path `{path}` is not relative to an input or the task working \
                         directory"
                    )
                })
        }
    }

    /// Inserts an input with a URL into the trie.
    fn insert_url(&mut self, kind: InputKind, url: Url) -> &Input {
        // Insert for scheme
        let mut node = self
            .urls
            .entry(url.scheme().to_string())
            .or_insert_with(|| {
                let node = InputTrieNode::new(self.next_id);
                self.next_id += 1;
                node
            });

        // Insert the authority; if the URL's path is empty, we'll
        let mut parent_id = node.id;
        node = node
            .children
            .entry(url.authority().to_string())
            .or_insert_with(|| {
                let node = InputTrieNode::new(self.next_id);
                self.next_id += 1;
                node
            });

        // Insert the path segments
        let mut last_segment = None;
        if let Some(segments) = url.path_segments() {
            for segment in segments {
                parent_id = node.id;
                node = node.children.entry(segment.to_string()).or_insert_with(|| {
                    let node = InputTrieNode::new(self.next_id);
                    self.next_id += 1;
                    node
                });

                if !segment.is_empty() {
                    last_segment = Some(segment);
                }
            }
        }

        // Check to see if the input already exists in the trie
        if let Some(index) = node.index {
            return &self.inputs[index];
        }

        let guest_path = self.guest_inputs_dir.as_ref().map(|d| {
            format!(
                "{d}/{parent_id}/{last}",
                last = last_segment.unwrap_or(".root")
            )
        });

        let index = self.inputs.len();
        self.inputs
            .push(Input::new(kind, EvaluationPath::Remote(url), guest_path));
        node.index = Some(index);
        &self.inputs[index]
    }
}

impl Default for InputTrie {
    fn default() -> Self {
        Self {
            guest_inputs_dir: None,
            urls: Default::default(),
            paths: Default::default(),
            inputs: Default::default(),
            // The first id starts at 1 as 0 is considered the "virtual root" of the trie
            next_id: 1,
        }
    }
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn empty_trie() {
        let empty = InputTrie::default();
        assert!(empty.as_slice().is_empty());
    }

    #[test]
    fn unmapped_inputs() {
        let mut trie = InputTrie::default();
        trie.insert(InputKind::File, "/foo/bar/baz").unwrap();
        assert_eq!(trie.as_slice().len(), 1);
        assert_eq!(trie.as_slice()[0].path().to_str(), Some("/foo/bar/baz"));
        assert!(trie.as_slice()[0].guest_path().is_none());
    }

    #[cfg(unix)]
    #[test]
    fn non_empty_trie_unix() {
        let mut trie = InputTrie::new("/inputs");
        trie.insert(InputKind::Directory, "/").unwrap().unwrap();
        trie.insert(InputKind::File, "/foo/bar/foo.txt")
            .unwrap()
            .unwrap();
        trie.insert(InputKind::File, "/foo/bar/bar.txt")
            .unwrap()
            .unwrap();
        trie.insert(InputKind::File, "/foo/baz/foo.txt")
            .unwrap()
            .unwrap();
        trie.insert(InputKind::File, "/foo/baz/bar.txt")
            .unwrap()
            .unwrap();
        trie.insert(InputKind::File, "/bar/foo/foo.txt")
            .unwrap()
            .unwrap();
        trie.insert(InputKind::File, "/bar/foo/bar.txt")
            .unwrap()
            .unwrap();
        trie.insert(InputKind::Directory, "/baz").unwrap().unwrap();
        trie.insert(InputKind::File, "https://example.com/")
            .unwrap()
            .unwrap();
        trie.insert(InputKind::File, "https://example.com/foo/bar/foo.txt")
            .unwrap()
            .unwrap();
        trie.insert(InputKind::File, "https://example.com/foo/bar/bar.txt")
            .unwrap()
            .unwrap();
        trie.insert(InputKind::File, "https://example.com/foo/baz/foo.txt")
            .unwrap()
            .unwrap();
        trie.insert(InputKind::File, "https://example.com/foo/baz/bar.txt")
            .unwrap()
            .unwrap();
        trie.insert(InputKind::File, "https://example.com/bar/foo/foo.txt")
            .unwrap()
            .unwrap();
        trie.insert(InputKind::File, "https://example.com/bar/foo/bar.txt")
            .unwrap()
            .unwrap();
        trie.insert(InputKind::File, "https://foo.com/bar")
            .unwrap()
            .unwrap();

        // Can't add relative path inputs
        assert!(trie.insert(InputKind::File, "foo.txt").unwrap().is_none());

        // The important part of the guest paths are:
        // 1) The guest file name should be the same (or `.root` if the path is
        //    considered to be root)
        // 2) Paths with the same parent should have the same guest parent
        let paths: Vec<_> = trie
            .as_slice()
            .iter()
            .map(|i| {
                (
                    i.path().to_str().expect("should be a string"),
                    i.guest_path().expect("should have guest path"),
                )
            })
            .collect();

        assert_eq!(
            paths,
            [
                ("/", "/inputs/0/.root"),
                ("/foo/bar/foo.txt", "/inputs/3/foo.txt"),
                ("/foo/bar/bar.txt", "/inputs/3/bar.txt"),
                ("/foo/baz/foo.txt", "/inputs/6/foo.txt"),
                ("/foo/baz/bar.txt", "/inputs/6/bar.txt"),
                ("/bar/foo/foo.txt", "/inputs/10/foo.txt"),
                ("/bar/foo/bar.txt", "/inputs/10/bar.txt"),
                ("/baz", "/inputs/1/baz"),
                ("https://example.com/", "/inputs/15/.root"),
                ("https://example.com/foo/bar/foo.txt", "/inputs/18/foo.txt"),
                ("https://example.com/foo/bar/bar.txt", "/inputs/18/bar.txt"),
                ("https://example.com/foo/baz/foo.txt", "/inputs/21/foo.txt"),
                ("https://example.com/foo/baz/bar.txt", "/inputs/21/bar.txt"),
                ("https://example.com/bar/foo/foo.txt", "/inputs/25/foo.txt"),
                ("https://example.com/bar/foo/bar.txt", "/inputs/25/bar.txt"),
                ("https://foo.com/bar", "/inputs/28/bar"),
            ]
        );
    }

    #[cfg(windows)]
    #[test]
    fn non_empty_trie_windows() {
        let mut trie = InputTrie::new("C:\\inputs");
        trie.insert(InputKind::Directory, "/").unwrap().unwrap();
        trie.insert(InputKind::File, "C:\\foo\\bar\\foo.txt")
            .unwrap()
            .unwrap();
        trie.insert(InputKind::File, "C:\\foo\\bar\\bar.txt")
            .unwrap()
            .unwrap();
        trie.insert(InputKind::File, "C:\\foo\\baz\\foo.txt")
            .unwrap()
            .unwrap();
        trie.insert(InputKind::File, "C:\\foo\\baz\\bar.txt")
            .unwrap()
            .unwrap();
        trie.insert(InputKind::File, "C:\\bar\\foo\\foo.txt")
            .unwrap()
            .unwrap();
        trie.insert(InputKind::File, "C:\\bar\\foo\\bar.txt")
            .unwrap()
            .unwrap();
        trie.insert(InputKind::Directory, "C:\\baz")
            .unwrap()
            .unwrap();
        trie.insert(InputKind::File, "https://example.com/")
            .unwrap()
            .unwrap();
        trie.insert(InputKind::File, "https://example.com/foo/bar/foo.txt")
            .unwrap()
            .unwrap();
        trie.insert(InputKind::File, "https://example.com/foo/bar/bar.txt")
            .unwrap()
            .unwrap();
        trie.insert(InputKind::File, "https://example.com/foo/baz/foo.txt")
            .unwrap()
            .unwrap();
        trie.insert(InputKind::File, "https://example.com/foo/baz/bar.txt")
            .unwrap()
            .unwrap();
        trie.insert(InputKind::File, "https://example.com/bar/foo/foo.txt")
            .unwrap()
            .unwrap();
        trie.insert(InputKind::File, "https://example.com/bar/foo/bar.txt")
            .unwrap()
            .unwrap();
        trie.insert(InputKind::File, "https://foo.com/bar")
            .unwrap()
            .unwrap();

        // Can't add relative path inputs
        assert!(trie.insert(InputKind::File, "foo.txt").unwrap().is_none());

        // The important part of the guest paths are:
        // 1) The guest file name should be the same (or `.root` if the path is
        //    considered to be root)
        // 2) Paths with the same parent should have the same guest parent
        let paths: Vec<_> = trie
            .as_slice()
            .iter()
            .map(|i| {
                (
                    i.path().to_str().expect("should be a string"),
                    i.guest_path().expect("should have guest path"),
                )
            })
            .collect();

        assert_eq!(
            paths,
            [
                ("C:\\", "/inputs/0/.root"),
                ("C:\\foo\\bar\\foo.txt", "/inputs/3/foo.txt"),
                ("C:\\foo\\bar\\bar.txt", "/inputs/3/bar.txt"),
                ("C:\\foo\\baz\\foo.txt", "/inputs/6/foo.txt"),
                ("C:\\foo\\baz\\bar.txt", "/inputs/6/bar.txt"),
                ("C:\\bar\\foo\\foo.txt", "/inputs/10/foo.txt"),
                ("C:\\bar\\foo\\bar.txt", "/inputs/10/bar.txt"),
                ("C:\\baz", "/inputs/1/baz"),
                ("https://example.com/", "/inputs/15/.root"),
                ("https://example.com/foo/bar/foo.txt", "/inputs/18/foo.txt"),
                ("https://example.com/foo/bar/bar.txt", "/inputs/18/bar.txt"),
                ("https://example.com/foo/baz/foo.txt", "/inputs/21/foo.txt"),
                ("https://example.com/foo/baz/bar.txt", "/inputs/21/bar.txt"),
                ("https://example.com/bar/foo/foo.txt", "/inputs/25/foo.txt"),
                ("https://example.com/bar/foo/bar.txt", "/inputs/25/bar.txt"),
                ("https://foo.com/bar", "/inputs/28/bar"),
            ]
        );
    }
}
