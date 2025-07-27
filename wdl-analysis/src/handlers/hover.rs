//! Handlers for hover requests.
//!
//! This module implements the LSP `textDocument/hover` functionality for WDL
//! files.
//!
//! See: https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_hover

use anyhow::Result;
use anyhow::bail;
use lsp_types::Hover;
use lsp_types::HoverContents;
use lsp_types::MarkupContent;
use lsp_types::MarkupKind;
use tracing::debug;
use url::Url;
use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::SyntaxKind;
use wdl_ast::SyntaxNode;
use wdl_ast::SyntaxToken;
use wdl_ast::TreeNode;
use wdl_ast::TreeToken;
use wdl_ast::v1::AccessExpr;
use wdl_ast::v1::CallExpr;
use wdl_ast::v1::CallTarget;

use crate::Document;
use crate::SourcePosition;
use crate::SourcePositionEncoding;
use crate::graph::DocumentGraph;
use crate::graph::ParseState;
use crate::handlers::TypeEvalContext;
use crate::handlers::common::find_identifier_token_at_offset;
use crate::handlers::common::location_from_span;
use crate::handlers::common::position_to_offset;
use crate::handlers::common::provide_struct_documentation;
use crate::handlers::common::provide_task_documentation;
use crate::handlers::common::provide_workflow_documentation;
use crate::stdlib::Function;
use crate::stdlib::STDLIB;
use crate::stdlib::TypeParameters;
use crate::types::CompoundType;
use crate::types::Type;
use crate::types::v1::ExprTypeEvaluator;

/// Handles a hover request.
///
/// Analyzes the context at the specified position and generates appropriate
/// hover information.
///
/// Provides hover information by:
/// 1. Attempting to resolve the symbol based on its CST context.
/// 2. Looking up the symbol in the current scope.
/// 3. Checking for global definitions (tasks, workflows and structs) across the
///    document and its imports.
pub fn hover(
    graph: &DocumentGraph,
    document_uri: &Url,
    position: SourcePosition,
    encoding: SourcePositionEncoding,
) -> Result<Option<Hover>> {
    let Some(index) = graph.get_index(document_uri) else {
        bail!("document `{document_uri}` not found in graph")
    };
    let node = graph.get(index);
    let (root, lines) = match node.parse_state() {
        ParseState::Parsed { lines, root, .. } => {
            (SyntaxNode::new_root(root.clone()), lines.clone())
        }
        _ => bail!("document `{uri}` has not been parsed", uri = document_uri),
    };

    let Some(document) = node.document() else {
        bail!("document analysis data not available for {}", document_uri);
    };

    let offset = position_to_offset(&lines, position, encoding)?;
    let Some(token) = find_identifier_token_at_offset(&root, offset) else {
        bail!("no identifier found at position");
    };

    let parent_node = token.parent().expect("token has no parent");

    if let Ok(Some(value)) = resolve_hover_content(&parent_node, &token, document, graph) {
        let range = location_from_span(document_uri, token.span(), &lines)?.range;
        Ok(Some(Hover {
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value,
            }),
            range: Some(range),
        }))
    } else {
        Ok(None)
    }
}

/// This function handles the search for hover information by trying
/// various resolution methods.
fn resolve_hover_content(
    parent_node: &SyntaxNode,
    token: &SyntaxToken,
    document: &Document,
    graph: &DocumentGraph,
) -> Result<Option<String>> {
    // Finds hover information based on the CST.
    if let Some(content) = resolve_hover_by_context(parent_node, token, document, graph)? {
        return Ok(Some(content));
    }

    // Finds hover information based on the scope.
    if let Some(scope) = document.find_scope_by_position(token.span().start()) {
        if let Some(name) = scope.lookup(token.text()) {
            let kind = match name.ty() {
                Type::Call(_) => "call",
                _ => "variable",
            };
            return Ok(Some(format!(
                "```wdl\n({kind}) {}: {}\n```",
                token.text(),
                name.ty()
            )));
        }
    }

    // Finds hover information across global definitions.
    if let Some(content) = find_global_hover_in_doc(document, token)? {
        return Ok(Some(content));
    }

    for (_, ns) in document.namespaces() {
        // SAFETY: we know `get_index` will return `Some` as `ns.source` comes from
        // `document.namespaces` which only contains namespaces for documents that
        // are guaranteed to be present in the graph.
        let node = graph.get(graph.get_index(ns.source()).unwrap());
        let Some(imported_doc) = node.document() else {
            continue;
        };
        if let Some(content) = find_global_hover_in_doc(imported_doc, token)? {
            return Ok(Some(content));
        }
    }

    Ok(None)
}

/// Resolves hover information based on the CST of the document.
///
/// This inspects the parent [`SyntaxNode`] of the hovered token to
/// determine the most specific context.
fn resolve_hover_by_context(
    parent_node: &SyntaxNode,
    token: &SyntaxToken,
    document: &Document,
    graph: &DocumentGraph,
) -> Result<Option<String>> {
    match parent_node.kind() {
        SyntaxKind::TypeRefNode | SyntaxKind::LiteralStructNode => {
            if let Some(s) = document.struct_by_name(token.text()) {
                let root = if let Some(ns_name) = s.namespace() {
                    // SAFETY: we just found a struct with this namespace name and the document
                    // guarantees that `document.namespaces` contains a corresponding entry for
                    // `ns_name`.
                    let ns = document.namespace(ns_name).unwrap();
                    let node = graph.get(graph.get_index(ns.source()).unwrap());
                    node.document().unwrap().root()
                } else {
                    document.root()
                };
                return Ok(provide_struct_documentation(s, &root));
            }
        }
        SyntaxKind::CallTargetNode => {
            let target = CallTarget::cast(parent_node.clone()).unwrap();
            let mut target_names = target.names();

            let (callee_name, ns_name) = match (target_names.next(), target_names.next()) {
                // Namespaced call
                (Some(ns), Some(name)) => {
                    if token.span() == name.span() {
                        (name, Some(ns))
                    } else if token.span() == ns.span() {
                        // namespace identifier hovered
                        if let Some(ns) = document.namespace(token.text()) {
                            return Ok(Some(format!(
                                "```wdl\n(import) {}\n```\nImports from `{}`",
                                token.text(),
                                ns.source()
                            )));
                        }
                        return Ok(None);
                    } else {
                        return Ok(None);
                    }
                }
                // Local call
                (Some(name), None) => {
                    if token.span() == name.span() {
                        (name, None)
                    } else {
                        return Ok(None);
                    }
                }
                _ => return Ok(None),
            };

            let (target_doc, target_root) = if let Some(ns_name) = ns_name {
                let Some(ns) = document.namespace(ns_name.text()) else {
                    return Ok(None);
                };
                let node = graph.get(graph.get_index(ns.source()).unwrap());
                let Some(doc) = node.document() else {
                    return Ok(None);
                };
                (doc, doc.root())
            } else {
                (document, document.root())
            };

            if let Some(task) = target_doc.task_by_name(callee_name.text()) {
                return Ok(provide_task_documentation(task, &target_root));
            }

            if let Some(workflow) = target_doc
                .workflow()
                .filter(|w| w.name() == callee_name.text())
            {
                return Ok(provide_workflow_documentation(workflow, &target_root));
            }
        }
        SyntaxKind::AccessExprNode => {
            let access_expr = AccessExpr::cast(parent_node.clone()).unwrap();
            let (expr, member) = access_expr.operands();
            if member.span() != token.span() {
                return Ok(None);
            }

            let Some(scope) = document.find_scope_by_position(parent_node.span().start()) else {
                return Ok(None);
            };
            let mut ctx = TypeEvalContext { scope, document };
            let mut evaluator = ExprTypeEvaluator::new(&mut ctx);
            let target_type = evaluator
                .evaluate_expr(&expr)
                .unwrap_or(crate::types::Type::Union);

            let member_ty = match target_type {
                Type::Compound(CompoundType::Struct(s), _) => {
                    s.members().get(member.text()).cloned()
                }
                Type::Call(c) => c.outputs().get(member.text()).map(|o| o.ty().clone()),
                Type::Compound(CompoundType::Pair(p), _) => match member.text() {
                    "left" => Some(p.left_type().clone()),
                    "right" => Some(p.right_type().clone()),
                    _ => None,
                },
                _ => None,
            };

            if let Some(ty) = member_ty {
                return Ok(Some(format!(
                    "```wdl\n(property) {}: {}\n```",
                    member.text(),
                    ty
                )));
            }
        }
        SyntaxKind::CallExprNode => {
            let Some(call_expr) = CallExpr::cast(parent_node.clone()) else {
                return Ok(None);
            };

            if call_expr.target().span() != token.span() {
                return Ok(None);
            }

            if let Some(func) = STDLIB.function(call_expr.target().text()) {
                let content = get_function_hover_content(call_expr.target().text(), func);
                return Ok(Some(content));
            }
        }
        _ => debug!("hover is not implemented for {:?}", parent_node.kind()),
    }

    Ok(None)
}

/// Finds hover information for a globally defined symbol within a [`Document`].
fn find_global_hover_in_doc(document: &Document, token: &SyntaxToken) -> Result<Option<String>> {
    if let Some(s) = document.struct_by_name(token.text()) {
        return Ok(provide_struct_documentation(s, &document.root()));
    }
    if let Some(t) = document.task_by_name(token.text()) {
        return Ok(provide_task_documentation(t, &document.root()));
    }
    if let Some(w) = document.workflow().filter(|w| w.name() == token.text()) {
        return Ok(provide_workflow_documentation(w, &document.root()));
    }
    Ok(None)
}

/// Generates markdown content for a standard library function's hover info.
///
/// This includes all overloaded signatures and the documentation from the WDL
/// specification.
fn get_function_hover_content(name: &str, func: &Function) -> String {
    let (detail, docs) = match func {
        Function::Monomorphic(m) => {
            let sig = m.signature();
            let params = TypeParameters::new(sig.type_parameters());
            let detail = format!("```wdl\n{}{}\n```", name, sig.display(&params));
            let docs = sig.definition().unwrap_or("");
            (detail, docs)
        }
        Function::Polymorphic(p) => {
            let detail = p
                .signatures()
                .iter()
                .map(|s| {
                    let params = TypeParameters::new(s.type_parameters());
                    format!("```wdl\n{}{}\n```", name, s.display(&params))
                })
                .collect::<Vec<_>>()
                .join("\n---\n");

            let docs = p
                .signatures()
                .first()
                .and_then(|s| s.definition())
                .unwrap_or("");
            (detail, docs)
        }
    };
    format!("{detail}\n\n{docs}")
}
