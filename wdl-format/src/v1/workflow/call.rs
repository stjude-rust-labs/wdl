//! Formatting for workflow calls.

use wdl_ast::SyntaxKind;

use crate::PreToken;
use crate::TokenStream;
use crate::Writable as _;
use crate::element::FormatElement;
use crate::exactly_one;

/// Formats a [`CallStatement`](wdl_ast::v1::CallStatement).
pub fn format_call_statement(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children_by_kind();

    if let Some(mut keywords) = children.remove(&SyntaxKind::CallKeyword) {
        let keyword = exactly_one!(keywords, "call keywords");
        (&keyword).write(stream);
    }

    stream.end_word();

    if let Some(mut call_nodes) = children.remove(&SyntaxKind::CallTargetNode) {
        let call_node = exactly_one!(call_nodes, "call target nodes");
        (&call_node).write(stream);
    }

    stream.end_word();

    if let Some(mut open_braces) = children.remove(&SyntaxKind::OpenBrace) {
        let open_brace = exactly_one!(open_braces, "open braces");
        (&open_brace).write(stream);
    }

    stream.end_word();

    if let Some(mut close_braces) = children.remove(&SyntaxKind::CloseBrace) {
        let close_brace = exactly_one!(close_braces, "close braces");
        (&close_brace).write(stream);
    }

    stream.end_line();

    if !children.is_empty() {
        todo!(
            "unhandled children for call statement: {:#?}",
            children.keys()
        );
    }
}

/// Formats a [`CallTarget`](wdl_ast::v1::CallTarget).
pub fn format_call_target(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children_by_kind();

    if let Some(idents) = children.remove(&SyntaxKind::Ident) {
        let mut idents = idents.into_iter();
        let first_ident = idents.next().expect("at least one ident");
        (&first_ident).write(stream);

        if let Some(mut dots) = children.remove(&SyntaxKind::Dot) {
            let dot = exactly_one!(dots, "dots");
            (&dot).write(stream);

            let second_ident = idents.next().expect("second ident");
            (&second_ident).write(stream);

            assert!(idents.next().is_none(), "too many idents");
        }
    }

    if !children.is_empty() {
        todo!(
            "unhandled children for call statement: {:#?}",
            children.keys()
        );
    }
}
