//! Formatting for workflow calls.

use wdl_ast::SyntaxKind;

use crate::PreToken;
use crate::TokenStream;
use crate::Writable as _;
use crate::element::FormatElement;

/// Formats a [`CallStatement`](wdl_ast::v1::CallStatement).
pub fn format_call_statement(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    for child in element.children().expect("call statement children") {
        match child.element().kind() {
            SyntaxKind::CallKeyword => {
                (&child).write(stream);
                stream.end_word();
            }
            SyntaxKind::CallTargetNode => {
                (&child).write(stream);
                stream.end_word();
            }
            SyntaxKind::OpenBrace => {
                (&child).write(stream);
                stream.end_line();
                stream.increment_indent();
            }
            SyntaxKind::CloseBrace => {
                stream.decrement_indent();
                (&child).write(stream);
                stream.end_line();
            }
            _ => {
                unreachable!(
                    "unexpected child in call statement: {:?}",
                    child.element().kind()
                );
            }
        }
    }
}

/// Formats a [`CallTarget`](wdl_ast::v1::CallTarget).
pub fn format_call_target(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    for child in element.children().expect("call target children") {
        (&child).write(stream);
    }
}
