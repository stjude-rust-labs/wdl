//! Formatting for tasks.

use wdl_ast::SyntaxKind;

use crate::PreToken;
use crate::TokenStream;
use crate::Writable as _;
use crate::element::FormatElement;
use crate::exactly_one;

/// Formats a [`TaskDefinition`](wdl_ast::v1::TaskDefinition).
pub fn format_task_definition(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children_by_kind();

    if let Some(mut keywords) = children.remove(&SyntaxKind::TaskKeyword) {
        let keyword = exactly_one!(keywords, "task keywords");
        (&keyword).write(stream);
    }

    if let Some(mut idents) = children.remove(&SyntaxKind::Ident) {
        let idents = exactly_one!(idents, "idents");
        (&idents).write(stream);
    }

    if let Some(mut braces) = children.remove(&SyntaxKind::OpenBrace) {
        let brace = exactly_one!(braces, "open braces");
        (&brace).write(stream);
    }

    if let Some(mut braces) = children.remove(&SyntaxKind::CloseBrace) {
        let brace = exactly_one!(braces, "closed braces");
        (&brace).write(stream);
    }

    if !children.is_empty() {
        todo!(
            "unhandled children for task definition: {:#?}",
            children.keys()
        );
    }
}
