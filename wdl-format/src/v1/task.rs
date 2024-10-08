//! Formatting for tasks.

use wdl_ast::SyntaxKind;

use crate::PreToken;
use crate::TokenStream;
use crate::Writable as _;
use crate::element::FormatElement;

/// Formats a [`TaskDefinition`](wdl_ast::v1::TaskDefinition).
pub fn format_task_definition(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children().expect("task definition children");

    let task_keyword = children.next().expect("task keyword");
    assert!(task_keyword.element().kind() == SyntaxKind::TaskKeyword);
    (&task_keyword).write(stream);
    stream.end_word();

    let name = children.next().expect("task name");
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
    let mut runtime = None;
    let mut command = None;
    let mut output = None;
    let mut close_brace = None;

    for child in children {
        match child.element().kind() {
            SyntaxKind::InputSectionNode => {
                input = Some(child.clone());
            }
            SyntaxKind::MetadataSectionNode => {
                meta = Some(child.clone());
            }
            SyntaxKind::ParameterMetadataSectionNode => {
                parameter_meta = Some(child.clone());
            }
            SyntaxKind::RuntimeSectionNode => {
                runtime = Some(child.clone());
            }
            SyntaxKind::CommandSectionNode => {
                command = Some(child.clone());
            }
            SyntaxKind::OutputSectionNode => {
                output = Some(child.clone());
            }
            SyntaxKind::BoundDeclNode => {
                body.push(child.clone());
            }
            SyntaxKind::CloseBrace => {
                close_brace = Some(child.clone());
            }
            _ => {
                unreachable!(
                    "unexpected child in task definition: {:?}",
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

    if let Some(command) = command {
        (&command).write(stream);
        stream.blank_line();
    }

    if let Some(output) = output {
        (&output).write(stream);
        stream.blank_line();
    }

    if let Some(runtime) = runtime {
        (&runtime).write(stream);
        stream.blank_line();
    }

    stream.decrement_indent();
    (&close_brace.expect("task close brace")).write(stream);
    stream.end_line();
}
