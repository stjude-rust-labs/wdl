//! Representation of the analysis document graph.

use std::collections::HashSet;
use std::fs;
use std::sync::Arc;
use std::time::Duration;
use std::time::Instant;

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use anyhow::bail;
use indexmap::IndexMap;
use indexmap::IndexSet;
use line_index::LineIndex;
use petgraph::Direction;
use petgraph::algo::has_path_connecting;
use petgraph::graph::NodeIndex;
use petgraph::stable_graph::StableDiGraph;
use petgraph::visit::Bfs;
use petgraph::visit::EdgeRef;
use petgraph::visit::Visitable;
use reqwest::Client;
use rowan::GreenNode;
use tokio::runtime::Handle;
use tracing::debug;
use tracing::info;
use url::Url;
use uuid::Uuid;
use wdl_ast::Diagnostic;
use wdl_ast::SyntaxNode;
use wdl_ast::Validator;

use crate::IncrementalChange;
use crate::scope::DocumentScope;

/// Represents space for a DFS search of a document graph.
pub type DfsSpace =
    petgraph::algo::DfsSpace<NodeIndex, <StableDiGraph<DocumentGraphNode, ()> as Visitable>::Map>;

/// Represents diagnostics for a document node.
#[derive(Debug, Clone)]
pub enum Diagnostics {
    /// The diagnostics are from the parse.
    Parse(Arc<[Diagnostic]>),
    /// The diagnostics are from validation.
    ///
    /// This implies there were no parse diagnostics.
    Validation(Arc<[Diagnostic]>),
}

impl AsRef<Arc<[Diagnostic]>> for Diagnostics {
    fn as_ref(&self) -> &Arc<[Diagnostic]> {
        match self {
            Self::Parse(d) => d,
            Self::Validation(d) => d,
        }
    }
}

/// Represents the parse state of a document graph node.
#[derive(Debug, Clone)]
pub enum ParseState {
    /// The document is not parsed.
    NotParsed,
    /// There was an error parsing the document.
    Error(Arc<anyhow::Error>),
    /// The document was parsed.
    Parsed {
        /// The monotonic version of the document that was parsed.
        ///
        /// This value comes from incremental changes to the file.
        ///
        /// If `None`, the parsed version had no incremental changes.
        version: Option<i32>,
        /// The root CST node of.
        root: GreenNode,
        /// The line index of the document.
        lines: Arc<LineIndex>,
        /// The diagnostics.
        diagnostics: Diagnostics,
    },
}

/// Represents the analysis state of a document graph node.
#[derive(Debug)]
pub struct Analysis {
    /// The unique identifier of the analysis.
    id: Arc<String>,
    /// The document's scope.
    scope: Arc<DocumentScope>,
    /// The analysis diagnostics.
    diagnostics: Arc<[Diagnostic]>,
}

impl Analysis {
    /// Constructs a new analysis.
    pub fn new(scope: DocumentScope, diagnostics: impl Into<Arc<[Diagnostic]>>) -> Self {
        Self {
            id: Arc::new(Uuid::new_v4().to_string()),
            scope: Arc::new(scope),
            diagnostics: diagnostics.into(),
        }
    }

    /// Gets the analysis result id.
    ///
    /// The identifier changes every time the document is analyzed.
    pub fn id(&self) -> &Arc<String> {
        &self.id
    }

    /// Gets the document scope from the analysis.
    pub fn scope(&self) -> &Arc<DocumentScope> {
        &self.scope
    }

    /// Gets the diagnostics from the analysis.
    pub fn diagnostics(&self) -> &Arc<[Diagnostic]> {
        &self.diagnostics
    }
}

/// Represents a node in a document graph.
#[derive(Debug)]
pub struct DocumentGraphNode {
    /// The URI of the document.
    uri: Arc<Url>,
    /// The current incremental change to the document.
    ///
    /// If `None`, there is no pending incremental change applied to the node.
    change: Option<IncrementalChange>,
    /// The parse state of the document.
    parse_state: ParseState,
    /// The analysis of the document.
    ///
    /// If `None`, an analysis does not exist for the current state of the node.
    analysis: Option<Analysis>,
}

impl DocumentGraphNode {
    /// Constructs a new unparsed document graph node.
    pub fn new(uri: Arc<Url>) -> Self {
        Self {
            uri,
            change: None,
            parse_state: ParseState::NotParsed,
            analysis: None,
        }
    }

    /// Gets the URI of the document node.
    pub fn uri(&self) -> &Arc<Url> {
        &self.uri
    }

    /// Notifies the document node that there's been an incremental change.
    pub fn notify_incremental_change(&mut self, change: IncrementalChange) {
        info!("document `{uri}` has incrementally changed", uri = self.uri);

        // Clear the analysis as there has been a change
        self.analysis = None;

        // Attempt to merge the edits of the change
        if let Some(IncrementalChange {
            version: existing_version,
            start: existing_start,
            edits: existing_edits,
        }) = &mut self.change
        {
            let IncrementalChange {
                version,
                start,
                edits,
            } = change;
            *existing_version = version;
            if start.is_some() {
                *existing_start = start;
                *existing_edits = edits;
            } else {
                existing_edits.extend(edits);
            }
        } else {
            self.change = Some(change)
        }
    }

    /// Notifies the document node that the document has fully changed.
    pub fn notify_change(&mut self, discard_pending: bool) {
        info!("document `{uri}` has changed", uri = self.uri);

        // Clear the analysis as there has been a change
        self.analysis = None;

        if !matches!(self.parse_state, ParseState::Parsed {
            version: Some(_),
            ..
        }) || discard_pending
        {
            self.parse_state = ParseState::NotParsed;
            self.change = None;
        }
    }

    /// Gets the parse state of the document node.
    pub fn parse_state(&self) -> &ParseState {
        &self.parse_state
    }

    /// Marks the parse as completed.
    pub fn parse_completed(&mut self, state: ParseState) {
        assert!(!matches!(state, ParseState::NotParsed));
        self.parse_state = state;

        // Clear any document change
        self.change = None;
    }

    /// Gets the analysis of the document node.
    pub fn analysis(&self) -> Option<&Analysis> {
        self.analysis.as_ref()
    }

    /// Marks the analysis as completed.
    pub fn analysis_completed(&mut self, analysis: Analysis) {
        self.analysis = Some(analysis);
    }

    /// Marks the document node for reanalysis.
    ///
    /// This may occur when a dependency has changed.
    pub fn reanalyze(&mut self) {
        self.analysis = None;
    }

    /// Gets the AST document of the node.
    ///
    /// Returns `None` if the document was not parsed.
    pub fn document(&self) -> Option<wdl_ast::Document> {
        if let ParseState::Parsed { root, .. } = &self.parse_state {
            return Some(
                wdl_ast::Document::cast(SyntaxNode::new_root(root.clone()))
                    .expect("node should cast"),
            );
        }

        None
    }

    /// Determines if the document needs to be parsed.
    pub fn needs_parse(&self) -> bool {
        self.change.is_some() || matches!(self.parse_state, ParseState::NotParsed)
    }

    /// Parses the document.
    ///
    /// If a parse is not necessary, the current parse state is returned.
    ///
    /// Otherwise, the new parse state is returned.
    pub fn parse(
        &self,
        tokio: &Handle,
        client: &Client,
        validator: &mut Validator,
    ) -> Result<ParseState> {
        if !self.needs_parse() {
            return Ok(self.parse_state.clone());
        }

        // First attempt an incremental parse
        if let Some(state) = self.incremental_parse() {
            return Ok(state);
        }

        // Otherwise, fall back to a full parse.
        self.full_parse(tokio, client, validator)
    }

    /// Performs an incremental parse of the document.
    ///
    /// Returns an error with the given change if the document needs a full
    /// parse.
    fn incremental_parse(&self) -> Option<ParseState> {
        match &self.change {
            None | Some(IncrementalChange { start: Some(_), .. }) => None,
            Some(IncrementalChange { start: None, .. }) => {
                // TODO: implement incremental parsing
                // For each edit:
                //   * determine if the edit is to a token; if so, replace it in the tree
                //   * otherwise, find a reparsable ancestor for the covering element and ask it
                //     to reparse; if one is found, reparse and replace the node
                //   * if a reparsable node can't be found, return an error to trigger a full
                //     reparse
                //   * incrementally update the parse diagnostics depending on the result
                None
            }
        }
    }

    /// Performs a full parse of the node.
    fn full_parse(
        &self,
        tokio: &Handle,
        client: &Client,
        validator: &mut Validator,
    ) -> Result<ParseState> {
        let (version, source, lines) = match &self.change {
            None => {
                // Fetch the source
                let result = match self.uri.to_file_path() {
                    Ok(path) => fs::read_to_string(path).map_err(Into::into),
                    Err(_) => match self.uri.scheme() {
                        "https" | "http" => Self::download_source(tokio, client, &self.uri),
                        scheme => Err(anyhow!("unsupported URI scheme `{scheme}`")),
                    },
                };

                match result {
                    Ok(source) => {
                        let lines = Arc::new(LineIndex::new(&source));
                        (None, source, lines)
                    }
                    Err(e) => return Ok(ParseState::Error(e.into())),
                }
            }
            Some(IncrementalChange {
                version,
                start,
                edits,
            }) => {
                // The document has been edited; if there is start source, apply the edits to it
                let (mut source, mut lines) = if let Some(start) = start {
                    let source = start.clone();
                    let lines = Arc::new(LineIndex::new(&source));
                    (source, lines)
                } else {
                    // Otherwise, apply the edits to the last parse
                    match &self.parse_state {
                        ParseState::Parsed { root, lines, .. } => (
                            SyntaxNode::new_root(root.clone()).text().to_string(),
                            lines.clone(),
                        ),
                        _ => panic!(
                            "cannot apply edits to a document that was not previously parsed"
                        ),
                    }
                };

                // We keep track of the last line we've processed so we only rebuild the line
                // index when there is a change that crosses a line
                let mut last_line = !0u32;
                for edit in edits {
                    let range = edit.range();
                    if last_line <= range.end.line {
                        // Only rebuild the line index if the edit has changed lines
                        lines = Arc::new(LineIndex::new(&source));
                    }

                    last_line = range.start.line;
                    edit.apply(&mut source, &lines)?;
                }

                if !edits.is_empty() {
                    // Rebuild the line index after all edits have been applied
                    lines = Arc::new(LineIndex::new(&source));
                }

                (Some(*version), source, lines)
            }
        };

        // Reparse from the source
        let start = Instant::now();
        let (document, diagnostics) = wdl_ast::Document::parse(&source);
        info!(
            "parsing of `{uri}` completed in {elapsed:?}",
            uri = self.uri,
            elapsed = start.elapsed()
        );

        let diagnostics = if diagnostics.is_empty() {
            Diagnostics::Validation(
                validator
                    .validate(&document)
                    .err()
                    .unwrap_or_default()
                    .into(),
            )
        } else {
            Diagnostics::Parse(diagnostics.into())
        };

        Ok(ParseState::Parsed {
            version,
            root: document.syntax().green().into(),
            lines,
            diagnostics,
        })
    }

    /// Downloads the source of a `http` or `https` scheme URI.
    ///
    /// This makes a request on the provided tokio runtime to download the
    /// source.
    fn download_source(tokio: &Handle, client: &Client, uri: &Url) -> Result<String> {
        /// The timeout for downloading the source, in seconds.
        const TIMEOUT_IN_SECS: u64 = 30;

        info!("downloading source from `{uri}`");

        tokio.block_on(async {
            let resp = client
                .get(uri.as_str())
                .timeout(Duration::from_secs(TIMEOUT_IN_SECS))
                .send()
                .await?;

            let code = resp.status();
            if !code.is_success() {
                bail!("server returned HTTP status {code}");
            }

            resp.text().await.context("failed to read response body")
        })
    }
}

/// Represents a document graph.
#[derive(Debug, Default)]
pub struct DocumentGraph {
    /// The inner directional graph.
    ///
    /// Edges in the graph denote inverse dependency relationships (i.e. "is
    /// depended upon by").
    inner: StableDiGraph<DocumentGraphNode, ()>,
    /// Map from document URI to graph node index.
    indexes: IndexMap<Arc<Url>, NodeIndex>,
    /// The current set of rooted nodes in the graph.
    ///
    /// Rooted nodes are those that were explicitly added to the analyzer.
    ///
    /// A rooted node is one that will not be collected even if the node has no
    /// outgoing edges (i.e. is not depended upon by any other file).
    roots: IndexSet<NodeIndex>,
    /// Represents dependency edges that, if they were added to the document
    /// graph, would form a cycle.
    ///
    /// The first in the pair is the dependant node and the second is the
    /// depended node.
    ///
    /// This is used to break import cycles; when analyzing the document, if the
    /// import relationship exists in this set, a diagnostic will be added and
    /// the import otherwise ignored.
    cycles: HashSet<(NodeIndex, NodeIndex)>,
}

impl DocumentGraph {
    /// Add a node to the document graph.
    pub fn add_node(&mut self, uri: Url, rooted: bool) -> NodeIndex {
        let index = if let Some(index) = self.indexes.get(&uri) {
            *index
        } else {
            debug!("inserting `{uri}` into the document graph");
            let uri = Arc::new(uri);
            let index = self.inner.add_node(DocumentGraphNode::new(uri.clone()));
            self.indexes.insert(uri, index);
            index
        };

        if rooted {
            self.roots.insert(index);
        }

        index
    }

    /// Removes a root from the document graph.
    ///
    /// Note that this does not remove any nodes, only removes the document from
    /// the set of rooted nodes.
    ///
    /// If the node has no outgoing edges, it will be removed on the next
    /// garbage collection.
    pub fn remove_root(&mut self, uri: &Url) {
        let base = match uri.to_file_path() {
            Ok(base) => base,
            Err(_) => return,
        };

        // As the URI might be a directory containing WDL files, look for prefixed files
        let mut removed = Vec::new();
        for (uri, index) in &self.indexes {
            let path = match uri.to_file_path() {
                Ok(path) => path,
                Err(_) => continue,
            };

            if path.starts_with(&base) {
                removed.push(*index);
            }
        }

        for index in removed {
            let node = &mut self.inner[index];

            // We don't actually remove nodes from the graph, just remove it as a root.
            // If the node has no outgoing edges, it will be collected in the next GC.
            if !self.roots.swap_remove(&index) {
                debug!(
                    "document `{uri}` is no longer rooted in the graph",
                    uri = node.uri
                );
            }

            node.parse_state = ParseState::NotParsed;
            node.analysis = None;
            node.change = None;

            // Do a BFS traversal to trigger re-analysis in dependent documents
            self.bfs_mut(index, |graph, dependent: NodeIndex| {
                let node = graph.get_mut(dependent);
                debug!("document `{uri}` needs to be reanalyzed", uri = node.uri);
                node.analysis = None;
            });
        }
    }

    /// Determines if the given node is rooted.
    pub fn is_rooted(&self, index: NodeIndex) -> bool {
        self.roots.contains(&index)
    }

    /// Gets the rooted nodes in the graph.
    pub fn roots(&self) -> &IndexSet<NodeIndex> {
        &self.roots
    }

    /// Determines if the given document node should be included in analysis
    /// results.
    pub fn include_result(&self, index: NodeIndex) -> bool {
        // Only consider rooted or parsed nodes that have been analyzed
        let node = self.get(index);
        node.analysis().is_some()
            && (self.roots.contains(&index)
                || matches!(node.parse_state(), ParseState::Parsed { .. }))
    }

    /// Gets a node from the graph.
    pub fn get(&self, index: NodeIndex) -> &DocumentGraphNode {
        &self.inner[index]
    }

    /// Gets a mutable node from the graph.
    pub fn get_mut(&mut self, index: NodeIndex) -> &mut DocumentGraphNode {
        &mut self.inner[index]
    }

    /// Gets the node index for the given document URI.
    ///
    /// Returns `None` if the document is not in the graph.
    pub fn get_index(&self, uri: &Url) -> Option<NodeIndex> {
        self.indexes.get(uri).copied()
    }

    /// Performs a breadth-first traversal of the graph starting at the given
    /// node.
    ///
    /// Mutations to the document nodes are permitted.
    pub fn bfs_mut(&mut self, index: NodeIndex, mut cb: impl FnMut(&mut Self, NodeIndex)) {
        let mut bfs = Bfs::new(&self.inner, index);
        while let Some(node) = bfs.next(&self.inner) {
            cb(self, node);
        }
    }

    /// Gets the direct dependencies of a node.
    pub fn dependencies(&self, index: NodeIndex) -> impl Iterator<Item = NodeIndex> + '_ {
        self.inner
            .edges_directed(index, Direction::Incoming)
            .map(|e| e.source())
    }

    /// Removes all dependency edges from the given node.
    pub fn remove_dependency_edges(&mut self, index: NodeIndex) {
        // Retain all edges where the target isn't the given node (i.e. an incoming
        // edge)
        self.inner.retain_edges(|g, e| {
            let (_, target) = g.edge_endpoints(e).expect("edge should be valid");
            target != index
        });
    }

    /// Adds a dependency edge from one document to another.
    ///
    /// If a dependency edge already exists, this is a no-op.
    pub fn add_dependency_edge(&mut self, from: NodeIndex, to: NodeIndex, space: &mut DfsSpace) {
        // Check to see if there is already a path between the nodes; if so, there's a
        // cycle
        if has_path_connecting(&self.inner, from, to, Some(space)) {
            // Adding the edge would cause a cycle, so record the cycle instead
            debug!(
                "an import cycle was detected between `{from}` and `{to}`",
                from = self.inner[from].uri,
                to = self.inner[to].uri
            );
            self.cycles.insert((from, to));
        } else if !self.inner.contains_edge(to, from) {
            debug!(
                "adding dependency edge from `{from}` to `{to}`",
                from = self.inner[from].uri,
                to = self.inner[to].uri
            );

            // Note that we store inverse dependency edges in the graph, so the relationship
            // is reversed
            self.inner.add_edge(to, from, ());
        }
    }

    /// Determines if there is a cycle between the given nodes.
    pub fn contains_cycle(&self, from: NodeIndex, to: NodeIndex) -> bool {
        self.cycles.contains(&(from, to))
    }

    /// Creates a subgraph of this graph for the given nodes to include.
    pub fn subgraph(&self, nodes: &IndexSet<NodeIndex>) -> StableDiGraph<NodeIndex, ()> {
        self.inner
            .filter_map(|i, _| nodes.contains(&i).then_some(i), |_, _| Some(()))
    }

    /// Performs a garbage collection on the graph.
    ///
    /// This removes any non-rooted nodes that have no outgoing edges (i.e. are
    /// not depended upon by another document).
    pub fn gc(&mut self) {
        let mut collected = HashSet::new();
        for node in self.inner.node_indices() {
            if self.roots.contains(&node) {
                continue;
            }

            if self
                .inner
                .edges_directed(node, Direction::Outgoing)
                .next()
                .is_none()
            {
                debug!(
                    "removing document `{uri}` from the graph",
                    uri = self.inner[node].uri
                );
                collected.insert(node);
            }
        }

        if collected.is_empty() {
            return;
        }

        for node in &collected {
            self.inner.remove_node(*node);
        }

        self.indexes.retain(|_, index| !collected.contains(index));

        self.cycles
            .retain(|(from, to)| !collected.contains(from) && !collected.contains(to));
    }
}
