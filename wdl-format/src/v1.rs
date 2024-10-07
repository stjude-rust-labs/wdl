//! Formatting of WDL v1.x elements.

use wdl_ast::SyntaxKind;

pub mod import;
pub mod task;
pub mod workflow;

use crate::PreToken;
use crate::TokenStream;
use crate::Writable as _;
use crate::element::FormatElement;
use crate::exactly_one;

/// Formats an [`Ast`](wdl_ast::Ast).
pub fn format_ast(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children_by_kind();

    if let Some(mut versions) = children.remove(&SyntaxKind::VersionStatementNode) {
        let version = exactly_one!(versions, "version statements");

        // TODO(clay): improve this by removing the reference.
        (&version).write(stream);
    }

    stream.blank_line();

    if let Some(imports) = children.remove(&SyntaxKind::ImportStatementNode) {
        for import in imports {
            (&import).write(stream);
        }
    }

    stream.blank_line();

    if let Some(tasks) = children.remove(&SyntaxKind::TaskDefinitionNode) {
        for task in tasks {
            (&task).write(stream);
            stream.blank_line();
        }
    }

    if let Some(workflows) = children.remove(&SyntaxKind::WorkflowDefinitionNode) {
        for workflow in workflows {
            (&workflow).write(stream);
            stream.blank_line();
        }
    }

    if !children.is_empty() {
        todo!("unhandled children for AST: {:#?}", children.keys());
    }
}

/// Formats a [`VersionStatement`](wdl_ast::VersionStatement).
pub fn format_version_statement(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    for child in element.children().expect("version statement children") {
        (&child).write(stream);
        stream.end_word();
    }
    stream.end_line();
}

/// Formats a [`LiteralString`](wdl_ast::v1::LiteralString).
pub fn format_literal_string(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    for child in element.children().expect("literal string children") {
        match child.element().kind() {
            SyntaxKind::SingleQuote => {
                stream.push_literal_in_place_of_token(
                    child.element().as_token().expect("token"),
                    "\"".to_owned(),
                );
            }
            _ => {
                (&child).write(stream);
            }
        }
    }
}
