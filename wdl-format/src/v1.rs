//! Formatting of WDL v1.x elements.

use wdl_ast::SyntaxKind;

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
    let mut children = element.children_by_kind();

    if let Some(mut keywords) = children.remove(&SyntaxKind::VersionKeyword) {
        let keyword = exactly_one!(keywords, "`version` keywords");
        (&keyword).write(stream);
    }

    stream.end_word();

    if let Some(mut versions) = children.remove(&SyntaxKind::Version) {
        let version = exactly_one!(versions, "versions");
        (&version).write(stream);
    }

    stream.end_line();

    if !children.is_empty() {
        todo!(
            "unhandled children for version statement: {:#?}",
            children.keys()
        );
    }
}
