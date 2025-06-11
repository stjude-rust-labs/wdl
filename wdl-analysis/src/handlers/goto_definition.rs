//! Implements goto definition functionality.

use std::sync::Arc;

use anyhow::Context;
use anyhow::Result;
use anyhow::anyhow;
use line_index::LineIndex;
use line_index::WideEncoding;
use lsp_types::GotoDefinitionResponse;
use lsp_types::Location;
use lsp_types::Position;
use rowan::TextSize;
use tracing::debug;
use url::Url;
use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::Span;
use wdl_ast::SyntaxKind;
use wdl_ast::SyntaxNode;
use wdl_ast::SyntaxToken;
use wdl_ast::TreeNode;
use wdl_ast::TreeToken;
use wdl_ast::v1;

use crate::DiagnosticsConfig;
use crate::SourcePosition;
use crate::SourcePositionEncoding;
use crate::diagnostics;
use crate::document::Document;
use crate::document::ScopeRef;
use crate::graph::DocumentGraph;
use crate::graph::ParseState;
use crate::types::v1::EvaluationContext;
use crate::types::v1::ExprTypeEvaluator;

/// Handler for goto definition requests.
#[derive(Debug)]
pub struct GotoDefinitionHandler;

impl Default for GotoDefinitionHandler {
    fn default() -> Self {
        Self
    }
}

impl GotoDefinitionHandler {
    /// Creates a new goto definition handler
    pub fn new() -> Self {
        Self
    }

    /// Finds the definition location for an identifier at the given position.
    ///
    /// Searches the document and its imports for the definition of the
    /// identifier at the specified position, returning the location if
    /// found.
    ///
    /// * If a definition is found for the identifier then a
    ///   [`GotoDefinitionResponse`] containing the URI and range is returned
    ///   wrapped in [`Some`].
    ///
    /// * Else, [`None`] is returned.
    pub fn goto_definition(
        &self,
        graph: &DocumentGraph,
        document_uri: Url,
        position: SourcePosition,
        encoding: SourcePositionEncoding,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let index = graph
            .get_index(&document_uri)
            .ok_or_else(|| anyhow!("document not found in graph"))?;

        let (root, lines, analysis_doc) = {
            let node = graph.get(index);

            let (root, lines) = match node.parse_state() {
                ParseState::Parsed { lines, root, .. } => {
                    (SyntaxNode::new_root(root.clone()), lines.clone())
                }
                _ => return Err(anyhow!("document not yet parsed: {}", document_uri)),
            };

            let analysis_doc = match node.document() {
                Some(doc) => doc.clone(),
                None => {
                    return Err(anyhow!(
                        "document analysis data not available for {}",
                        document_uri
                    ));
                }
            };

            (root, lines, analysis_doc)
        };

        let offset = position_to_offset(&lines, position, encoding)?;
        let token = match find_identifier_token_at_offset(&root, offset) {
            Some(tok) => tok,
            None => return Err(anyhow!("no identifier found at position")),
        };

        let ident_text = token.text();
        let parent_node = token
            .parent()
            .ok_or_else(|| anyhow!("identifier token has no parent"))?;

        // Context based resolution
        if let Some(location) = self.resolve_by_context(
            &parent_node,
            &token,
            &analysis_doc,
            &document_uri,
            &lines,
            graph,
        )? {
            return Ok(Some(location));
        }

        // Scope based resolution
        if let Some(scope_ref) = analysis_doc.find_scope_by_position(token.span().start()) {
            if let Some(name_def) = scope_ref.lookup(ident_text) {
                return Ok(Some(location(&document_uri, name_def.span(), &lines)?));
            }
        }

        // Global resolution
        self.resolve_global_identifier(&analysis_doc, ident_text, &document_uri, &lines, graph)
    }

    /// Resolves identifier definition based on their parent node's syntax kind
    fn resolve_by_context(
        &self,
        parent_node: &SyntaxNode,
        token: &SyntaxToken,
        analysis_doc: &Document,
        document_uri: &Url,
        lines: &Arc<LineIndex>,
        graph: &DocumentGraph,
    ) -> Result<Option<GotoDefinitionResponse>> {
        match parent_node.kind() {
            SyntaxKind::TypeRefNode => {
                self.resolve_type_reference(analysis_doc, token.text(), document_uri, lines, graph)
            }

            SyntaxKind::CallTargetNode => self.resolve_call_target(
                parent_node,
                token,
                analysis_doc,
                document_uri,
                lines,
                graph,
            ),
            SyntaxKind::ImportStatementNode => {
                self.resolve_import_namespace(parent_node, token, document_uri, lines)
            }

            SyntaxKind::AccessExprNode => self.resolve_access_expression(
                parent_node,
                token,
                analysis_doc,
                document_uri,
                lines,
                graph,
            ),

            // TODO:
            SyntaxKind::BoundDeclNode
            | SyntaxKind::UnboundDeclNode
            | SyntaxKind::ScatterStatementNode
            | SyntaxKind::ImportAliasNode
            | SyntaxKind::StructDefinitionNode
            | SyntaxKind::TaskDefinitionNode
            | SyntaxKind::WorkflowDefinitionNode => {
                debug!("NOT YET IMPLEMENTED: {kind:?}", kind = parent_node.kind());
                Ok(None)
            }

            // handled by scope resolution
            SyntaxKind::NameRefExprNode => Ok(None),
            _ => Ok(None),
        }
    }

    /// Resolves type references to their definition locations.
    ///
    /// Searches for struct definitions in the current document and imported
    /// namespaces
    fn resolve_type_reference(
        &self,
        analysis_doc: &Document,
        ident_text: &str,
        document_uri: &Url,
        lines: &Arc<LineIndex>,
        graph: &DocumentGraph,
    ) -> Result<Option<GotoDefinitionResponse>> {
        // NOTE: Local structs
        if let Some(struct_info) = analysis_doc.struct_by_name(ident_text) {
            let (uri, def_lines) = if let Some(ns_name) = struct_info.namespace() {
                let ns = analysis_doc.namespace(ns_name).unwrap();
                let imported_lines = graph
                    .get(graph.get_index(ns.source()).unwrap())
                    .parse_state()
                    .lines()
                    .unwrap()
                    .clone();

                (ns.source().as_ref(), imported_lines)
            } else {
                (document_uri, lines.clone())
            };
            return Ok(Some(location(uri, struct_info.name_span(), &def_lines)?));
        };

        // NOTE: Imported structs
        for (_ns_name_str, ns_info) in analysis_doc.namespaces() {
            if let Some(imported_doc) = graph
                .get(graph.get_index(ns_info.source()).unwrap())
                .document()
            {
                if let Some(struct_info) = imported_doc.struct_by_name(ident_text) {
                    let imported_lines = graph
                        .get(graph.get_index(ns_info.source()).unwrap())
                        .parse_state()
                        .lines()
                        .unwrap()
                        .clone();

                    return Ok(Some(location(
                        ns_info.source(),
                        struct_info.name_span(),
                        &imported_lines,
                    )?));
                }
            }
        }
        Err(anyhow!(
            "could not resolve type reference for: {}",
            ident_text
        ))
    }

    /// Resolves call targets to their definition locations.
    ///
    /// Handles both local and namespaced function calls, resolving them to task
    /// and workflow definition in the current document or imported
    /// namespaces.
    fn resolve_call_target(
        &self,
        parent_node: &SyntaxNode,
        token: &SyntaxToken,
        analysis_doc: &Document,
        document_uri: &Url,
        lines: &Arc<LineIndex>,
        graph: &DocumentGraph,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let ident_text = token.text();
        let target = wdl_ast::v1::CallTarget::cast(parent_node.clone()).unwrap();
        let target_names: Vec<_> = target.names().collect();
        let is_callee_name_clicked = target_names.last().is_some_and(|n| n.text() == ident_text);

        if is_callee_name_clicked {
            let callee_name_str = token.text();

            // NOTE: Namespaced (foo.bar)
            if target_names.len() > 1 {
                let namespaced_name_str = target_names.first().unwrap().text();
                if let Some(ns_info) = analysis_doc.namespace(namespaced_name_str) {
                    if let Some(imported_doc) = graph
                        .get(graph.get_index(ns_info.source()).unwrap())
                        .document()
                    {
                        let imported_lines = graph
                            .get(graph.get_index(ns_info.source()).unwrap())
                            .parse_state()
                            .lines()
                            .unwrap()
                            .clone();

                        if let Some(task_def) = imported_doc.task_by_name(callee_name_str) {
                            return Ok(Some(location(
                                ns_info.source(),
                                task_def.name_span(),
                                &imported_lines,
                            )?));
                        }

                        if let Some(wf_def) = imported_doc
                            .workflow()
                            .filter(|w| w.name() == callee_name_str)
                        {
                            return Ok(Some(location(
                                ns_info.source(),
                                wf_def.name_span(),
                                &imported_lines,
                            )?));
                        }
                    }
                }
            } else {
                // NOTE: Local calls
                if let Some(task_def) = analysis_doc.task_by_name(callee_name_str) {
                    return Ok(Some(location(document_uri, task_def.name_span(), lines)?));
                }

                if let Some(wf_def) = analysis_doc
                    .workflow()
                    .filter(|w| w.name() == callee_name_str)
                {
                    return Ok(Some(location(document_uri, wf_def.name_span(), lines)?));
                }
            }
        } else if let Some(ns_info) = analysis_doc.namespace(token.text()) {
            return Ok(Some(location(document_uri, ns_info.span(), lines)?));
        }

        Ok(None)
    }

    /// Resolves import namespace identifier to their definition locations.
    fn resolve_import_namespace(
        &self,
        parent_node: &SyntaxNode,
        token: &SyntaxToken,
        document_uri: &Url,
        lines: &Arc<LineIndex>,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let import_stmt = wdl_ast::v1::ImportStatement::cast(parent_node.clone()).unwrap();
        let ident_text = token.text();

        if import_stmt
            .explicit_namespace()
            .is_some_and(|ns_ident| ns_ident.text() == ident_text)
        {
            return Ok(Some(location(document_uri, token.span(), lines)?));
        }

        Ok(None)
    }

    /// Searches for global definitions(structs, tasks, workflows) in the
    /// current document and all imported namespaces
    fn resolve_global_identifier(
        &self,
        analysis_doc: &Document,
        ident_text: &str,
        document_uri: &Url,
        lines: &Arc<LineIndex>,
        graph: &DocumentGraph,
    ) -> Result<Option<GotoDefinitionResponse>> {
        if let Some(location) =
            find_global_definition_in_doc(analysis_doc, ident_text, document_uri, lines)?
        {
            return Ok(Some(location));
        }

        for (_, ns_info) in analysis_doc.namespaces() {
            if let Some(imported_doc) = graph
                .get(graph.get_index(ns_info.source()).unwrap())
                .document()
            {
                let imported_lines = graph
                    .get(graph.get_index(ns_info.source()).unwrap())
                    .parse_state()
                    .lines()
                    .unwrap()
                    .clone();

                if let Some(location) = find_global_definition_in_doc(
                    imported_doc,
                    ident_text,
                    ns_info.source().as_ref(),
                    &imported_lines,
                )? {
                    return Ok(Some(location));
                }
            }
        }

        Ok(None)
    }

    /// Resolves access expressions to their member definition locations.
    ///
    /// Evaluates the target expression's type and resolves member access to the
    /// appropriate def. location.
    ///
    /// # Supports:
    /// - Struct member access (`person.name`)
    /// - Call output access (`call_result.output`)
    /// - TODO: Arrays
    fn resolve_access_expression(
        &self,
        parent_node: &SyntaxNode,
        token: &SyntaxToken,
        analysis_doc: &Document,
        document_uri: &Url,
        lines: &Arc<LineIndex>,
        graph: &DocumentGraph,
    ) -> Result<Option<GotoDefinitionResponse>> {
        let access_expr = wdl_ast::v1::AccessExpr::cast(parent_node.clone()).unwrap();
        let (target_expr, member_ident) = access_expr.operands();

        if member_ident.span() != token.span() {
            return Ok(None);
        }

        let scope = analysis_doc
            .find_scope_by_position(parent_node.span().start())
            .ok_or_else(|| anyhow!("could not find scope for access expression"))?;

        let mut ctx = GotoDefEvalContext {
            scope,
            document: analysis_doc,
        };
        let mut evaluator = ExprTypeEvaluator::new(&mut ctx);
        let target_type = evaluator
            .evaluate_expr(&target_expr)
            .unwrap_or(crate::types::Type::Union);

        if let Some(struct_ty) = target_type.as_struct() {
            let struct_def = analysis_doc
                .structs()
                .find(|(_, s)| {
                    if let Some(s_ty) = s.ty() {
                        if let Some(s_struct_ty) = s_ty.as_struct() {
                            return s_struct_ty.name() == struct_ty.name();
                        }
                    }
                    s.name() == struct_ty.name().as_str()
                })
                .map(|(_, s)| s)
                .ok_or_else(|| anyhow!("struct definition not found for {}", struct_ty.name()))?;

            let (uri, def_lines) = if let Some(ns_name) = struct_def.namespace() {
                let ns = analysis_doc.namespace(ns_name).unwrap();
                let imported_node = graph.get(graph.get_index(ns.source()).unwrap());
                let lines = imported_node.parse_state().lines().unwrap().clone();
                (ns.source().as_ref(), lines)
            } else {
                (document_uri, lines.clone())
            };

            let struct_node =
                v1::StructDefinition::cast(SyntaxNode::new_root(struct_def.node().clone()))
                    .expect("should cast to struct definition");

            if let Some(member) = struct_node
                .members()
                .find(|m| m.name().text() == member_ident.text())
            {
                let member_span = member.name().span();
                let span = Span::new(member_span.start() + struct_def.offset(), member_span.len());
                return Ok(Some(location(uri, span, &def_lines)?));
            }
        }

        if let Some(call_ty) = target_type.as_call() {
            if let Some(output) = call_ty.outputs().get(member_ident.text()) {
                let (uri, callee_lines) = if let Some(ns_name) = call_ty.namespace() {
                    let ns = analysis_doc.namespace(ns_name).unwrap();
                    let imported_node = graph.get(graph.get_index(ns.source()).unwrap());
                    let lines = imported_node.parse_state().lines().unwrap().clone();
                    (ns.source().as_ref(), lines.clone())
                } else {
                    (document_uri, lines.clone())
                };

                return Ok(Some(location(uri, output.name_span(), &callee_lines)?));
            }
        }

        Ok(None)
    }
}

/// Finds an identifier token at the specified `TextSize` offset in the concrete
/// syntax tree.
fn find_identifier_token_at_offset(node: &SyntaxNode, offset: TextSize) -> Option<SyntaxToken> {
    node.token_at_offset(offset)
        .find(|t| t.kind() == SyntaxKind::Ident)
}

/// Converts a text size offset to LSP position.
fn position(index: &LineIndex, offset: TextSize) -> Result<Position> {
    let line_col = index.line_col(offset);
    let line_col = index
        .to_wide(WideEncoding::Utf16, line_col)
        .with_context(|| {
            format!(
                "invalid line column: {line}:{column}",
                line = line_col.line,
                column = line_col.col
            )
        })?;

    Ok(Position::new(line_col.line, line_col.col))
}

/// Converts a `Span` to an LSP location
fn location(uri: &Url, span: Span, lines: &Arc<LineIndex>) -> Result<GotoDefinitionResponse> {
    let start_offset = TextSize::from(span.start() as u32);
    let end_offset = TextSize::from(span.end() as u32);
    let range = lsp_types::Range {
        start: position(lines, start_offset)?,
        end: position(lines, end_offset)?,
    };

    Ok(GotoDefinitionResponse::Scalar(Location::new(
        uri.clone(),
        range,
    )))
}

/// Finds global structs, tasks and workflow definition in a document
fn find_global_definition_in_doc(
    analysis_doc: &Document,
    ident_text: &str,
    document_uri: &Url,
    lines: &Arc<LineIndex>,
) -> Result<Option<GotoDefinitionResponse>> {
    if let Some(s) = analysis_doc.struct_by_name(ident_text) {
        return Ok(Some(location(document_uri, s.name_span(), lines)?));
    }
    if let Some(t) = analysis_doc.task_by_name(ident_text) {
        return Ok(Some(location(document_uri, t.name_span(), lines)?));
    }
    if let Some(w) = analysis_doc
        .workflow()
        .filter(|w_def| w_def.name() == ident_text)
    {
        return Ok(Some(location(document_uri, w.name_span(), lines)?));
    }

    Ok(None)
}

/// Converts a source postion to a text offset based on the specified encoding.
fn position_to_offset(
    lines: &Arc<LineIndex>,
    position: SourcePosition,
    encoding: SourcePositionEncoding,
) -> Result<TextSize> {
    let line_col = match encoding {
        SourcePositionEncoding::UTF8 => line_index::LineCol {
            line: position.line,
            col: position.character,
        },
        SourcePositionEncoding::UTF16 => lines
            .to_utf8(
                line_index::WideEncoding::Utf16,
                line_index::WideLineCol {
                    line: position.line,
                    col: position.character,
                },
            )
            .ok_or_else(|| anyhow!("invalid utf-16 position: {position:?}"))?,
    };

    lines
        .offset(line_col)
        .ok_or_else(|| anyhow!("line_col is invalid"))
}

/// Context for evaluating expressions during goto definition resolution.
struct GotoDefEvalContext<'a> {
    /// The scope reference containing the variable and name bindings at the
    /// current position
    scope: ScopeRef<'a>,

    /// The document being analyzed.
    document: &'a Document,
}

impl EvaluationContext for GotoDefEvalContext<'_> {
    fn version(&self) -> wdl_ast::SupportedVersion {
        self.document
            .version()
            .expect("document should have a version")
    }

    fn resolve_name(&self, name: &str, _span: Span) -> Option<crate::types::Type> {
        self.scope.lookup(name).map(|n| n.ty().clone())
    }

    fn resolve_type_name(
        &mut self,
        name: &str,
        span: Span,
    ) -> std::result::Result<crate::types::Type, wdl_ast::Diagnostic> {
        if let Some(s) = self.document.struct_by_name(name) {
            if let Some(ty) = s.ty() {
                return Ok(ty.clone());
            }
        }
        Err(diagnostics::unknown_type(name, span))
    }

    fn task(&self) -> Option<&crate::document::Task> {
        None
    }

    fn diagnostics_config(&self) -> DiagnosticsConfig {
        DiagnosticsConfig::default()
    }

    fn add_diagnostic(&mut self, _: wdl_ast::Diagnostic) {}
}
