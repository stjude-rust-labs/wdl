//! Representation of analyzed WDL documents.

use std::cmp::Ordering;
use std::collections::HashMap;
use std::str::FromStr;
use std::sync::Arc;

use indexmap::IndexMap;
use petgraph::graph::NodeIndex;
use rowan::GreenNode;
use url::Url;
use uuid::Uuid;
use wdl_ast::Ast;
use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::Diagnostic;
use wdl_ast::Span;
use wdl_ast::SupportedVersion;
use wdl_ast::SyntaxNode;

use crate::DiagnosticsConfig;
use crate::diagnostics::unused_import;
use crate::graph::DocumentGraph;
use crate::graph::ParseState;
use crate::types::CallType;
use crate::types::Type;

mod v1;

/// The `task` variable name available in task command sections and outputs in
/// WDL 1.2.
pub const TASK_VAR_NAME: &str = "task";

/// Represents a namespace introduced by an import.
#[derive(Debug)]
pub struct Namespace {
    /// The span of the import that introduced the namespace.
    span: Span,
    /// The URI of the imported document that introduced the namespace.
    source: Arc<Url>,
    /// The namespace's document.
    document: Arc<Document>,
    /// Whether or not the namespace is used (i.e. referenced) in the document.
    used: bool,
    /// Whether or not the namespace is excepted from the "unused import"
    /// diagnostic.
    excepted: bool,
}

impl Namespace {
    /// Gets the span of the import that introduced the namespace.
    pub fn span(&self) -> Span {
        self.span
    }

    /// Gets the URI of the imported document that introduced the namespace.
    pub fn source(&self) -> &Arc<Url> {
        &self.source
    }

    /// Gets the imported document.
    pub fn document(&self) -> &Document {
        &self.document
    }
}

/// Represents a struct in a document.
#[derive(Debug, Clone)]
pub struct Struct {
    /// The span that introduced the struct.
    ///
    /// This is either the name of a struct definition (local) or an import's
    /// URI or alias (imported).
    span: Span,
    /// The offset of the CST node from the start of the document.
    ///
    /// This is used to adjust diagnostics resulting from traversing the struct
    /// node as if it were the root of the CST.
    offset: usize,
    /// Stores the CST node of the struct.
    ///
    /// This is used to calculate type equivalence for imports.
    node: rowan::GreenNode,
    /// The namespace that defines the struct.
    ///
    /// This is `Some` only for imported structs.
    namespace: Option<String>,
    /// The type of the struct.
    ///
    /// Initially this is `None` until a type check occurs.
    ty: Option<Type>,
}

impl Struct {
    /// Gets the namespace that defines this struct.
    ///
    /// Returns `None` for structs defined in the containing document or `Some`
    /// for a struct introduced by an import.
    pub fn namespace(&self) -> Option<&str> {
        self.namespace.as_deref()
    }

    /// Gets the type of the struct.
    ///
    /// A value of `None` indicates that the type could not be determined for
    /// the struct; this may happen if the struct definition is recursive.
    pub fn ty(&self) -> Option<&Type> {
        self.ty.as_ref()
    }
}

/// Represents information about a name in a scope.
#[derive(Debug, Clone)]
pub struct Name {
    /// The span of the name.
    span: Span,
    /// The type of the name.
    ty: Type,
}

impl Name {
    /// Gets the span of the name.
    pub fn span(&self) -> Span {
        self.span
    }

    /// Gets the type of the name.
    pub fn ty(&self) -> &Type {
        &self.ty
    }
}

/// Represents an index of a scope in a collection of scopes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct ScopeIndex(usize);

/// Represents a scope in a WDL document.
#[derive(Debug)]
struct Scope {
    /// The index of the parent scope.
    ///
    /// This is `None` for task and workflow scopes.
    parent: Option<ScopeIndex>,
    /// The span in the document where the names of the scope are visible.
    span: Span,
    /// The map of names in scope to their span and types.
    names: IndexMap<String, Name>,
}

impl Scope {
    /// Creates a new scope given the parent scope and span.
    fn new(parent: Option<ScopeIndex>, span: Span) -> Self {
        Self {
            parent,
            span,
            names: Default::default(),
        }
    }

    /// Inserts a name into the scope.
    pub fn insert(&mut self, name: impl Into<String>, span: Span, ty: Type) {
        self.names.insert(name.into(), Name { span, ty });
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
    fn new(scopes: &'a [Scope], index: ScopeIndex) -> Self {
        Self { scopes, index }
    }

    /// Gets the span of the scope.
    pub fn span(&self) -> Span {
        self.scopes[self.index.0].span
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

    /// Gets all of the names available at this scope.
    pub fn names(&self) -> impl Iterator<Item = (&str, &Name)> + use<'_> {
        self.scopes[self.index.0]
            .names
            .iter()
            .map(|(name, n)| (name.as_str(), n))
    }

    /// Gets a name local to this scope.
    ///
    /// Returns `None` if a name local to this scope was not found.
    pub fn local(&self, name: &str) -> Option<&Name> {
        self.scopes[self.index.0].names.get(name)
    }

    /// Lookups a name in the scope.
    ///
    /// Returns `None` if the name is not available in the scope.
    pub fn lookup(&self, name: &str) -> Option<&Name> {
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

/// Represents a mutable reference to a scope.
#[derive(Debug)]
struct ScopeRefMut<'a> {
    /// The reference to all scopes.
    scopes: &'a mut [Scope],
    /// The index to the scope.
    index: ScopeIndex,
}

impl<'a> ScopeRefMut<'a> {
    /// Creates a new mutable scope reference given the scope index.
    fn new(scopes: &'a mut [Scope], index: ScopeIndex) -> Self {
        Self { scopes, index }
    }

    /// Lookups a name in the scope.
    ///
    /// Returns `None` if the name is not available in the scope.
    pub fn lookup(&self, name: &str) -> Option<&Name> {
        let mut current = Some(self.index);

        while let Some(index) = current {
            if let Some(name) = self.scopes[index.0].names.get(name) {
                return Some(name);
            }

            current = self.scopes[index.0].parent;
        }

        None
    }

    /// Inserts a name into the scope.
    pub fn insert(&mut self, name: impl Into<String>, span: Span, ty: Type) {
        self.scopes[self.index.0]
            .names
            .insert(name.into(), Name { span, ty });
    }

    /// Converts the mutable scope reference to an immutable scope reference.
    pub fn as_scope_ref(&'a self) -> ScopeRef<'a> {
        ScopeRef {
            scopes: self.scopes,
            index: self.index,
        }
    }
}

/// Represents a task or workflow input.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Input {
    /// The type of the input.
    ty: Type,
    /// Whether or not the input is required.
    required: bool,
}

impl Input {
    /// Gets the type of the input.
    pub fn ty(&self) -> &Type {
        &self.ty
    }

    /// Whether or not the input is required.
    pub fn required(&self) -> bool {
        self.required
    }
}

/// Represents a task or workflow output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Output {
    /// The type of the output.
    ty: Type,
}

impl Output {
    /// Creates a new output with the given type.
    pub(crate) fn new(ty: Type) -> Self {
        Self { ty }
    }

    /// Gets the type of the output.
    pub fn ty(&self) -> &Type {
        &self.ty
    }
}

/// Represents a task in a document.
#[derive(Debug)]
pub struct Task {
    /// The span of the task name.
    name_span: Span,
    /// The name of the task.
    name: String,
    /// The scopes contained in the task.
    ///
    /// The first scope will always be the task's scope.
    ///
    /// The scopes will be in sorted order by span start.
    scopes: Vec<Scope>,
    /// The inputs of the task.
    inputs: Arc<IndexMap<String, Input>>,
    /// The outputs of the task.
    outputs: Arc<IndexMap<String, Output>>,
}

impl Task {
    /// Gets the name of the task.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets the scope of the task.
    pub fn scope(&self) -> ScopeRef<'_> {
        ScopeRef::new(&self.scopes, ScopeIndex(0))
    }

    /// Gets the inputs of the task.
    pub fn inputs(&self) -> &IndexMap<String, Input> {
        &self.inputs
    }

    /// Gets the outputs of the task.
    pub fn outputs(&self) -> &IndexMap<String, Output> {
        &self.outputs
    }
}

/// Represents a workflow in a document.
#[derive(Debug)]
pub struct Workflow {
    /// The span of the workflow name.
    name_span: Span,
    /// The name of the workflow.
    name: String,
    /// The scopes contained in the workflow.
    ///
    /// The first scope will always be the workflow's scope.
    ///
    /// The scopes will be in sorted order by span start.
    scopes: Vec<Scope>,
    /// The inputs of the workflow.
    inputs: Arc<IndexMap<String, Input>>,
    /// The outputs of the workflow.
    outputs: Arc<IndexMap<String, Output>>,
    /// The calls made by the workflow.
    calls: HashMap<String, CallType>,
    /// Whether or not nested inputs are allowed for the workflow.
    allows_nested_inputs: bool,
}

impl Workflow {
    /// Gets the name of the workflow.
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Gets the scope of the workflow.
    pub fn scope(&self) -> ScopeRef<'_> {
        ScopeRef::new(&self.scopes, ScopeIndex(0))
    }

    /// Gets the inputs of the workflow.
    pub fn inputs(&self) -> &IndexMap<String, Input> {
        &self.inputs
    }

    /// Gets the outputs of the workflow.
    pub fn outputs(&self) -> &IndexMap<String, Output> {
        &self.outputs
    }

    /// Gets the calls made by the workflow.
    pub fn calls(&self) -> &HashMap<String, CallType> {
        &self.calls
    }

    /// Determines if the workflow allows nested inputs.
    pub fn allows_nested_inputs(&self) -> bool {
        self.allows_nested_inputs
    }
}

/// Represents an analyzed WDL document.
#[derive(Debug)]
pub struct Document {
    /// The root CST node of the document.
    ///
    /// This is `None` when the document could not be parsed.
    root: Option<GreenNode>,
    /// The document identifier.
    ///
    /// The identifier changes every time the document is analyzed.
    id: Arc<String>,
    /// The URI of the analyzed document.
    uri: Arc<Url>,
    /// The version of the document.
    version: Option<SupportedVersion>,
    /// The namespaces in the document.
    namespaces: IndexMap<String, Namespace>,
    /// The tasks in the document.
    tasks: IndexMap<String, Task>,
    /// The singular workflow in the document.
    workflow: Option<Workflow>,
    /// The structs in the document.
    structs: IndexMap<String, Struct>,
    /// The diagnostics for the document.
    diagnostics: Vec<Diagnostic>,
}

impl Document {
    /// Creates a new analyzed document from a document graph node.
    pub(crate) fn from_graph_node(
        config: DiagnosticsConfig,
        graph: &DocumentGraph,
        index: NodeIndex,
    ) -> Self {
        let node = graph.get(index);

        let diagnostics = match node.parse_state() {
            ParseState::NotParsed => panic!("node should have been parsed"),
            ParseState::Error(_) => {
                return Self::new(node.uri().clone(), None, None, Default::default());
            }
            ParseState::Parsed { diagnostics, .. } => {
                Vec::from_iter(diagnostics.as_ref().iter().cloned())
            }
        };

        let root = node.document().expect("node should have been parsed");
        let (version, config) = match root.version_statement() {
            Some(stmt) => (stmt.version(), config.excepted_for_node(stmt.syntax())),
            None => {
                // Don't process a document with a missing version
                return Self::new(
                    node.uri().clone(),
                    Some(root.syntax().green().into()),
                    None,
                    diagnostics,
                );
            }
        };

        let mut document = Self::new(
            node.uri().clone(),
            Some(root.syntax().green().into()),
            SupportedVersion::from_str(version.as_str()).ok(),
            diagnostics,
        );
        match root.ast() {
            Ast::Unsupported => {}
            Ast::V1(ast) => {
                v1::populate_document(&mut document, config, graph, index, &ast, &version)
            }
        }

        // Check for unused imports
        if let Some(severity) = config.unused_import {
            let Document {
                namespaces,
                diagnostics,
                ..
            } = &mut document;

            diagnostics.extend(
                namespaces
                    .iter()
                    .filter(|(_, ns)| !ns.used && !ns.excepted)
                    .map(|(name, ns)| unused_import(name, ns.span()).with_severity(severity)),
            );
        }

        // Sort the diagnostics by start
        document
            .diagnostics
            .sort_by(|a, b| match (a.labels().next(), b.labels().next()) {
                (None, None) => Ordering::Equal,
                (None, Some(_)) => Ordering::Less,
                (Some(_), None) => Ordering::Greater,
                (Some(a), Some(b)) => a.span().start().cmp(&b.span().start()),
            });

        document
    }

    /// Constructs a new analysis document.
    fn new(
        uri: Arc<Url>,
        root: Option<GreenNode>,
        version: Option<SupportedVersion>,
        diagnostics: Vec<Diagnostic>,
    ) -> Self {
        Self {
            root,
            id: Uuid::new_v4().to_string().into(),
            uri,
            version,
            namespaces: Default::default(),
            tasks: Default::default(),
            workflow: Default::default(),
            structs: Default::default(),
            diagnostics,
        }
    }

    /// Gets the root AST document node.
    pub fn node(&self) -> wdl_ast::Document {
        wdl_ast::Document::cast(SyntaxNode::new_root(
            self.root.clone().expect("should have a root"),
        ))
        .expect("should cast")
    }

    /// Gets the identifier of the document.
    ///
    /// This value changes when a document is reanalyzed.
    pub fn id(&self) -> &Arc<String> {
        &self.id
    }

    /// Gets the URI of the document.
    pub fn uri(&self) -> &Arc<Url> {
        &self.uri
    }

    /// Gets the supported version of the document.
    ///
    /// Returns `None` if the document could not be parsed or contains an
    /// unsupported version.
    pub fn version(&self) -> Option<SupportedVersion> {
        self.version
    }

    /// Gets the namespaces in the document.
    pub fn namespaces(&self) -> impl Iterator<Item = (&str, &Namespace)> {
        self.namespaces.iter().map(|(n, ns)| (n.as_str(), ns))
    }

    /// Gets a namespace in the document by name.
    pub fn namespace(&self, name: &str) -> Option<&Namespace> {
        self.namespaces.get(name)
    }

    /// Gets the tasks in the document.
    pub fn tasks(&self) -> impl Iterator<Item = &Task> {
        self.tasks.iter().map(|(_, t)| t)
    }

    /// Gets a task in the document by name.
    pub fn task_by_name(&self, name: &str) -> Option<&Task> {
        self.tasks.get(name)
    }

    /// Gets a workflow in the document.
    ///
    /// Returns `None` if the document did not contain a workflow.
    pub fn workflow(&self) -> Option<&Workflow> {
        self.workflow.as_ref()
    }

    /// Gets the structs in the document.
    pub fn structs(&self) -> impl Iterator<Item = (&str, &Struct)> {
        self.structs.iter().map(|(n, s)| (n.as_str(), s))
    }

    /// Gets a struct in the document by name.
    pub fn struct_by_name(&self, name: &str) -> Option<&Struct> {
        self.structs.get(name)
    }

    /// Gets the analysis diagnostics for the document.
    pub fn diagnostics(&self) -> &[Diagnostic] {
        &self.diagnostics
    }

    /// Finds a scope based on a position within the document.
    pub fn find_scope_by_position(&self, position: usize) -> Option<ScopeRef<'_>> {
        /// Finds a scope within a collection of sorted scopes by position.
        fn find_scope(scopes: &[Scope], position: usize) -> Option<ScopeRef<'_>> {
            let mut index = match scopes.binary_search_by_key(&position, |s| s.span.start()) {
                Ok(index) => index,
                Err(index) => {
                    // This indicates that we couldn't find a match and the match would go _before_
                    // the first scope, so there is no containing scope.
                    if index == 0 {
                        return None;
                    }

                    index - 1
                }
            };

            // We now have the index to start looking up the list of scopes
            // We walk up the list to try to find a span that contains the position
            loop {
                let scope = &scopes[index];
                if scope.span.contains(position) {
                    return Some(ScopeRef::new(scopes, ScopeIndex(index)));
                }

                if index == 0 {
                    return None;
                }

                index -= 1;
            }
        }

        // Check to see if the position is contained in the workflow
        if let Some(workflow) = &self.workflow {
            if workflow.scope().span().contains(position) {
                return find_scope(&workflow.scopes, position);
            }
        }

        // Search for a task that might contain the position
        let task = match self
            .tasks
            .binary_search_by_key(&position, |_, t| t.scope().span().start())
        {
            Ok(index) => &self.tasks[index],
            Err(index) => {
                // This indicates that we couldn't find a match and the match would go _before_
                // the first task, so there is no containing task.
                if index == 0 {
                    return None;
                }

                &self.tasks[index - 1]
            }
        };

        if task.scope().span().contains(position) {
            return find_scope(&task.scopes, position);
        }

        None
    }
}
