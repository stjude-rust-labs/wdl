//! Formatting for tasks.

use wdl_ast::SyntaxKind;

use crate::PreToken;
use crate::TokenStream;
use crate::Writable as _;
use crate::element::FormatElement;

/// Formats a [`TaskDefinition`](wdl_ast::v1::TaskDefinition).
pub fn format_task_definition(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    for child in element.children().expect("task definition children") {
        match child.element().kind() {
            SyntaxKind::TaskKeyword => {
                (&child).write(stream);
                stream.end_word();
            }
            SyntaxKind::Ident => {
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
                    "unexpected child in task definition: {:?}",
                    child.element().kind()
                );
            }
        }
    }
}
