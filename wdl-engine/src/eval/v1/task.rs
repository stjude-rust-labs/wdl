//! Implementation of evaluation for V1 tasks.

use std::collections::HashMap;
use std::future::Future;
use std::mem;
use std::path::Path;
use std::sync::Arc;

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use petgraph::algo::toposort;
use rowan::ast::AstPtr;
use tokio::sync::oneshot;
use tracing::debug;
use tracing::info;
use tracing::warn;
use wdl_analysis::diagnostics::multiple_type_mismatch;
use wdl_analysis::diagnostics::unknown_name;
use wdl_analysis::document::Document;
use wdl_analysis::document::TASK_VAR_NAME;
use wdl_analysis::document::Task;
use wdl_analysis::eval::v1::TaskGraphBuilder;
use wdl_analysis::eval::v1::TaskGraphNode;
use wdl_analysis::types::Optional;
use wdl_analysis::types::Type;
use wdl_analysis::types::v1::task_hint_types;
use wdl_analysis::types::v1::task_requirement_types;
use wdl_ast::Ast;
use wdl_ast::AstNodeExt;
use wdl_ast::AstToken;
use wdl_ast::Diagnostic;
use wdl_ast::Ident;
use wdl_ast::Severity;
use wdl_ast::SupportedVersion;
use wdl_ast::TokenStrHash;
use wdl_ast::v1::CommandPart;
use wdl_ast::v1::CommandSection;
use wdl_ast::v1::Decl;
use wdl_ast::v1::RequirementsSection;
use wdl_ast::v1::RuntimeSection;
use wdl_ast::v1::StrippedCommandPart;
use wdl_ast::v1::TaskHintsSection;
use wdl_ast::version::V1;

use super::DeclPtr;
use super::ProgressKind;
use crate::Coercible;
use crate::EvaluationContext;
use crate::EvaluationResult;
use crate::Outputs;
use crate::Scope;
use crate::ScopeIndex;
use crate::ScopeRef;
use crate::TaskExecutionBackend;
use crate::TaskExecutionRoot;
use crate::TaskInputs;
use crate::TaskSpawnRequest;
use crate::TaskValue;
use crate::Value;
use crate::config::Config;
use crate::diagnostics::output_evaluation_failed;
use crate::diagnostics::runtime_type_mismatch;
use crate::eval::EvaluatedTask;
use crate::v1::ExprEvaluator;

/// The default value for the `cpu` requirement.
pub const DEFAULT_TASK_REQUIREMENT_CPU: f64 = 1.0;
/// The default value for the `memory` requirement.
pub const DEFAULT_TASK_REQUIREMENT_MEMORY: i64 = 2 * 1024 * 1024 * 1024;

/// The index of a task's root scope.
const ROOT_SCOPE_INDEX: ScopeIndex = ScopeIndex::new(0);
/// The index of a task's output scope.
const OUTPUT_SCOPE_INDEX: ScopeIndex = ScopeIndex::new(1);
/// The index of the evaluation scope where the WDL 1.2 `task` variable is
/// visible.
const TASK_SCOPE_INDEX: ScopeIndex = ScopeIndex::new(2);

/// Represents a "pointer" to a task evaluation graph node.
///
/// Unlike `TaskGraphNode`, this type is `Send`+`Sync`.
///
/// This type is cheaply cloned.
#[derive(Debug, Clone)]
enum TaskGraphNodePtr {
    /// The node is an input.
    Input(DeclPtr),
    /// The node is a private decl.
    Decl(DeclPtr),
    /// The node is an output decl.
    Output(DeclPtr),
    /// The node is a command section.
    Command(AstPtr<CommandSection>),
    /// The node is a `runtime` section.
    Runtime(AstPtr<RuntimeSection>),
    /// The node is a `requirements`` section.
    Requirements(AstPtr<RequirementsSection>),
    /// The node is a `hints`` section.
    Hints(AstPtr<TaskHintsSection>),
}

impl TaskGraphNodePtr {
    /// Constructs a new indirect task graph node from a task graph
    /// node.
    fn new(node: &TaskGraphNode) -> Self {
        match node {
            TaskGraphNode::Input(decl) => Self::Input(DeclPtr::new(decl)),
            TaskGraphNode::Decl(decl) => Self::Decl(DeclPtr::new(decl)),
            TaskGraphNode::Output(decl) => Self::Output(DeclPtr::new(decl)),
            TaskGraphNode::Command(section) => Self::Command(AstPtr::new(section)),
            TaskGraphNode::Runtime(section) => Self::Runtime(AstPtr::new(section)),
            TaskGraphNode::Requirements(section) => Self::Requirements(AstPtr::new(section)),
            TaskGraphNode::Hints(section) => Self::Hints(AstPtr::new(section)),
        }
    }

    /// Converts the pointer back to the task graph node.
    fn to_node(&self, document: &Document) -> TaskGraphNode {
        match self {
            Self::Input(decl) => TaskGraphNode::Input(decl.to_node(document)),
            Self::Decl(decl) => TaskGraphNode::Decl(decl.to_node(document)),
            Self::Output(decl) => TaskGraphNode::Output(decl.to_node(document)),
            Self::Command(section) => {
                TaskGraphNode::Command(section.to_node(document.node().syntax()))
            }
            Self::Runtime(section) => {
                TaskGraphNode::Runtime(section.to_node(document.node().syntax()))
            }
            Self::Requirements(section) => {
                TaskGraphNode::Requirements(section.to_node(document.node().syntax()))
            }
            Self::Hints(section) => TaskGraphNode::Hints(section.to_node(document.node().syntax())),
        }
    }
}

/// Used to evaluate expressions in tasks.
struct TaskEvaluationContext<'a, 'b> {
    /// The task execution root.
    root: &'a TaskExecutionRoot,
    /// The associated evaluation state.
    state: &'a State<'b>,
    /// The current evaluation scope.
    scope: ScopeIndex,
    /// The standard out value to use.
    stdout: Option<&'a Value>,
    /// The standard error value to use.
    stderr: Option<&'a Value>,
    /// Whether or not the evaluation has associated task information.
    ///
    /// This is `true` when evaluating hints sections.
    task: bool,
}

impl<'a, 'b> TaskEvaluationContext<'a, 'b> {
    /// Constructs a new expression evaluation context.
    pub fn new(root: &'a TaskExecutionRoot, state: &'a State<'b>, scope: ScopeIndex) -> Self {
        Self {
            root,
            state,
            scope,
            stdout: None,
            stderr: None,
            task: false,
        }
    }

    /// Sets the stdout value to use for the evaluation context.
    pub fn with_stdout(mut self, stdout: &'a Value) -> Self {
        self.stdout = Some(stdout);
        self
    }

    /// Sets the stderr value to use for the evaluation context.
    pub fn with_stderr(mut self, stderr: &'a Value) -> Self {
        self.stderr = Some(stderr);
        self
    }

    /// Marks the evaluation as having associated task information.
    ///
    /// This is used in evaluating hints sections.
    pub fn with_task(mut self) -> Self {
        self.task = true;
        self
    }
}

impl EvaluationContext for TaskEvaluationContext<'_, '_> {
    fn version(&self) -> SupportedVersion {
        self.state
            .document
            .version()
            .expect("document should have a version")
    }

    fn resolve_name(&self, name: &Ident) -> Result<Value, Diagnostic> {
        ScopeRef::new(&self.state.scopes, self.scope)
            .lookup(name.as_str())
            .cloned()
            .ok_or_else(|| unknown_name(name.as_str(), name.span()))
    }

    fn resolve_type_name(&mut self, name: &Ident) -> Result<Type, Diagnostic> {
        crate::resolve_type_name(self.state.document, name)
    }

    fn work_dir(&self) -> &Path {
        self.root.work_dir()
    }

    fn temp_dir(&self) -> &Path {
        self.root.temp_dir()
    }

    fn stdout(&self) -> Option<&Value> {
        self.stdout
    }

    fn stderr(&self) -> Option<&Value> {
        self.stderr
    }

    fn task(&self) -> Option<&Task> {
        if self.task {
            Some(self.state.task)
        } else {
            None
        }
    }
}

/// Represents task evaluation state.
struct State<'a> {
    /// The task spawn request being built by task evaluation.
    ///
    /// Initially `State` is the only reference holder of the request and is
    /// therefore mutable; once the task is spawned, there will be more than
    /// one reference and the request can no longer be made mutable.
    request: Arc<TaskSpawnRequest>,
    /// The document containing the workflow being evaluated.
    document: &'a Document,
    /// The task being evaluated.
    task: &'a Task,
    /// The scopes of the task being evaluated.
    ///
    /// The first scope is the root scope, the second is the output scope, and
    /// the third is the scope where the "task" variable is visible in 1.2+
    /// evaluations.
    scopes: [Scope; 3],
}

impl<'a> State<'a> {
    /// Constructs a new task evaluation state.
    fn new(root: &Path, document: &'a Document, task: &'a Task) -> Result<Self> {
        // Tasks have a root scope (index 0), an output scope (index 1), and a `task`
        // variable scope (index 2). The output scope inherits from the root scope and
        // the task scope inherits from the output scope. Inputs and private
        // declarations are evaluated into the root scope. Outputs are evaluated into
        // the output scope. The task scope is used for evaluating expressions in both
        // the command and output sections. Only the `task` variable in WDL 1.2 is
        // introduced into the task scope; in previous WDL versions, the task scope will
        // not have any local names.
        let scopes = [
            Scope::default(),
            Scope::new(ROOT_SCOPE_INDEX),
            Scope::new(OUTPUT_SCOPE_INDEX),
        ];

        Ok(Self {
            request: Arc::new(TaskSpawnRequest::new(root)?),
            document,
            task,
            scopes,
        })
    }
}

/// Represents a WDL V1 task evaluator.
pub struct TaskEvaluator {
    /// The associated evaluation configuration.
    config: Arc<Config>,
    /// The associated task execution backend.
    backend: Arc<dyn TaskExecutionBackend>,
}

impl TaskEvaluator {
    /// Constructs a new task evaluator with the given evaluation
    /// configuration.
    ///
    /// This method creates a default task execution backend.
    ///
    /// Returns an error if the configuration isn't valid.
    pub fn new(config: Config) -> Result<Self> {
        let backend = config.create_backend()?;
        Self::new_with_backend(config, backend)
    }

    /// Constructs a new task evaluator with the given evaluation
    /// configuration and task execution backend.
    ///
    /// Returns an error if the configuration isn't valid.
    pub fn new_with_backend(
        config: Config,
        backend: Arc<dyn TaskExecutionBackend>,
    ) -> Result<Self> {
        config.validate()?;

        Ok(Self {
            config: Arc::new(config),
            backend,
        })
    }

    /// Creates a new task evaluator with the given configuration and backend.
    ///
    /// This method does not validate the configuration.
    pub(crate) fn new_unchecked(
        config: Arc<Config>,
        backend: Arc<dyn TaskExecutionBackend>,
    ) -> Self {
        Self { config, backend }
    }

    /// Evaluates the given task.
    ///
    /// Upon success, returns the evaluated task.
    pub async fn evaluate<P, R>(
        &mut self,
        document: &Document,
        task: &Task,
        inputs: &TaskInputs,
        root: impl AsRef<Path>,
        progress: P,
    ) -> EvaluationResult<EvaluatedTask>
    where
        P: Fn(ProgressKind<'_>) -> R + Send + Sync + 'static,
        R: Future<Output = ()> + Send,
    {
        self.evaluate_with_progress(
            document,
            task,
            inputs,
            root.as_ref(),
            task.name(),
            Arc::new(progress),
        )
        .await
    }

    /// Evaluates the given task with the given shared progress callback.
    pub(crate) async fn evaluate_with_progress<P, R>(
        &mut self,
        document: &Document,
        task: &Task,
        inputs: &TaskInputs,
        root: &Path,
        id: &str,
        progress: Arc<P>,
    ) -> EvaluationResult<EvaluatedTask>
    where
        P: Fn(ProgressKind<'_>) -> R + Send + Sync + 'static,
        R: Future<Output = ()> + Send,
    {
        progress(ProgressKind::TaskStarted { id }).await;

        let result = self
            .perform_evaluation(document, task, inputs, root, id, progress.clone())
            .await;

        progress(ProgressKind::TaskCompleted {
            id,
            result: &result,
        })
        .await;

        result
    }

    /// Performs the actual evaluation of the task.
    async fn perform_evaluation<P, R>(
        &mut self,
        document: &Document,
        task: &Task,
        inputs: &TaskInputs,
        root: &Path,
        id: &str,
        progress: Arc<P>,
    ) -> EvaluationResult<EvaluatedTask>
    where
        P: Fn(ProgressKind<'_>) -> R + Send + Sync + 'static,
        R: Future<Output = ()> + Send,
    {
        // Return the first error analysis diagnostic if there was one
        // With this check, we can assume certain correctness properties of the document
        if let Some(diagnostic) = document
            .diagnostics()
            .iter()
            .find(|d| d.severity() == Severity::Error)
        {
            return Err(diagnostic.clone().into());
        }

        inputs.validate(document, task, None).with_context(|| {
            format!(
                "failed to validate the inputs to task `{task}`",
                task = task.name()
            )
        })?;

        // This scope exists to ensure all AST nodes are dropped before any awaits as
        // they are not `Send`. The block should only produce types that are
        // `Send`
        let (mut state, version, graph, nodes, current, definition) = {
            let ast = match document.node().ast() {
                Ast::V1(ast) => ast,
                _ => {
                    return Err(
                        anyhow!("task evaluation is only supported for WDL 1.x documents").into(),
                    );
                }
            };

            // Find the task in the AST
            let definition = ast
                .tasks()
                .find(|t| t.name().as_str() == task.name())
                .expect("task should exist in the AST");

            let version = document.version().expect("document should have version");

            // Build an evaluation graph for the task
            let mut diagnostics = Vec::new();
            let graph = TaskGraphBuilder::default().build(version, &definition, &mut diagnostics);
            let graph = graph.map(|_, n| TaskGraphNodePtr::new(n), |_, e| *e);
            if let Some(diagnostic) = diagnostics.pop() {
                return Err(diagnostic.into());
            }

            info!(
                "evaluating task `{task}` in `{uri}`",
                task = task.name(),
                uri = document.uri()
            );

            let mut state = State::new(root, document, task)?;
            let nodes = toposort(&graph, None).expect("graph should be acyclic");
            let mut current = 0;
            while current < nodes.len() {
                match graph[nodes[current]].to_node(document) {
                    TaskGraphNode::Input(decl) => {
                        self.evaluate_input(&mut state, &decl, inputs)?;
                    }
                    TaskGraphNode::Decl(decl) => {
                        self.evaluate_decl(&mut state, &decl)?;
                    }
                    TaskGraphNode::Output(_) => {
                        // Stop at the first output; at this point the task can be executed
                        break;
                    }
                    TaskGraphNode::Command(section) => {
                        // Get the execution constraints
                        let constraints = self
                            .backend
                            .constraints(state.request.requirements(), state.request.hints())
                            .with_context(|| {
                                format!("failed to execute task `{task}`", task = task.name())
                            })?;

                        // Introduce the task variable at this point; valid for both the command
                        // section and the outputs section
                        if version >= SupportedVersion::V1(V1::Two) {
                            let task = TaskValue::new_v1(task.name(), id, &definition, constraints);
                            state.scopes[TASK_SCOPE_INDEX.0]
                                .insert(TASK_VAR_NAME, Value::Task(task));
                        }

                        self.evaluate_command(&mut state, &section)?;
                    }
                    TaskGraphNode::Runtime(section) => {
                        assert!(
                            state.request.requirements().is_empty(),
                            "requirements should not have been evaluated"
                        );
                        assert!(
                            state.request.hints().is_empty(),
                            "hints should not have been evaluated"
                        );

                        self.evaluate_runtime_section(&mut state, &section, inputs)?;
                    }
                    TaskGraphNode::Requirements(section) => {
                        assert!(
                            state.request.requirements().is_empty(),
                            "requirements should not have been evaluated"
                        );
                        self.evaluate_requirements_section(&mut state, &section, inputs)?;
                    }
                    TaskGraphNode::Hints(section) => {
                        assert!(
                            state.request.hints().is_empty(),
                            "hints should not have been evaluated"
                        );
                        self.evaluate_hints_section(&mut state, &section, inputs)?;
                    }
                }

                current += 1;
            }

            (
                state,
                version,
                graph,
                nodes,
                current,
                AstPtr::new(&definition),
            )
        };

        // TODO: check call cache for a hit. if so, skip task execution and use cache
        // paths for output evaluation

        // Spawn the task in a retry loop
        let mut retry = 0;
        let mut evaluated = loop {
            let (tx, rx) = oneshot::channel();

            let task = self.backend.spawn(state.request.clone(), tx)?;

            // Await the spawned notification first
            rx.await.expect("failed to await spawned notification");

            progress(ProgressKind::TaskExecutionStarted { id });

            let result = task
                .await
                .expect("failed to receive response from spawned task");

            progress(ProgressKind::TaskExecutionCompleted {
                id,
                result: &result,
            });

            let status_code = result?;
            let evaluated = EvaluatedTask::new(state.request.root(), status_code)?;

            // Update the task variable
            if version >= SupportedVersion::V1(V1::Two) {
                let task = state.scopes[TASK_SCOPE_INDEX.0]
                    .get_mut(TASK_VAR_NAME)
                    .unwrap()
                    .as_task_mut()
                    .unwrap();

                task.set_attempt(retry as i64 + 1);
                task.set_return_code(evaluated.status_code);
            }

            if let Err(e) = evaluated.handle_exit(state.request.requirements()) {
                if retry >= self.config.task.retries.unwrap_or(0) {
                    return Err(e.into());
                }

                retry += 1;
                continue;
            }

            break evaluated;
        };

        // Evaluate the remaining inputs (unused), and decls, and outputs
        for index in &nodes[current..] {
            match graph[*index].to_node(document) {
                TaskGraphNode::Input(decl) => {
                    self.evaluate_input(&mut state, &decl, inputs)?;
                }
                TaskGraphNode::Decl(decl) => {
                    self.evaluate_decl(&mut state, &decl)?;
                }
                TaskGraphNode::Output(decl) => {
                    self.evaluate_output(&mut state, &decl, &evaluated)?;
                }
                _ => {
                    unreachable!("only declarations should be evaluated after the command")
                }
            }
        }

        // Take the output scope and return it
        let mut outputs: Outputs = mem::take(&mut state.scopes[OUTPUT_SCOPE_INDEX.0]).into();
        if let Some(section) = definition.to_node(document.node().syntax()).output() {
            let indexes: HashMap<_, _> = section
                .declarations()
                .enumerate()
                .map(|(i, d)| (TokenStrHash::new(d.name()), i))
                .collect();
            outputs.sort_by(move |a, b| indexes[a].cmp(&indexes[b]))
        }

        evaluated.outputs = Ok(outputs);
        Ok(evaluated)
    }

    /// Evaluates a task input.
    fn evaluate_input(
        &mut self,
        state: &mut State<'_>,
        decl: &Decl,
        inputs: &TaskInputs,
    ) -> EvaluationResult<()> {
        let name = decl.name();
        let decl_ty = decl.ty();
        let ty = crate::convert_ast_type_v1(state.document, &decl_ty)?;

        let (value, span) = match inputs.get(name.as_str()) {
            Some(input) => (input.clone(), name.span()),
            None => {
                if let Some(expr) = decl.expr() {
                    debug!(
                        "evaluating input `{name}` for task `{task}` in `{uri}`",
                        name = name.as_str(),
                        task = state.task.name(),
                        uri = state.document.uri(),
                    );

                    let mut evaluator = ExprEvaluator::new(TaskEvaluationContext::new(
                        state.request.root(),
                        state,
                        ROOT_SCOPE_INDEX,
                    ));
                    let value = evaluator.evaluate_expr(&expr)?;
                    (value, expr.span())
                } else {
                    assert!(decl.ty().is_optional(), "type should be optional");
                    (Value::None, name.span())
                }
            }
        };

        let value = value
            .coerce(&ty)
            .map_err(|e| runtime_type_mismatch(e, &ty, name.span(), &value.ty(), span))?;
        state.scopes[ROOT_SCOPE_INDEX.0].insert(name.as_str(), value.clone());

        // If the request is still mutable, update it for the declaration
        if let Some(request) = Arc::get_mut(&mut state.request) {
            self.update_request_for_decl(request, &name, &value, decl.env().is_some())?;
        }

        Ok(())
    }

    /// Evaluates a task private declaration.
    fn evaluate_decl(&mut self, state: &mut State<'_>, decl: &Decl) -> EvaluationResult<()> {
        let name = decl.name();
        debug!(
            "evaluating private declaration `{name}` for task `{task}` in `{uri}`",
            name = name.as_str(),
            task = state.task.name(),
            uri = state.document.uri(),
        );

        let decl_ty = decl.ty();
        let ty = crate::convert_ast_type_v1(state.document, &decl_ty)?;

        let mut evaluator = ExprEvaluator::new(TaskEvaluationContext::new(
            state.request.root(),
            state,
            ROOT_SCOPE_INDEX,
        ));

        let expr = decl.expr().expect("private decls should have expressions");
        let value = evaluator.evaluate_expr(&expr)?;
        let value = value
            .coerce(&ty)
            .map_err(|e| runtime_type_mismatch(e, &ty, name.span(), &value.ty(), expr.span()))?;
        state.scopes[ROOT_SCOPE_INDEX.0].insert(name.as_str(), value.clone());

        // If the request is still mutable, update it for the declaration
        if let Some(request) = Arc::get_mut(&mut state.request) {
            self.update_request_for_decl(request, &name, &value, decl.env().is_some())?;
        }

        Ok(())
    }

    /// Helper for updating the task spawn request for a declaration.
    fn update_request_for_decl(
        &mut self,
        request: &mut TaskSpawnRequest,
        name: &Ident,
        value: &Value,
        env: bool,
    ) -> Result<(), crate::EvaluationError> {
        // Update the environment variables if the decl is marked `env`
        if env {
            request.env_mut().insert(
                name.as_str().to_string(),
                value
                    .as_primitive()
                    .expect("value should be primitive")
                    .raw()
                    .to_string(),
            );
        }

        // If the backend uses containers, add a mapping entry for any paths the value
        // might contain Initially, it will be an empty value, but will be
        // populated in `map_request_paths`.
        if self.backend.container_root().is_some() {
            value.visit_paths(&mut |path| {
                request
                    .path_mapping_mut()
                    .insert(path.to_string(), String::new());
                Ok(())
            })?;
        }

        Ok(())
    }

    /// Evaluates the runtime section.
    fn evaluate_runtime_section(
        &mut self,
        state: &mut State<'_>,
        section: &RuntimeSection,
        inputs: &TaskInputs,
    ) -> EvaluationResult<()> {
        debug!(
            "evaluating runtimes section for task `{task}` in `{uri}`",
            task = state.task.name(),
            uri = state.document.uri()
        );

        let version = state
            .document
            .version()
            .expect("document should have version");
        for item in section.items() {
            let name = item.name();
            if let Some(value) = inputs.requirement(name.as_str()) {
                Arc::get_mut(&mut state.request)
                    .expect("request should still be mutable")
                    .requirements_mut()
                    .insert(name.as_str().to_string(), value.clone());
                continue;
            } else if let Some(value) = inputs.hint(name.as_str()) {
                Arc::get_mut(&mut state.request)
                    .expect("request should still be mutable")
                    .hints_mut()
                    .insert(name.as_str().to_string(), value.clone());
                continue;
            }

            let mut evaluator = ExprEvaluator::new(TaskEvaluationContext::new(
                state.request.root(),
                state,
                ROOT_SCOPE_INDEX,
            ));

            let (types, requirement) = match task_requirement_types(version, name.as_str()) {
                Some(types) => (Some(types), true),
                None => match task_hint_types(version, name.as_str(), false) {
                    Some(types) => (Some(types), false),
                    None => (None, false),
                },
            };

            // Evaluate and coerce to the expected type
            let expr = item.expr();
            let mut value = evaluator.evaluate_expr(&expr)?;
            if let Some(types) = types {
                value = types
                    .iter()
                    .find_map(|ty| value.coerce(ty).ok())
                    .ok_or_else(|| {
                        multiple_type_mismatch(types, name.span(), &value.ty(), expr.span())
                    })?;
            }

            if requirement {
                Arc::get_mut(&mut state.request)
                    .expect("request should still be mutable")
                    .requirements_mut()
                    .insert(name.as_str().to_string(), value);
            } else {
                Arc::get_mut(&mut state.request)
                    .expect("request should still be mutable")
                    .hints_mut()
                    .insert(name.as_str().to_string(), value);
            }
        }

        Ok(())
    }

    /// Evaluates the requirements section.
    fn evaluate_requirements_section(
        &mut self,
        state: &mut State<'_>,
        section: &RequirementsSection,
        inputs: &TaskInputs,
    ) -> EvaluationResult<()> {
        debug!(
            "evaluating requirements section for task `{task}` in `{uri}`",
            task = state.task.name(),
            uri = state.document.uri()
        );

        let version = state
            .document
            .version()
            .expect("document should have version");
        for item in section.items() {
            let name = item.name();
            if let Some(value) = inputs.requirement(name.as_str()) {
                Arc::get_mut(&mut state.request)
                    .expect("request should still be mutable")
                    .requirements_mut()
                    .insert(name.as_str().to_string(), value.clone());
                continue;
            }

            let mut evaluator = ExprEvaluator::new(TaskEvaluationContext::new(
                state.request.root(),
                state,
                ROOT_SCOPE_INDEX,
            ));

            let types = task_requirement_types(version, name.as_str())
                .expect("requirement should be known");

            // Evaluate and coerce to the expected type
            let expr = item.expr();
            let value = evaluator.evaluate_expr(&expr)?;
            let value = types
                .iter()
                .find_map(|ty| value.coerce(ty).ok())
                .ok_or_else(|| {
                    multiple_type_mismatch(types, name.span(), &value.ty(), expr.span())
                })?;

            Arc::get_mut(&mut state.request)
                .expect("request should still be mutable")
                .requirements_mut()
                .insert(name.as_str().to_string(), value);
        }

        Ok(())
    }

    /// Evaluates the hints section.
    fn evaluate_hints_section(
        &mut self,
        state: &mut State<'_>,
        section: &TaskHintsSection,
        inputs: &TaskInputs,
    ) -> EvaluationResult<()> {
        debug!(
            "evaluating hints section for task `{task}` in `{uri}`",
            task = state.task.name(),
            uri = state.document.uri()
        );

        for item in section.items() {
            let name = item.name();
            if let Some(value) = inputs.hint(name.as_str()) {
                Arc::get_mut(&mut state.request)
                    .expect("request should still be mutable")
                    .hints_mut()
                    .insert(name.as_str().to_string(), value.clone());
                continue;
            }

            let mut evaluator = ExprEvaluator::new(
                TaskEvaluationContext::new(state.request.root(), state, ROOT_SCOPE_INDEX)
                    .with_task(),
            );

            let value = evaluator.evaluate_hints_item(&name, &item.expr())?;
            Arc::get_mut(&mut state.request)
                .expect("request should still be mutable")
                .hints_mut()
                .insert(name.as_str().to_string(), value);
        }

        Ok(())
    }

    /// Evaluates the command of a task.
    fn evaluate_command(
        &mut self,
        state: &mut State<'_>,
        section: &CommandSection,
    ) -> EvaluationResult<()> {
        debug!(
            "evaluating command section for task `{task}` in `{uri}`",
            task = state.task.name(),
            uri = state.document.uri()
        );

        assert!(state.request.command().is_empty());

        // Map any paths needed for command evaluation
        self.map_request_paths(state)?;

        let mut command = String::new();
        if let Some(parts) = section.strip_whitespace() {
            let mut evaluator = ExprEvaluator::new(TaskEvaluationContext::new(
                state.request.root(),
                state,
                TASK_SCOPE_INDEX,
            ));

            for part in parts {
                match part {
                    StrippedCommandPart::Text(t) => {
                        command.push_str(t.as_str());
                    }
                    StrippedCommandPart::Placeholder(placeholder) => {
                        evaluator.evaluate_placeholder(
                            &placeholder,
                            &mut command,
                            state.request.path_mapping(),
                        )?;
                    }
                }
            }
        } else {
            warn!(
                "command for task `{task}` in `{uri}` has mixed indentation; whitespace stripping \
                 was skipped",
                task = state.task.name(),
                uri = state.document.uri(),
            );

            let mut evaluator = ExprEvaluator::new(TaskEvaluationContext::new(
                state.request.root(),
                state,
                TASK_SCOPE_INDEX,
            ));

            let heredoc = section.is_heredoc();
            for part in section.parts() {
                match part {
                    CommandPart::Text(t) => {
                        t.unescape_to(heredoc, &mut command);
                    }
                    CommandPart::Placeholder(placeholder) => {
                        evaluator.evaluate_placeholder(
                            &placeholder,
                            &mut command,
                            state.request.path_mapping(),
                        )?;
                    }
                }
            }
        }

        *Arc::get_mut(&mut state.request)
            .expect("request should still be mutable")
            .command_mut() = command;
        Ok(())
    }

    /// Evaluates a task output.
    fn evaluate_output(
        &mut self,
        state: &mut State<'_>,
        decl: &Decl,
        evaluated: &EvaluatedTask,
    ) -> EvaluationResult<()> {
        let name = decl.name();
        debug!(
            "evaluating output `{name}` for task `{task}` in `{uri}`",
            name = name.as_str(),
            task = state.task.name(),
            uri = state.document.uri()
        );

        let decl_ty = decl.ty();
        let ty = crate::convert_ast_type_v1(state.document, &decl_ty)?;
        let mut evaluator = ExprEvaluator::new(
            TaskEvaluationContext::new(state.request.root(), state, TASK_SCOPE_INDEX)
                .with_stdout(&evaluated.stdout)
                .with_stderr(&evaluated.stderr),
        );

        let expr = decl.expr().expect("outputs should have expressions");
        let value = evaluator.evaluate_expr(&expr)?;

        // First coerce the output value to the expected type
        let mut value = value
            .coerce(&ty)
            .map_err(|e| runtime_type_mismatch(e, &ty, name.span(), &value.ty(), expr.span()))?;

        // Finally, join the path with the working directory, checking for existence
        value
            .join_paths(&evaluated.work_dir, true, ty.is_optional())
            .map_err(|e| {
                output_evaluation_failed(e, state.task.name(), true, name.as_str(), name.span())
            })?;

        state.scopes[OUTPUT_SCOPE_INDEX.0].insert(name.as_str(), value);
        Ok(())
    }

    /// Maps any host paths to guest paths for task execution.
    fn map_request_paths(&self, _state: &mut State<'_>) -> EvaluationResult<()> {
        // TODO: implement this for container backends
        Ok(())
    }
}
