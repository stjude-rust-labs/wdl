//! Evaluation graphs for WDL 1.x.

use std::fmt;

use indexmap::IndexMap;
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
use wdl_ast::Span;
use wdl_ast::SupportedVersion;
use wdl_ast::SyntaxNode;
use wdl_ast::TokenStrHash;
use wdl_ast::v1::CommandPart;
use wdl_ast::v1::CommandSection;
use wdl_ast::v1::Decl;
use wdl_ast::v1::NameRef;
use wdl_ast::v1::RequirementsSection;
use wdl_ast::v1::RuntimeSection;
use wdl_ast::v1::TaskDefinition;
use wdl_ast::v1::TaskHintsSection;
use wdl_ast::v1::TaskItem;
use wdl_ast::version::V1;

use crate::scope::TASK_VAR_NAME;

/// Represents context of a declaration in a task.
enum TaskDeclContext {
    /// The name was introduced by an task input.
    Input(Span),
    /// The name was introduced by an task output.
    Output(Span),
    /// The name was introduced by a private declaration.
    Decl(Span),
}

impl TaskDeclContext {
    /// Gets the span of the name.
    pub fn span(&self) -> Span {
        match self {
            Self::Input(s) => *s,
            Self::Output(s) => *s,
            Self::Decl(s) => *s,
        }
    }
}

impl fmt::Display for TaskDeclContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Input(_) => write!(f, "input"),
            Self::Output(_) => write!(f, "output"),
            Self::Decl(_) => write!(f, "declaration"),
        }
    }
}

impl From<TaskGraphNode> for TaskDeclContext {
    fn from(value: TaskGraphNode) -> Self {
        match value {
            TaskGraphNode::Input(decl) => Self::Input(decl.name().span()),
            TaskGraphNode::Decl(decl) => Self::Decl(decl.name().span()),
            TaskGraphNode::Output(decl) => Self::Output(decl.name().span()),
            _ => unreachable!("expected a declaration node"),
        }
    }
}

/// Creates a "task decl conflict" diagnostic
fn task_decl_conflict(
    name: &str,
    conflicting: TaskDeclContext,
    first: TaskDeclContext,
) -> Diagnostic {
    Diagnostic::error(format!("conflicting {conflicting} name `{name}`"))
        .with_label(
            format!("this {conflicting} conflicts with a previously used name"),
            conflicting.span(),
        )
        .with_label(
            format!("the {first} with the conflicting name is here"),
            first.span(),
        )
}

/// Creates an "unknown name" diagnostic.
fn unknown_name(name: &str, span: Span) -> Diagnostic {
    // Handle special case names here
    let message = match name {
        "task" => "the `task` variable may only be used within a task command section or task \
                   output section using WDL 1.2 or later"
            .to_string(),
        _ => format!("unknown name `{name}`"),
    };

    Diagnostic::error(message).with_highlight(span)
}

/// Creates a "self-referential" diagnostic.
fn self_referential(name: &str, span: Span, reference: Span) -> Diagnostic {
    Diagnostic::error(format!("declaration of `{name}` is self-referential"))
        .with_label("self-reference is here", reference)
        .with_highlight(span)
}

/// Creates a "reference cycle" diagnostic.
fn reference_cycle(from: &str, from_span: Span, to: &str, to_span: Span) -> Diagnostic {
    Diagnostic::error("a name reference cycle was detected")
        .with_label(
            format!("ensure this expression does not directly or indirectly refer to `{from}`"),
            to_span,
        )
        .with_label(format!("a reference back to `{to}` is here"), from_span)
}

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
    names: IndexMap<TokenStrHash<Ident>, NodeIndex>,
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
                        graph.add_decl_node(decl.name(), TaskGraphNode::Input(decl), diagnostics);
                    }
                }
                TaskItem::Output(section) if outputs.is_none() => {
                    outputs = Some(section);
                }
                TaskItem::Declaration(decl) => {
                    graph.add_decl_node(
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
                TaskItem::Hints(section)
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
                if let Some(index) = graph.add_decl_node(
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

    /// Adds a declaration node to the graph.
    fn add_decl_node(
        &mut self,
        name: Ident,
        node: TaskGraphNode,
        diagnostics: &mut Vec<Diagnostic>,
    ) -> Option<NodeIndex> {
        // Check for conflicting nodes
        if let Some(existing) = self.names.get(name.as_str()) {
            diagnostics.push(task_decl_conflict(
                name.as_str(),
                node.into(),
                self.inner[*existing].clone().into(),
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
                    self.add_decl_edges(from, decl, false, space, diagnostics);
                }
                TaskGraphNode::Output(decl) => {
                    self.add_decl_edges(
                        from,
                        decl,
                        version >= SupportedVersion::V1(V1::Two),
                        space,
                        diagnostics,
                    );
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

    /// Adds name reference edges for a declaration.
    fn add_decl_edges(
        &mut self,
        from: NodeIndex,
        decl: Decl,
        allow_task_var: bool,
        space: &mut DfsSpace<NodeIndex, <DiGraph<TaskGraphNode, ()> as Visitable>::Map>,
        diagnostics: &mut Vec<Diagnostic>,
    ) {
        let span = decl.name().span();
        let expr = decl.expr();
        if let Some(expr) = expr {
            for r in expr.syntax().descendants().filter_map(NameRef::cast) {
                let name = r.name();

                // Only add an edge if the name is known to us
                if let Some(to) = self.names.get(name.as_str()) {
                    // Check to see if the node is self-referential
                    if *to == from {
                        diagnostics.push(self_referential(name.as_str(), span, name.span()));
                        continue;
                    }

                    // Check for a dependency cycle
                    if has_path_connecting(&self.inner, from, *to, Some(space)) {
                        diagnostics.push(reference_cycle(
                            self.names
                                .get_index(from.index())
                                .unwrap()
                                .0
                                .as_ref()
                                .as_str(),
                            r.span(),
                            name.as_str(),
                            match &self.inner[*to] {
                                TaskGraphNode::Decl(to) => {
                                    to.expr().expect("should have expr to form a cycle").span()
                                }
                                _ => panic!("expected decl node"),
                            },
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
}
