//! Formatting for workflows.

pub mod call;

use wdl_ast::SyntaxKind;

use crate::PreToken;
use crate::TokenStream;
use crate::Writable as _;
use crate::element::FormatElement;

/// Formats a [`ConditionalStatement`](wdl_ast::v1::ConditionalStatement).
pub fn format_conditional_statement(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children().expect("conditional statement children");

    let if_keyword = children.next().expect("if keyword");
    assert!(if_keyword.element().kind() == SyntaxKind::IfKeyword);
    (&if_keyword).write(stream);
    stream.end_word();

    let open_paren = children.next().expect("open paren");
    assert!(open_paren.element().kind() == SyntaxKind::OpenParen);
    (&open_paren).write(stream);

    for child in children.by_ref() {
        (&child).write(stream);
        if child.element().kind() == SyntaxKind::CloseParen {
            stream.end_word();
            break;
        }
    }

    let open_brace = children.next().expect("open brace");
    assert!(open_brace.element().kind() == SyntaxKind::OpenBrace);
    (&open_brace).write(stream);
    stream.increment_indent();

    for child in children {
        if child.element().kind() == SyntaxKind::CloseBrace {
            stream.decrement_indent();
        }
        (&child).write(stream);
    }
    stream.end_line();
}

/// Formats a [`ScatterStatement`](wdl_ast::v1::ScatterStatement).
pub fn format_scatter_statement(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children().expect("scatter statement children");

    let scatter_keyword = children.next().expect("scatter keyword");
    assert!(scatter_keyword.element().kind() == SyntaxKind::ScatterKeyword);
    (&scatter_keyword).write(stream);
    stream.end_word();

    let open_paren = children.next().expect("open paren");
    assert!(open_paren.element().kind() == SyntaxKind::OpenParen);
    (&open_paren).write(stream);

    let variable = children.next().expect("scatter variable");
    assert!(variable.element().kind() == SyntaxKind::Ident);
    (&variable).write(stream);
    stream.end_word();

    let in_keyword = children.next().expect("in keyword");
    assert!(in_keyword.element().kind() == SyntaxKind::InKeyword);
    (&in_keyword).write(stream);
    stream.end_word();

    for child in children.by_ref() {
        (&child).write(stream);
        if child.element().kind() == SyntaxKind::CloseParen {
            stream.end_word();
            break;
        }
    }

    let open_brace = children.next().expect("open brace");
    assert!(open_brace.element().kind() == SyntaxKind::OpenBrace);
    (&open_brace).write(stream);
    stream.end_line();
    stream.increment_indent();

    for child in children {
        if child.element().kind() == SyntaxKind::CloseBrace {
            stream.decrement_indent();
        }
        (&child).write(stream);
    }
    stream.end_line();
}

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
            SyntaxKind::ScatterStatementNode => {
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
