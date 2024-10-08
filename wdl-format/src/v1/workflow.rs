//! Formatting for workflows.

pub mod call;

use wdl_ast::SyntaxKind;

use crate::PreToken;
use crate::TokenStream;
use crate::Writable as _;
use crate::element::FormatElement;

/// Formats a [`WorkflowDefinition`](wdl_ast::v1::WorkflowDefinition).
pub fn format_workflow_definition(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children().expect("workflow definition children");

    let workflow_keyword = children.next().expect("workflow keyword");
    assert!(workflow_keyword.element().kind() == SyntaxKind::WorkflowKeyword);
    (&workflow_keyword).write(stream);
    stream.end_word();

    let name = children.next().expect("workflow name");
    assert!(name.element().kind() == SyntaxKind::Ident);
    (&name).write(stream);
    stream.end_word();

    let open_brace = children.next().expect("open brace");
    assert!(open_brace.element().kind() == SyntaxKind::OpenBrace);
    (&open_brace).write(stream);
    stream.end_line();
    stream.increment_indent();

    let mut meta = None;
    let mut parameter_meta = None;
    let mut input = None;
    let mut body = Vec::new();
    let mut output = None;
    let mut close_brace = None;

    for child in children {
        match child.element().kind() {
            SyntaxKind::MetadataSectionNode => {
                meta = Some(child.clone());
            }
            SyntaxKind::ParameterMetadataSectionNode => {
                parameter_meta = Some(child.clone());
            }
            SyntaxKind::InputSectionNode => {
                input = Some(child.clone());
            }
            SyntaxKind::BoundDeclNode => {
                body.push(child.clone());
            }
            SyntaxKind::CallStatementNode => {
                body.push(child.clone());
            }
            SyntaxKind::ConditionalStatementNode => {
                body.push(child.clone());
            }
            SyntaxKind::OutputSectionNode => {
                output = Some(child.clone());
            }
            SyntaxKind::CloseBrace => {
                close_brace = Some(child.clone());
            }
            _ => {
                unreachable!(
                    "unexpected child in workflow definition: {:?}",
                    child.element().kind()
                );
            }
        }
    }

    if let Some(meta) = meta {
        (&meta).write(stream);
        stream.blank_line();
    }

    if let Some(parameter_meta) = parameter_meta {
        (&parameter_meta).write(stream);
        stream.blank_line();
    }

    if let Some(input) = input {
        (&input).write(stream);
        stream.blank_line();
    }

    for child in body {
        (&child).write(stream);
    }

    if let Some(output) = output {
        (&output).write(stream);
        stream.blank_line();
    }

    stream.decrement_indent();
    (&close_brace.expect("workflow close brace")).write(stream);
    stream.end_line();
}
