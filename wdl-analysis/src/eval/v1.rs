//! Evaluation graphs for WDL 1.x.

use std::collections::HashMap;
use std::fmt;

use petgraph::algo::DfsSpace;
use petgraph::algo::has_path_connecting;
use petgraph::algo::toposort;
use petgraph::graph::DiGraph;
use petgraph::graph::NodeIndex;
use petgraph::visit::Visitable;
use wdl_ast::AstNode;
use wdl_ast::AstNodeExt;
use wdl_ast::AstToken;
use wdl_ast::Diagnostic;
use wdl_ast::Ident;
use wdl_ast::SupportedVersion;
use wdl_ast::SyntaxKind;
use wdl_ast::SyntaxNode;
use wdl_ast::TokenStrHash;
use wdl_ast::v1::CallStatement;
use wdl_ast::v1::CommandPart;
use wdl_ast::v1::CommandSection;
use wdl_ast::v1::ConditionalStatement;
use wdl_ast::v1::Decl;
use wdl_ast::v1::Expr;
use wdl_ast::v1::NameRef;
use wdl_ast::v1::RequirementsSection;
use wdl_ast::v1::RuntimeSection;
use wdl_ast::v1::ScatterStatement;
use wdl_ast::v1::TaskDefinition;
use wdl_ast::v1::TaskHintsSection;
use wdl_ast::v1::TaskItem;
use wdl_ast::v1::WorkflowDefinition;
use wdl_ast::v1::WorkflowItem;
use wdl_ast::v1::WorkflowStatement;
use wdl_ast::version::V1;

use crate::diagnostics::NameContext;
use crate::diagnostics::call_conflict;
use crate::diagnostics::name_conflict;
use crate::diagnostics::self_referential;
use crate::diagnostics::task_reference_cycle;
use crate::diagnostics::unknown_name;
use crate::diagnostics::workflow_reference_cycle;
use crate::scope::TASK_VAR_NAME;

/// Represents a node in an task evaluation graph.
#[derive(Debug, Clone)]
pub enum TaskGraphNode {
    /// The node is an input.
    Input(Decl),
    /// The node is a private decl.
    Decl(Decl),
    /// The node is an output decl.
    Output(Decl),
    /// The node is a command section.
    Command(CommandSection),
    /// The node is a `runtime` section.
    Runtime(RuntimeSection),
    /// The node is a `requirements`` section.
    Requirements(RequirementsSection),
    /// The node is a `hints`` section.
    Hints(TaskHintsSection),
}

impl TaskGraphNode {
    /// Gets the context of the name introduced by the node.
    ///
    /// Returns `None` if the node did not introduce a name.
    fn context(&self) -> Option<NameContext> {
        match self {
            Self::Input(decl) => Some(NameContext::Input(decl.name().span())),
            Self::Decl(decl) => Some(NameContext::Decl(decl.name().span())),
            Self::Output(decl) => Some(NameContext::Output(decl.name().span())),
            Self::Command(_) | Self::Runtime(_) | Self::Requirements(_) | Self::Hints(_) => None,
        }
    }

    /// Gets the expression associated with the node.
    ///
    /// Returns `None` if the node has no expression.
    fn expr(&self) -> Option<Expr> {
        match self {
            Self::Input(decl) | Self::Decl(decl) | Self::Output(decl) => decl.expr(),
            Self::Command(_) | Self::Runtime(_) | Self::Requirements(_) | Self::Hints(_) => None,
        }
    }
}

impl fmt::Display for TaskGraphNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Input(decl) | Self::Decl(decl) | Self::Output(decl) => {
                write!(f, "`{name}`", name = decl.name().as_str())
            }
            Self::Command(_) => write!(f, "command section"),
            Self::Runtime(_) => write!(f, "runtime section"),
            Self::Requirements(_) => write!(f, "requirements section"),
            Self::Hints(_) => write!(f, "hints section"),
        }
    }
}

/// Represents a task evaluation graph.
///
/// This is used to evaluate declarations and sections in topological order.
#[derive(Debug, Default)]
pub struct TaskGraph {
    /// The inner directed graph.
    ///
    /// Note that edges in this graph are in *reverse* dependency ordering
    /// (implies "depended upon by" relationships).
    inner: DiGraph<TaskGraphNode, ()>,
    /// The map of declaration names to node indexes in the graph.
    names: HashMap<TokenStrHash<Ident>, NodeIndex>,
    /// The command node index.
    command: Option<NodeIndex>,
    /// The runtime node index.
    runtime: Option<NodeIndex>,
    /// The requirements node index.
    requirements: Option<NodeIndex>,
    /// The hints node index.
    hints: Option<NodeIndex>,
}

impl TaskGraph {
    /// Constructs a new task evaluation graph.
    pub fn new(
        version: SupportedVersion,
        task: &TaskDefinition,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Self {
        // Populate the declaration types and build a name reference graph
        let mut space = Default::default();
        let mut saw_inputs = false;
        let mut outputs = None;
        let mut graph = Self::default();
        for item in task.items() {
            match item {
                TaskItem::Input(section) if !saw_inputs => {
                    saw_inputs = true;
                    for decl in section.declarations() {
                        graph.add_named_node(decl.name(), TaskGraphNode::Input(decl), diagnostics);
                    }
                }
                TaskItem::Output(section) if outputs.is_none() => {
                    outputs = Some(section);
                }
                TaskItem::Declaration(decl) => {
                    graph.add_named_node(
                        decl.name(),
                        TaskGraphNode::Decl(Decl::Bound(decl)),
                        diagnostics,
                    );
                }
                TaskItem::Command(section) if graph.command.is_none() => {
                    graph.command = Some(graph.inner.add_node(TaskGraphNode::Command(section)));
                }
                TaskItem::Runtime(section) if graph.runtime.is_none() => {
                    graph.runtime = Some(graph.inner.add_node(TaskGraphNode::Runtime(section)));
                }
                TaskItem::Requirements(section)
                    if version >= SupportedVersion::V1(V1::Two)
                        && graph.requirements.is_none()
                        && graph.runtime.is_none() =>
                {
                    graph.requirements =
                        Some(graph.inner.add_node(TaskGraphNode::Requirements(section)));
                }
                TaskItem::TaskHints(section)
                    if version >= SupportedVersion::V1(V1::Two)
                        && graph.hints.is_none()
                        && graph.runtime.is_none() =>
                {
                    graph.hints = Some(graph.inner.add_node(TaskGraphNode::Hints(section)));
                }
                _ => continue,
            }
        }

        // Add name reference edges before adding the outputs
        graph.add_reference_edges(version, None, &mut space, diagnostics);

        let count = graph.inner.node_count();
        if let Some(section) = outputs {
            for decl in section.declarations() {
                if let Some(index) = graph.add_named_node(
                    decl.name(),
                    TaskGraphNode::Output(Decl::Bound(decl)),
                    diagnostics,
                ) {
                    // Add an edge to the command node as all outputs depend on the command
                    if let Some(command) = graph.command {
                        graph.inner.update_edge(command, index, ());
                    }
                }
            }
        }

        // Add reference edges again, but only for the output declaration nodes
        graph.add_reference_edges(version, Some(count), &mut space, diagnostics);

        // Finally, add edges from the command to runtime/requirements/hints
        if let Some(command) = graph.command {
            if let Some(runtime) = graph.runtime {
                graph.inner.update_edge(runtime, command, ());
            }

            if let Some(requirements) = graph.requirements {
                graph.inner.update_edge(requirements, command, ());
            }

            if let Some(hints) = graph.hints {
                graph.inner.update_edge(hints, command, ());
            }
        }

        graph
    }

    /// Performs a topological sort of the graph nodes.
    pub fn toposort(&self) -> Vec<TaskGraphNode> {
        toposort(&self.inner, None)
            .expect("graph should be acyclic")
            .into_iter()
            .map(|i| self.inner[i].clone())
            .collect()
    }

    /// Adds a named node to the graph.
    fn add_named_node(
        &mut self,
        name: Ident,
        node: TaskGraphNode,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Option<NodeIndex> {
        // Check for conflicting nodes
        if let Some(existing) = self.names.get(name.as_str()) {
            diagnostics.push(name_conflict(
                name.as_str(),
                node.context().expect("node should have context").into(),
                self.inner[*existing]
                    .context()
                    .expect("node should have context")
                    .into(),
            ));
            return None;
        }

        let index = self.inner.add_node(node);
        self.names.insert(TokenStrHash::new(name), index);
        Some(index)
    }

    /// Adds edges from task sections to declarations.
    fn add_section_edges(
        &mut self,
        from: NodeIndex,
        descendants: impl Iterator<Item = SyntaxNode>,
        allow_task_var: bool,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        // Add edges for any descendant name references
        for r in descendants.filter_map(NameRef::cast) {
            let name = r.name();

            // Look up the name; we don't check for cycles here as decls can't
            // reference a section.
            if let Some(to) = self.names.get(name.as_str()) {
                self.inner.update_edge(*to, from, ());
            } else if name.as_str() != TASK_VAR_NAME || !allow_task_var {
                diagnostics.push(unknown_name(name.as_str(), name.span()));
            }
        }
    }

    /// Adds name reference edges to the graph.
    fn add_reference_edges(
        &mut self,
        version: SupportedVersion,
        skip: Option<usize>,
        space: &mut DfsSpace<NodeIndex, <DiGraph<TaskGraphNode, ()> as Visitable>::Map>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        // Populate edges for any nodes that reference other nodes by name
        for from in self.inner.node_indices().skip(skip.unwrap_or(0)) {
            match self.inner[from].clone() {
                TaskGraphNode::Input(decl) | TaskGraphNode::Decl(decl) => {
                    if let Some(expr) = decl.expr() {
                        self.add_expr_edges(from, expr, false, space, diagnostics);
                    }
                }
                TaskGraphNode::Output(decl) => {
                    if let Some(expr) = decl.expr() {
                        self.add_expr_edges(
                            from,
                            expr,
                            version >= SupportedVersion::V1(V1::Two),
                            space,
                            diagnostics,
                        );
                    }
                }
                TaskGraphNode::Command(section) => {
                    // Add name references from the command section to any decls in scope
                    let section = section.clone();
                    for part in section.parts() {
                        if let CommandPart::Placeholder(p) = part {
                            self.add_section_edges(
                                from,
                                p.syntax().descendants(),
                                version >= SupportedVersion::V1(V1::Two),
                                diagnostics,
                            );
                        }
                    }
                }
                TaskGraphNode::Runtime(section) => {
                    // Add name references from the runtime section to any decls in scope
                    let section = section.clone();
                    for item in section.items() {
                        self.add_section_edges(
                            from,
                            item.syntax().descendants(),
                            false,
                            diagnostics,
                        );
                    }
                }
                TaskGraphNode::Requirements(section) => {
                    // Add name references from the requirements section to any decls in scope
                    let section = section.clone();
                    for item in section.items() {
                        self.add_section_edges(
                            from,
                            item.syntax().descendants(),
                            false,
                            diagnostics,
                        );
                    }
                }
                TaskGraphNode::Hints(section) => {
                    // Add name references from the hints section to any decls in scope
                    let section = section.clone();
                    for item in section.items() {
                        self.add_section_edges(
                            from,
                            item.syntax().descendants(),
                            false,
                            diagnostics,
                        );
                    }
                }
            }
        }
    }

    /// Adds name reference edges for an expression.
    fn add_expr_edges(
        &mut self,
        from: NodeIndex,
        expr: Expr,
        allow_task_var: bool,
        space: &mut DfsSpace<NodeIndex, <DiGraph<TaskGraphNode, ()> as Visitable>::Map>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        for r in expr.syntax().descendants().filter_map(NameRef::cast) {
            let name = r.name();

            // Only add an edge if the name is known
            if let Some(to) = self.names.get(name.as_str()) {
                // Check to see if the node is self-referential
                if *to == from {
                    diagnostics.push(self_referential(
                        name.as_str(),
                        self.inner[from]
                            .context()
                            .expect("node should have context")
                            .span(),
                        name.span(),
                    ));
                    continue;
                }

                // Check for a dependency cycle
                if has_path_connecting(&self.inner, from, *to, Some(space)) {
                    diagnostics.push(task_reference_cycle(
                        &self.inner[from],
                        r.span(),
                        name.as_str(),
                        self.inner[*to]
                            .expr()
                            .expect("should have expr to form a cycle")
                            .span(),
                    ));
                    continue;
                }

                self.inner.update_edge(*to, from, ());
            } else if name.as_str() != TASK_VAR_NAME || !allow_task_var {
                diagnostics.push(unknown_name(name.as_str(), name.span()));
            }
        }
    }
}

/// Represents a node in an workflow evaluation graph.
#[derive(Debug, Clone)]
pub enum WorkflowGraphNode {
    /// The node is an input.
    Input(Decl),
    /// The node is a private decl.
    Decl(Decl),
    /// The node is an output decl.
    Output(Decl),
    /// The node is a conditional statement
    Conditional(ConditionalStatement),
    /// The node is a scatter statement.
    Scatter(ScatterStatement),
    /// The node is a call statement.
    Call(CallStatement),
    /// The node is an exit of a conditional statement.
    ///
    /// This is a special node that is paired with each conditional statement
    /// node.
    ///
    /// It is the point by which the conditional is being exited and the outputs
    /// of the statement are introduced into the parent scope.
    ExitConditional(ConditionalStatement),
    /// The node is an exit of a scatter statement.
    ///
    /// This is a special node that is paired with each scatter statement node.
    ///
    /// It is the point by which the scatter is being exited and the outputs of
    /// the statement are introduced into the parent scope.
    ExitScatter(ScatterStatement),
}

impl WorkflowGraphNode {
    /// Gets the context of the name introduced by the node.
    ///
    /// Returns `None` if the node did not introduce a name.
    fn context(&self) -> Option<NameContext> {
        match self {
            Self::Input(decl) => Some(NameContext::Input(decl.name().span())),
            Self::Decl(decl) => Some(NameContext::Decl(decl.name().span())),
            Self::Output(decl) => Some(NameContext::Output(decl.name().span())),
            Self::Scatter(statement) => {
                Some(NameContext::ScatterVariable(statement.variable().span()))
            }
            Self::Call(statement) => statement
                .alias()
                .map(|a| NameContext::Call(a.name().span()))
                .or_else(|| {
                    statement
                        .target()
                        .names()
                        .last()
                        .map(|t| NameContext::Call(t.span()))
                }),
            Self::Conditional(_) | Self::ExitConditional(_) | Self::ExitScatter(_) => None,
        }
    }

    /// Gets the syntax node associated with the graph node.
    fn syntax(&self) -> &SyntaxNode {
        match self {
            Self::Input(decl) | Self::Decl(decl) | Self::Output(decl) => decl.syntax(),
            Self::Conditional(statement) => statement.syntax(),
            Self::Scatter(statement) => statement.syntax(),
            Self::Call(statement) => statement.syntax(),
            Self::ExitConditional(_) | Self::ExitScatter(_) => {
                unreachable!("exit nodes have no syntax node")
            }
        }
    }
}

impl fmt::Display for WorkflowGraphNode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Input(decl) | Self::Decl(decl) | Self::Output(decl) => {
                write!(f, "`{name}`", name = decl.name().as_str())
            }
            Self::Scatter(statement) => write!(f, "`{name}`", name = statement.variable().as_str()),
            Self::Call(statement) => write!(
                f,
                "`{name}`",
                name = statement
                    .alias()
                    .map(|a| a.name())
                    .or_else(|| statement.target().names().last())
                    .expect("should have name")
                    .as_str()
            ),
            Self::Conditional(_) => write!(f, "conditional expression"),
            Self::ExitConditional(_) | Self::ExitScatter(_) => write!(f, "exit"),
        }
    }
}

/// Represents a workflow evaluation graph.
///
/// This is used to evaluate declarations and statements in topological order.
#[derive(Debug, Default)]
pub struct WorkflowGraph {
    /// The inner directed graph.
    ///
    /// Note that edges in this graph are in *reverse* dependency ordering
    /// (implies "depended upon by" relationships).
    inner: DiGraph<WorkflowGraphNode, ()>,
    /// The map of declaration names to node indexes in the graph.
    names: HashMap<TokenStrHash<Ident>, NodeIndex>,
    /// A stack of scatter variable names.
    variables: Vec<Ident>,
    /// A map of AST syntax nodes to their entry and exit nodes in the graph.
    ///
    /// This is used to add edges to the graph for references to names that are
    /// nested inside of conditional or scatter statements.
    entry_exits: HashMap<SyntaxNode, (NodeIndex, NodeIndex)>,
}

impl WorkflowGraph {
    /// Constructs a new workflow evaluation graph.
    pub fn new(workflow: &WorkflowDefinition, diagnostics: &mut Vec<Diagnostic>) -> Self {
        // Populate the declaration types and build a name reference graph
        let mut space = Default::default();
        let mut finder = CommonAncestorFinder::default();
        let mut saw_inputs = false;
        let mut outputs = None;
        let mut graph = Self::default();
        for item in workflow.items() {
            match item {
                WorkflowItem::Input(section) if !saw_inputs => {
                    saw_inputs = true;
                    for decl in section.declarations() {
                        graph.add_named_node(
                            decl.name(),
                            WorkflowGraphNode::Input(decl),
                            diagnostics,
                        );
                    }
                }
                WorkflowItem::Output(section) => {
                    outputs = Some(section);
                }
                WorkflowItem::Conditional(statement) => {
                    graph.add_workflow_statement(
                        WorkflowStatement::Conditional(statement),
                        None,
                        diagnostics,
                    );
                }
                WorkflowItem::Scatter(statement) => {
                    graph.add_workflow_statement(
                        WorkflowStatement::Scatter(statement),
                        None,
                        diagnostics,
                    );
                }
                WorkflowItem::Call(statement) => {
                    graph.add_workflow_statement(
                        WorkflowStatement::Call(statement),
                        None,
                        diagnostics,
                    );
                }
                WorkflowItem::Declaration(decl) => {
                    graph.add_workflow_statement(
                        WorkflowStatement::Declaration(decl),
                        None,
                        diagnostics,
                    );
                }
                _ => continue,
            }
        }

        // Add name reference edges before adding the outputs
        graph.add_reference_edges(None, &mut finder, &mut space, diagnostics);

        let count = graph.inner.node_count();
        if let Some(section) = outputs {
            for decl in section.declarations() {
                graph.add_named_node(
                    decl.name(),
                    WorkflowGraphNode::Output(Decl::Bound(decl)),
                    diagnostics,
                );
            }
        }

        // Add reference edges again, but only for the output declaration nodes
        graph.add_reference_edges(Some(count), &mut finder, &mut space, diagnostics);
        graph
    }

    /// Performs a topological sort of the graph nodes.
    pub fn toposort(&self) -> Vec<WorkflowGraphNode> {
        toposort(&self.inner, None)
            .expect("graph should be acyclic")
            .into_iter()
            .map(|i| self.inner[i].clone())
            .collect()
    }

    /// Adds nodes from a workflow statement to the graph.
    fn add_workflow_statement(
        &mut self,
        statement: WorkflowStatement,
        parent_entry_exit: Option<(NodeIndex, NodeIndex)>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let entry_exit = match statement {
            WorkflowStatement::Conditional(statement) => {
                // Create the entry and exit nodes for the conditional statement
                // The exit node always depends on the entry node
                let entry = self
                    .inner
                    .add_node(WorkflowGraphNode::Conditional(statement.clone()));
                let exit = self
                    .inner
                    .add_node(WorkflowGraphNode::ExitConditional(statement.clone()));
                self.inner.update_edge(entry, exit, ());
                self.entry_exits
                    .insert(statement.syntax().clone(), (entry, exit));

                // Add all of the statement's statements
                for statement in statement.statements() {
                    self.add_workflow_statement(statement, Some((entry, exit)), diagnostics);
                }

                Some((entry, exit))
            }
            WorkflowStatement::Scatter(statement) => {
                // Create the entry and exit nodes for the scatter statement
                // The exit node always depends on the entry node
                let entry = self
                    .inner
                    .add_node(WorkflowGraphNode::Scatter(statement.clone()));
                let exit = self
                    .inner
                    .add_node(WorkflowGraphNode::ExitScatter(statement.clone()));
                self.inner.update_edge(entry, exit, ());
                self.entry_exits
                    .insert(statement.syntax().clone(), (entry, exit));

                // Push the scatter variable onto the stack if it isn't already conflicting
                let variable = statement.variable();
                let pushed = if let Some(existing) = self.names.get(variable.as_str()) {
                    diagnostics.push(name_conflict(
                        variable.as_str(),
                        NameContext::ScatterVariable(variable.span()).into(),
                        self.inner[*existing]
                            .context()
                            .expect("node should have context")
                            .into(),
                    ));
                    false
                } else {
                    self.variables.push(variable);
                    true
                };

                // Add all of the statement's statements
                for statement in statement.statements() {
                    self.add_workflow_statement(statement, Some((entry, exit)), diagnostics);
                }

                if pushed {
                    self.variables.pop();
                }

                Some((entry, exit))
            }
            WorkflowStatement::Call(statement) => {
                let name = statement.alias().map(|a| a.name()).unwrap_or_else(|| {
                    statement
                        .target()
                        .names()
                        .last()
                        .expect("expected a last call target name")
                });

                self.add_named_node(
                    name,
                    WorkflowGraphNode::Call(statement.clone()),
                    diagnostics,
                )
                // The calls's node is both the entry and exit nodes
                .map(|i| (i, i))
            }
            WorkflowStatement::Declaration(decl) => self
                .add_named_node(
                    decl.name(),
                    WorkflowGraphNode::Decl(Decl::Bound(decl)),
                    diagnostics,
                )
                // The declaration's node is both the entry and exit nodes
                .map(|i| (i, i)),
        };

        // Add (reverse) dependency edges to parent entry from child entry and to child
        // exit from parent exit
        if let (Some((entry, exit)), Some((parent_entry, parent_exit))) =
            (entry_exit, parent_entry_exit)
        {
            self.inner.update_edge(parent_entry, entry, ());
            self.inner.update_edge(exit, parent_exit, ());
        }
    }

    /// Adds a named node to the graph.
    fn add_named_node(
        &mut self,
        name: Ident,
        node: WorkflowGraphNode,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Option<NodeIndex> {
        // Check for a conflicting name, either from a declaration or from a scatter
        // variable
        let (context, cont) = if let Some(existing) = self.names.get(name.as_str()) {
            // Conflict with a declaration
            (
                Some(
                    self.inner[*existing]
                        .context()
                        .expect("node should have context"),
                ),
                false,
            )
        } else if let Some(existing) = self.variables.iter().find(|i| i.as_str() == name.as_str()) {
            // Conflict with a scatter variable; we continue to add the node so that any
            // declaration overrides the scatter variable
            (Some(NameContext::ScatterVariable(existing.span())), true)
        } else {
            // No conflict
            (None, true)
        };

        // Check to see if a diagnostic should be added
        if let Some(context) = context {
            let diagnostic = if let WorkflowGraphNode::Call(call) = &node {
                call_conflict(&name, context, call.alias().is_none())
            } else {
                name_conflict(
                    name.as_str(),
                    node.context().expect("node should have context").into(),
                    context.into(),
                )
            };

            diagnostics.push(diagnostic);

            if !cont {
                return None;
            }
        }

        let index = self.inner.add_node(node);
        self.names.insert(TokenStrHash::new(name), index);
        Some(index)
    }

    /// Adds name reference edges to the graph.
    fn add_reference_edges(
        &mut self,
        skip: Option<usize>,
        finder: &mut CommonAncestorFinder,
        space: &mut DfsSpace<NodeIndex, <DiGraph<TaskGraphNode, ()> as Visitable>::Map>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        // Populate edges for any nodes that reference other nodes by name
        for from in self.inner.node_indices().skip(skip.unwrap_or(0)) {
            match self.inner[from].clone() {
                WorkflowGraphNode::Input(decl)
                | WorkflowGraphNode::Decl(decl)
                | WorkflowGraphNode::Output(decl) => {
                    if let Some(expr) = decl.expr() {
                        self.add_expr_edges(finder, from, expr, space, diagnostics);
                    }
                }
                WorkflowGraphNode::Conditional(statement) => {
                    self.add_expr_edges(finder, from, statement.expr(), space, diagnostics);
                }
                WorkflowGraphNode::Scatter(statement) => {
                    self.add_expr_edges(finder, from, statement.expr(), space, diagnostics);
                }
                WorkflowGraphNode::Call(statement) => {
                    // Add edges for the input expressions
                    // If an input does not have an expression, add an edge to the name
                    for input in statement.inputs() {
                        let name = input.name();
                        if let Some(expr) = input.expr() {
                            self.add_expr_edges(finder, from, expr, space, diagnostics);
                        } else if let Some(to) =
                            self.find_node_by_name(name.as_str(), input.syntax().clone())
                        {
                            // Check for a dependency cycle
                            if has_path_connecting(&self.inner, from, to, Some(space)) {
                                diagnostics.push(workflow_reference_cycle(
                                    &self.inner[from],
                                    name.span(),
                                    name.as_str(),
                                    self.inner[to]
                                        .context()
                                        .expect("node should have context")
                                        .span(),
                                ));
                                continue;
                            }

                            self.add_dependency_edge(finder, from, to);
                        }
                    }

                    // Add edges to other the requested calls
                    for after in statement.after() {
                        let name = after.name();
                        if let Some(to) =
                            self.find_node_by_name(name.as_str(), after.syntax().clone())
                        {
                            // Check for a dependency cycle
                            if has_path_connecting(&self.inner, from, to, Some(space)) {
                                diagnostics.push(workflow_reference_cycle(
                                    &self.inner[from],
                                    name.span(),
                                    name.as_str(),
                                    self.inner[to]
                                        .context()
                                        .expect("node should have context")
                                        .span(),
                                ));
                                continue;
                            }

                            self.add_dependency_edge(finder, from, to);
                        }
                    }
                }
                WorkflowGraphNode::ExitConditional(_) | WorkflowGraphNode::ExitScatter(_) => {
                    continue;
                }
            }
        }
    }

    /// Adds name reference edges for an expression.
    fn add_expr_edges(
        &mut self,
        finder: &mut CommonAncestorFinder,
        from: NodeIndex,
        expr: Expr,
        space: &mut DfsSpace<NodeIndex, <DiGraph<TaskGraphNode, ()> as Visitable>::Map>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        for r in expr.syntax().descendants().filter_map(NameRef::cast) {
            let name = r.name();

            // Only add an edge if the name is known
            if let Some(to) = self.find_node_by_name(name.as_str(), expr.syntax().clone()) {
                // Check to see if the node is self-referential
                if to == from {
                    diagnostics.push(self_referential(
                        name.as_str(),
                        self.inner[from]
                            .context()
                            .expect("node should have a context")
                            .span(),
                        name.span(),
                    ));
                    continue;
                }

                // Check for a dependency cycle
                if has_path_connecting(&self.inner, from, to, Some(space)) {
                    diagnostics.push(workflow_reference_cycle(
                        &self.inner[from],
                        r.span(),
                        name.as_str(),
                        self.inner[to]
                            .context()
                            .expect("node should have context")
                            .span(),
                    ));
                    continue;
                }

                self.add_dependency_edge(finder, from, to);
            } else {
                diagnostics.push(unknown_name(name.as_str(), name.span()));
            }
        }
    }

    /// Adds a dependency edge between two nodes.
    ///
    /// Dependency edges can only be formed between nodes at the same "scope".
    ///
    /// This works by walking up the AST ancestors looking for a common ancestor
    /// (A) of the two nodes.
    ///
    /// For the child of A that is an ancestor of `to` (or `to` itself), we use
    /// the exit node of that child if there is one.
    ///
    /// For the child of A this is an ancestor of `from` (or `from` itself), we
    /// use the entry node of that child if there is one.
    ///
    /// If either child does not have an entry/exit node, the original nodes are
    /// used.
    fn add_dependency_edge(
        &mut self,
        finder: &mut CommonAncestorFinder,
        from: NodeIndex,
        to: NodeIndex,
    ) {
        assert!(from != to, "cannot add a self dependency edge");

        let (from, to) = if let Some((f, t)) = finder.find_children_of_common_ancestor(
            self.inner[from].syntax().ancestors(),
            self.inner[to].syntax().ancestors(),
            SyntaxKind::WorkflowDefinitionNode,
        ) {
            let from = self
                .entry_exits
                .get(&f)
                .map(|(entry, _)| *entry)
                .unwrap_or(from);
            let to = self
                .entry_exits
                .get(&t)
                .map(|(_, exit)| *exit)
                .unwrap_or(to);
            (from, to)
        } else {
            (from, to)
        };

        if from == to {
            // No need to add an edge when the entry and exit are the same node
            // This can occur for scatter variables referenced within the scatter body
            return;
        }

        // Add the actual edge in reverse order
        self.inner.update_edge(to, from, ());
    }

    /// Finds a node in the graph by name for the referencing expression.
    ///
    /// This takes into account finding a scatter variable that's in scope.
    fn find_node_by_name(&self, name: &str, expr: SyntaxNode) -> Option<NodeIndex> {
        // If the name came from a declaration or call, return the node
        if let Some(index) = self.names.get(name) {
            return Some(*index);
        }

        // Otherwise, we need to walk up the parent chain looking for a scatter variable
        // with the name
        let mut current = expr;
        while let Some(parent) = current.parent() {
            if let SyntaxKind::ScatterStatementNode = parent.kind() {
                let statement = ScatterStatement::cast(parent.clone()).expect("node should cast");
                let variable = statement.variable();
                if variable.as_str() == name {
                    // Return the entry node for the scatter statement
                    return Some(self.entry_exits[&parent].0);
                }
            }

            current = parent;
        }

        None
    }
}

/// A helper for finding the children of a common ancestor in the AST.
///
/// This exists so we can reuse previously allocated space when adding
/// dependency edges.
#[derive(Debug, Default)]
struct CommonAncestorFinder {
    /// The stack of ancestors for the `first` node.
    first: Vec<SyntaxNode>,
    /// The stack of ancestors for the `second` node.
    second: Vec<SyntaxNode>,
}

impl CommonAncestorFinder {
    /// Finds the children of a common ancestor in two list of ancestors.
    fn find_children_of_common_ancestor(
        &mut self,
        first: impl Iterator<Item = SyntaxNode>,
        second: impl Iterator<Item = SyntaxNode>,
        stop: SyntaxKind,
    ) -> Option<(SyntaxNode, SyntaxNode)> {
        self.first.clear();
        for ancestor in first {
            self.first.push(ancestor.clone());
            if ancestor.kind() == stop {
                break;
            }
        }

        self.second.clear();
        for ancestor in second {
            self.second.push(ancestor.clone());
            if ancestor.kind() == stop {
                break;
            }
        }

        for (first, second) in self.first.iter().rev().zip(self.second.iter().rev()) {
            if first == second {
                continue;
            }

            return Some((first.clone(), second.clone()));
        }

        None
    }
}
