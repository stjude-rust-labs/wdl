//! Formatting for tasks.

use wdl_ast::SyntaxKind;

use crate::PreToken;
use crate::Trivia;
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
    let mut command = None;
    let mut output = None;
    let mut requirements = None;
    let mut runtime = None;
    let mut hints = None;
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
            SyntaxKind::BoundDeclNode => {
                body.push(child.clone());
            }
            SyntaxKind::CommandSectionNode => {
                command = Some(child.clone());
            }
            SyntaxKind::OutputSectionNode => {
                output = Some(child.clone());
            }
            SyntaxKind::RequirementsSectionNode => {
                requirements = Some(child.clone());
            }
            SyntaxKind::RuntimeSectionNode => {
                runtime = Some(child.clone());
            }
            SyntaxKind::TaskHintsSectionNode => {
                hints = Some(child.clone());
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

    let need_blank = !body.is_empty();
    for child in body {
        (&child).write(stream);
    }
    if need_blank {
        stream.blank_line();
    }

    if let Some(command) = command {
        (&command).write(stream);
        stream.blank_line();
    }

    if let Some(output) = output {
        (&output).write(stream);
        stream.blank_line();
    }

    if let Some(requirements) = requirements {
        (&requirements).write(stream);
        stream.blank_line();
    } else if let Some(runtime) = runtime {
        (&runtime).write(stream);
        stream.blank_line();
    }

    if let Some(hints) = hints {
        (&hints).write(stream);
        stream.blank_line();
    }

    stream.trim_while(|t| matches!(t, PreToken::BlankLine | PreToken::Trivia(Trivia::BlankLine)));

    stream.decrement_indent();
    (&close_brace.expect("task close brace")).write(stream);
    stream.end_line();
}

/// Formats a [`CommandSection`](wdl_ast::v1::CommandSection).
pub fn format_command_section(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children().expect("command section children");

    let command_keyword = children.next().expect("command keyword");
    assert!(command_keyword.element().kind() == SyntaxKind::CommandKeyword);
    (&command_keyword).write(stream);
    stream.end_word();

    let open_delimiter = children.next().expect("open delimiter");
    match open_delimiter.element().kind() {
        SyntaxKind::OpenBrace => {
            stream.push_literal_in_place_of_token(open_delimiter.element().as_token().expect("open brace should be token"), "<<<".to_string());
        },
        SyntaxKind::OpenHeredoc => {
            (&open_delimiter).write(stream);
        },
        _ => {
            unreachable!("unexpected open delimiter in command section: {:?}", open_delimiter.element().kind());
        }
    }
    stream.increment_indent();

    for child in children {
        let kind = child.element().kind();
        if kind == SyntaxKind::CloseBrace {
            stream.decrement_indent();
            stream.push_literal_in_place_of_token(child.element().as_token().expect("close brace should be token"), ">>>".to_string());
        } else if kind == SyntaxKind::CloseHeredoc {
            stream.decrement_indent();
            (&child).write(stream);
        } else {
            assert!(matches!(kind, SyntaxKind::LiteralCommandText | SyntaxKind::PlaceholderNode));
            (&child).write(stream);
        }
    }
    stream.end_line();
}

/// Formats a [`RequirementsItem`](wdl_ast::v1::RequirementsItem).
pub fn format_requirements_item(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children().expect("requirements item children");

    let name = children.next().expect("requirements item name");
    assert!(name.element().kind() == SyntaxKind::Ident);
    (&name).write(stream);

    let colon = children.next().expect("requirements item colon");
    assert!(colon.element().kind() == SyntaxKind::Colon);
    (&colon).write(stream);
    stream.end_word();

    let value = children.next().expect("requirements item value");
    (&value).write(stream);

    assert!(children.next().is_none());
}

/// Formats a [`RequirementsSection`](wdl_ast::v1::RequirementsSection).
pub fn format_requirements_section(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children().expect("requirements section children");

    let requirements_keyword = children.next().expect("requirements keyword");
    assert!(requirements_keyword.element().kind() == SyntaxKind::RequirementsKeyword);
    (&requirements_keyword).write(stream);
    stream.end_word();

    let open_brace = children.next().expect("open brace");
    assert!(open_brace.element().kind() == SyntaxKind::OpenBrace);
    (&open_brace).write(stream);
    stream.increment_indent();

    let mut items = Vec::new();
    let mut close_brace = None;

    for child in children {
        match child.element().kind() {
            SyntaxKind::RequirementsItemNode => {
                items.push(child.clone());
            }
            SyntaxKind::CloseBrace => {
                close_brace = Some(child.clone());
            }
            _ => {
                unreachable!(
                    "unexpected child in requirements section: {:?}",
                    child.element().kind()
                );
            }
        }
    }

    for item in items {
        (&item).write(stream);
    }

    stream.decrement_indent();
    (&close_brace.expect("requirements close brace")).write(stream);
    stream.end_line();
}

/// Formats a [`TaskHintsItem`](wdl_ast::v1::TaskHintsItem).
pub fn format_task_hints_item(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children().expect("task hints item children");

    let name = children.next().expect("task hints item name");
    assert!(name.element().kind() == SyntaxKind::Ident);
    (&name).write(stream);

    let colon = children.next().expect("task hints item colon");
    assert!(colon.element().kind() == SyntaxKind::Colon);
    (&colon).write(stream);
    stream.end_word();

    let value = children.next().expect("task hints item value");
    (&value).write(stream);

    assert!(children.next().is_none());
}

/// Formats a [`RuntimeItem`](wdl_ast::v1::RuntimeItem).
pub fn format_runtime_item(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children().expect("runtime item children");

    let name = children.next().expect("runtime item name");
    assert!(name.element().kind() == SyntaxKind::Ident);
    (&name).write(stream);

    let colon = children.next().expect("runtime item colon");
    assert!(colon.element().kind() == SyntaxKind::Colon);
    (&colon).write(stream);
    stream.end_word();

    let value = children.next().expect("runtime item value");
    (&value).write(stream);

    assert!(children.next().is_none());
}

/// Formats a [`RuntimeSection`](wdl_ast::v1::RuntimeSection).
pub fn format_runtime_section(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children().expect("runtime section children");

    let runtime_keyword = children.next().expect("runtime keyword");
    assert!(runtime_keyword.element().kind() == SyntaxKind::RuntimeKeyword);
    (&runtime_keyword).write(stream);
    stream.end_word();

    let open_brace = children.next().expect("open brace");
    assert!(open_brace.element().kind() == SyntaxKind::OpenBrace);
    (&open_brace).write(stream);
    stream.increment_indent();

    let mut items = Vec::new();
    let mut close_brace = None;

    for child in children {
        match child.element().kind() {
            SyntaxKind::RuntimeItemNode => {
                items.push(child.clone());
            }
            SyntaxKind::CloseBrace => {
                close_brace = Some(child.clone());
            }
            _ => {
                unreachable!(
                    "unexpected child in runtime section: {:?}",
                    child.element().kind()
                );
            }
        }
    }

    for item in items {
        (&item).write(stream);
        stream.end_line();
    }

    stream.decrement_indent();
    (&close_brace.expect("runtime close brace")).write(stream);
    stream.end_line();
}

/// Formats a [`TaskHintsSection`](wdl_ast::v1::TaskHintsSection).
pub fn format_task_hints_section(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children().expect("task hints section children");

    let hints_keyword = children.next().expect("hints keyword");
    assert!(hints_keyword.element().kind() == SyntaxKind::HintsKeyword);
    (&hints_keyword).write(stream);
    stream.end_word();

    let open_brace = children.next().expect("open brace");
    assert!(open_brace.element().kind() == SyntaxKind::OpenBrace);
    (&open_brace).write(stream);
    stream.increment_indent();

    let mut items = Vec::new();
    let mut close_brace = None;

    for child in children {
        match child.element().kind() {
            SyntaxKind::TaskHintsItemNode => {
                items.push(child.clone());
            }
            SyntaxKind::CloseBrace => {
                close_brace = Some(child.clone());
            }
            _ => {
                unreachable!(
                    "unexpected child in task hints section: {:?}",
                    child.element().kind()
                );
            }
        }
    }

    for item in items {
        (&item).write(stream);
    }

    stream.decrement_indent();
    (&close_brace.expect("task hints close brace")).write(stream);
    stream.end_line();
}
