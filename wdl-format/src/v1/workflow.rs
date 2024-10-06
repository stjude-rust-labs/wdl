//! Formatting for workflows.

pub mod call;

use wdl_ast::SyntaxKind;

use crate::PreToken;
use crate::TokenStream;
use crate::Writable as _;
use crate::element::FormatElement;
use crate::exactly_one;

/// Formats a [`WorkflowDefinition`](wdl_ast::v1::WorkflowDefinition).
pub fn format_workflow_definition(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children_by_kind();

    if let Some(mut keywords) = children.remove(&SyntaxKind::WorkflowKeyword) {
        let keyword = exactly_one!(keywords, "workflow keywords");
        (&keyword).write(stream);
    }

    stream.end_word();

    if let Some(mut idents) = children.remove(&SyntaxKind::Ident) {
        let idents = exactly_one!(idents, "idents");
        (&idents).write(stream);
    }

    stream.end_word();

    if let Some(mut braces) = children.remove(&SyntaxKind::OpenBrace) {
        let brace = exactly_one!(braces, "open braces");
        (&brace).write(stream);
    }

    stream.end_line();
    stream.increment_indent();

    if let Some(calls) = children.remove(&SyntaxKind::CallStatementNode) {
        for call in calls {
            (&call).write(stream);
            stream.end_line();
        }
    }

    stream.decrement_indent();

    if let Some(mut braces) = children.remove(&SyntaxKind::CloseBrace) {
        let brace = exactly_one!(braces, "closed braces");
        (&brace).write(stream);
        stream.end_line();
    }

    if !children.is_empty() {
        todo!(
            "unhandled children for workflow definition: {:#?}",
            children.keys()
        );
    }
}
