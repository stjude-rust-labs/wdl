//! Formatting for workflows.

pub mod call;

use wdl_ast::SyntaxKind;

use crate::PreToken;
use crate::TokenStream;
use crate::Writable as _;
use crate::element::FormatElement;

/// Formats a [`WorkflowDefinition`](wdl_ast::v1::WorkflowDefinition).
pub fn format_workflow_definition(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    for child in element.children().expect("workflow definition children") {
        match child.element().kind() {
            SyntaxKind::WorkflowKeyword => {
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
            SyntaxKind::CallStatementNode => {
                (&child).write(stream);
            }
            SyntaxKind::CloseBrace => {
                stream.decrement_indent();
                (&child).write(stream);
                stream.end_line();
            }
            _ => {
                unreachable!(
                    "unexpected child in workflow definition: {:?}",
                    child.element().kind()
                );
            }
        }
    }
}
