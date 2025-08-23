//! Handlers for document symbols.
//!
//! This module implements the LSP "textDocument/documentSymbol" functionality
//! for WDL files. It traverses the AST of a document and creates a hierarchical
//! list of symbols.
//!
//! See: https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_documentSymbol

use anyhow::Result;
use anyhow::bail;
use lsp_types::DocumentSymbol;
use lsp_types::DocumentSymbolResponse;
use lsp_types::SymbolKind;
use url::Url;
use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::v1::BoundDecl;
use wdl_ast::v1::Decl;
use wdl_ast::v1::DocumentItem;
use wdl_ast::v1::ImportStatement;
use wdl_ast::v1::InputSection;
use wdl_ast::v1::OutputSection;
use wdl_ast::v1::StructDefinition;
use wdl_ast::v1::TaskDefinition;
use wdl_ast::v1::UnboundDecl;
use wdl_ast::v1::WorkflowDefinition;

use crate::graph::DocumentGraph;
use crate::graph::ParseState;
use crate::handlers::common;

/// Handles a document symbol request
pub fn document_symbol(graph: &DocumentGraph, uri: &Url) -> Result<Option<DocumentSymbolResponse>> {
    let Some(index) = graph.get_index(uri) else {
        bail!("document `{uri}` not found in graph");
    };

    let node = graph.get(index);
    let lines = match node.parse_state() {
        ParseState::Parsed { lines, .. } => lines.clone(),
        _ => bail!("document `{uri}` has not been parsed", uri = uri),
    };

    let Some(document) = node.document() else {
        bail!("document analysis data not available for {}", uri);
    };

    let mut symbols = Vec::new();
    let Some(ast) = document.root().ast().into_v1() else {
        return Ok(None);
    };

    // NOTE: The reason for using `Ast` here is we don't want to wait for analysis
    // to complete and call `structs`, `tasks` and `workflow` on
    // `analysis::Document`. Doing so will break the outline while the user is
    // typing.
    for item in ast.items() {
        match item {
            DocumentItem::Workflow(workflow) => {
                symbols.push(workflow_to_symbol(uri, &workflow, &lines)?);
            }
            DocumentItem::Task(task) => {
                symbols.push(task_to_symbol(uri, &task, &lines)?);
            }
            DocumentItem::Struct(s) => {
                symbols.push(struct_to_symbol(uri, &s, &lines)?);
            }
            DocumentItem::Import(ns) => {
                symbols.push(import_to_symbol(uri, &ns, &lines)?);
            }
        }
    }

    Ok(Some(DocumentSymbolResponse::Nested(symbols)))
}

/// Converts a [`ImportStatement`] to a [`DocumentSymbol`]
fn import_to_symbol(
    uri: &Url,
    import: &ImportStatement,
    lines: &std::sync::Arc<line_index::LineIndex>,
) -> Result<DocumentSymbol> {
    let (name, selection_span) = import.namespace().unwrap_or_else(|| {
        (
            import.uri().text().unwrap().text().to_string(),
            import.uri().span(),
        )
    });

    Ok(DocumentSymbol {
        name,
        detail: Some(import.uri().text().unwrap().text().to_string()),
        kind: SymbolKind::NAMESPACE,
        range: common::location_from_span(uri, import.span(), lines)?.range,
        selection_range: common::location_from_span(uri, selection_span, lines)?.range,
        children: None,
        tags: None,
        #[allow(deprecated)]
        deprecated: None,
    })
}

/// Converts a [`WorkflowDefinition`] to a [`DocumentSymbol`]
fn workflow_to_symbol(
    uri: &Url,
    workflow: &WorkflowDefinition,
    lines: &std::sync::Arc<line_index::LineIndex>,
) -> Result<DocumentSymbol> {
    let mut children = Vec::new();

    for item in workflow.items() {
        match item {
            wdl_ast::v1::WorkflowItem::Input(section) => {
                children.extend(input_section_to_symbols(uri, &section, lines)?);
            }
            wdl_ast::v1::WorkflowItem::Output(section) => {
                children.extend(output_section_to_symbols(uri, &section, lines)?);
            }
            wdl_ast::v1::WorkflowItem::Declaration(decl) => {
                children.push(bound_decl_to_symbol(uri, &decl, lines)?);
            }
            wdl_ast::v1::WorkflowItem::Call(call) => {
                let name = call
                    .alias()
                    .map(|a| a.name())
                    .unwrap_or_else(|| call.target().names().last().unwrap());
                children.push(DocumentSymbol {
                    name: name.text().to_string(),
                    detail: Some(call.target().text().to_string()),
                    kind: SymbolKind::FUNCTION,
                    range: common::location_from_span(uri, call.span(), lines)?.range,
                    selection_range: common::location_from_span(uri, name.span(), lines)?.range,
                    children: None,
                    tags: None,
                    #[allow(deprecated)]
                    deprecated: None,
                });
            }
            _ => {}
        }
    }

    Ok(DocumentSymbol {
        name: workflow.name().text().to_string(),
        detail: Some("workflow".to_string()),
        kind: SymbolKind::MODULE,
        range: common::location_from_span(uri, workflow.span(), lines)?.range,
        selection_range: common::location_from_span(uri, workflow.name().span(), lines)?.range,
        children: Some(children),
        tags: None,
        #[allow(deprecated)]
        deprecated: None,
    })
}

/// Converts a [`TaskDefinition`] to a [`DocumentSymbol`].
fn task_to_symbol(
    uri: &Url,
    task: &TaskDefinition,
    lines: &std::sync::Arc<line_index::LineIndex>,
) -> Result<DocumentSymbol> {
    let mut children = Vec::new();

    if let Some(input_section) = task.input() {
        children.extend(input_section_to_symbols(uri, &input_section, lines)?);
    }

    if let Some(output_section) = task.output() {
        children.extend(output_section_to_symbols(uri, &output_section, lines)?);
    }

    for decl in task.declarations() {
        children.push(bound_decl_to_symbol(uri, &decl, lines)?);
    }

    Ok(DocumentSymbol {
        name: task.name().text().to_string(),
        detail: Some("task".to_string()),
        kind: SymbolKind::FUNCTION,
        range: common::location_from_span(uri, task.span(), lines)?.range,
        selection_range: common::location_from_span(uri, task.name().span(), lines)?.range,
        children: Some(children),
        tags: None,
        #[allow(deprecated)]
        deprecated: None,
    })
}

/// Converts a [`StructDefinition`] to a [`DocumentSymbol`].
fn struct_to_symbol(
    uri: &Url,
    s: &StructDefinition,
    lines: &std::sync::Arc<line_index::LineIndex>,
) -> Result<DocumentSymbol> {
    let mut children = Vec::new();

    for member in s.members() {
        children.push(unbound_decl_to_symbol(uri, &member, lines)?);
    }

    Ok(DocumentSymbol {
        name: s.name().text().to_string(),
        detail: Some("struct".to_string()),
        kind: SymbolKind::STRUCT,
        range: common::location_from_span(uri, s.span(), lines)?.range,
        selection_range: common::location_from_span(uri, s.name().span(), lines)?.range,
        children: Some(children),
        tags: None,
        #[allow(deprecated)]
        deprecated: None,
    })
}

/// Converts an [`InputSection`] to a [`Vec<DocumentSymbol>`.]
fn input_section_to_symbols(
    uri: &Url,
    section: &InputSection,
    lines: &std::sync::Arc<line_index::LineIndex>,
) -> Result<Vec<DocumentSymbol>> {
    let mut symbols = Vec::new();
    for decl in section.declarations() {
        symbols.push(decl_to_symbol(uri, &decl, lines)?);
    }
    Ok(symbols)
}

/// Converts an [`OutputSection`] to a [`Vec<DocumentSymbol>`.]
fn output_section_to_symbols(
    uri: &Url,
    section: &OutputSection,
    lines: &std::sync::Arc<line_index::LineIndex>,
) -> Result<Vec<DocumentSymbol>> {
    let mut symbols = Vec::new();
    for decl in section.declarations() {
        symbols.push(bound_decl_to_symbol(uri, &decl, lines)?);
    }
    Ok(symbols)
}

/// Converts a [`Decl`] to a [`DocumentSymbol`].
fn decl_to_symbol(
    uri: &Url,
    decl: &Decl,
    lines: &std::sync::Arc<line_index::LineIndex>,
) -> Result<DocumentSymbol> {
    Ok(DocumentSymbol {
        name: decl.name().text().to_string(),
        detail: Some(decl.ty().to_string()),
        kind: SymbolKind::VARIABLE,
        range: common::location_from_span(uri, decl.name().span(), lines)?.range,
        selection_range: common::location_from_span(uri, decl.name().span(), lines)?.range,
        children: None,
        tags: None,
        #[allow(deprecated)]
        deprecated: None,
    })
}

/// Converts an [`UnboundDecl`] to a [`DocumentSymbol`].
fn unbound_decl_to_symbol(
    uri: &Url,
    decl: &UnboundDecl,
    lines: &std::sync::Arc<line_index::LineIndex>,
) -> Result<DocumentSymbol> {
    Ok(DocumentSymbol {
        name: decl.name().text().to_string(),
        detail: Some(decl.ty().to_string()),
        kind: SymbolKind::FIELD,
        range: common::location_from_span(uri, decl.span(), lines)?.range,
        selection_range: common::location_from_span(uri, decl.name().span(), lines)?.range,
        children: None,
        tags: None,
        #[allow(deprecated)]
        deprecated: None,
    })
}

/// Converts a [`BoundDecl`] to a [`DocumentSymbol`].
fn bound_decl_to_symbol(
    uri: &Url,
    decl: &BoundDecl,
    lines: &std::sync::Arc<line_index::LineIndex>,
) -> Result<DocumentSymbol> {
    Ok(DocumentSymbol {
        name: decl.name().text().to_string(),
        detail: Some(decl.ty().to_string()),
        kind: SymbolKind::VARIABLE,
        range: common::location_from_span(uri, decl.span(), lines)?.range,
        selection_range: common::location_from_span(uri, decl.name().span(), lines)?.range,
        children: None,
        tags: None,
        #[allow(deprecated)]
        deprecated: None,
    })
}
