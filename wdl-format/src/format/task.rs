/// Format a task definition.
use wdl_ast::v1::CommandPart;
use wdl_ast::v1::CommandSection;
use wdl_ast::v1::RuntimeSection;
use wdl_ast::v1::TaskDefinition;
use wdl_ast::v1::TaskItem;
use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::SyntaxElement;
use wdl_ast::SyntaxKind;

use super::comments::format_inline_comment;
use super::comments::format_preceding_comments;
use super::INDENT;
use super::NEWLINE;
use super::*;

/// Format a command section.
fn format_command_section(command: CommandSection) -> String {
    let mut result = String::new();

    result.push_str(&format_preceding_comments(
        &SyntaxElement::Node(command.syntax().clone()),
        0,
        false,
        false,
    ));
    result.push_str(INDENT);
    result.push_str("command");
    result.push_str(&format_inline_comment(
        &command
            .syntax()
            .first_child_or_token()
            .expect("Command section should have a first child"),
        false,
    ));

    if command.is_heredoc() {
        let open_heredoc = command
            .syntax()
            .children_with_tokens()
            .find(|c| c.kind() == SyntaxKind::OpenHeredoc)
            .expect("Command section should have an open heredoc");
        result.push_str(&format_preceding_comments(
            &open_heredoc,
            1,
            false,
            false,
        ));
        if result.ends_with(NEWLINE) {
            result.push_str(INDENT);
        } else {
            result.push(' ');
        }
        result.push_str("<<<");
        // Open heredoc inline comment will be part of the command text
    } else {
        let open_brace = command
            .syntax()
            .children_with_tokens()
            .find(|c| c.kind() == SyntaxKind::OpenBrace)
            .expect("Command section should have an open brace");
        result.push_str(&format_preceding_comments(
            &open_brace,
            1,
            false,
            false,
        ));
        if !result.ends_with(NEWLINE) {
            result.push(' ');
        } else {
            result.push_str(INDENT);
        }
        result.push('{');
        // Open brace inline comment will be part of the command text
    }

    for part in command.parts() {
        match part {
            CommandPart::Text(t) => {
                result.push_str(t.as_str());
            }
            CommandPart::Placeholder(p) => {
                result.push_str(&p.syntax().to_string());
            }
        }
    }

    if command.is_heredoc() {
        let close_heredoc = command
            .syntax()
            .children_with_tokens()
            .find(|c| c.kind() == SyntaxKind::CloseHeredoc)
            .expect("Command section should have a close heredoc");
        // Close heredoc preceding comment will be part of the command text
        result.push_str(">>>");
        result.push_str(&format_inline_comment(&close_heredoc, true));
    } else {
        let close_brace = command
            .syntax()
            .children_with_tokens()
            .find(|c| c.kind() == SyntaxKind::CloseBrace)
            .expect("Command section should have a close brace");
        // Close brace preceding comment will be part of the command text
        result.push('}');
        result.push_str(&format_inline_comment(&close_brace, true));
    }

    result
}

/// Format a runtime section
fn format_runtime_section(runtime: RuntimeSection) -> String {
    let mut result = String::new();
    let one_indent = INDENT;
    let two_indents = INDENT.repeat(2);

    result.push_str(&format_preceding_comments(
        &SyntaxElement::Node(runtime.syntax().clone()),
        1,
        false,
        false,
    ));
    result.push_str(one_indent);
    result.push_str("runtime");
    result.push_str(&format_inline_comment(
        &runtime
            .syntax()
            .first_child_or_token()
            .expect("Runtime section should have a first child"),
        false,
    ));

    let open_brace = runtime
        .syntax()
        .children_with_tokens()
        .find(|c| c.kind() == SyntaxKind::OpenBrace)
        .expect("Runtime section should have an open brace");
    result.push_str(&format_preceding_comments(
        &open_brace,
        1,
        false,
        false,
    ));
    if !result.ends_with(NEWLINE) {
        result.push(' ');
    } else {
        result.push_str(one_indent);
    }
    result.push('{');
    result.push_str(&format_inline_comment(&open_brace, true));

    for item in runtime.items() {
        result.push_str(&format_preceding_comments(
            &SyntaxElement::Node(item.syntax().clone()),
            2,
            false,
            false,
        ));
        result.push_str(&two_indents);
        result.push_str(item.name().as_str());
        result.push_str(&format_inline_comment(
            &SyntaxElement::Token(item.name().syntax().clone()),
            false,
        ));

        let colon = item
            .syntax()
            .children_with_tokens()
            .find(|c| c.kind() == SyntaxKind::Colon)
            .expect("Runtime item should have a colon");
        result.push_str(&format_preceding_comments(
            &colon,
            2,
            false,
            false,
        ));
        result.push(':');
        result.push_str(&format_inline_comment(&colon, false));

        result.push_str(&format_preceding_comments(
            &SyntaxElement::Node(item.expr().syntax().clone()),
            2,
            false,
            false,
        ));
        if result.ends_with(NEWLINE) {
            result.push_str(&two_indents);
        } else {
            result.push(' ');
        }
        result.push_str(&item.expr().syntax().to_string());
        result.push_str(&format_inline_comment(
            &SyntaxElement::Node(item.syntax().clone()),
            true,
        ));
    }

    let close_brace = runtime
        .syntax()
        .children_with_tokens()
        .find(|c| c.kind() == SyntaxKind::CloseBrace)
        .expect("Runtime section should have a close brace");
    result.push_str(&format_preceding_comments(
        &close_brace,
        1,
        false,
        false,
    ));
    if !result.ends_with(NEWLINE) {
        result.push_str(NEWLINE);
    }
    result.push_str(one_indent);
    result.push('}');
    result.push_str(&format_inline_comment(&close_brace, true));

    result
}

/// Format a task definition.
pub fn format_task(task_def: &TaskDefinition) -> String {
    let mut result = String::new();
    result.push_str(&format_preceding_comments(
        &SyntaxElement::Node(task_def.syntax().clone()),
        0,
        false,
        false,
    ));
    result.push_str("task");
    result.push_str(&format_inline_comment(
        &task_def
            .syntax()
            .first_child_or_token()
            .expect("Task definition should have a first child"),
        false,
    ));

    result.push_str(&format_preceding_comments(
        &SyntaxElement::Token(task_def.name().syntax().clone()),
        1,
        false,
        false,
    ));
    if result.ends_with("task") {
        result.push(' ');
    } else {
        result.push_str(INDENT);
    }
    result.push_str(task_def.name().as_str());
    result.push_str(&format_inline_comment(
        &SyntaxElement::Token(task_def.name().syntax().clone()),
        false,
    ));

    let open_brace = task_def
        .syntax()
        .children_with_tokens()
        .find(|c| c.kind() == SyntaxKind::OpenBrace)
        .expect("Task definition should have an open brace");
    result.push_str(&format_preceding_comments(
        &open_brace,
        0,
        false,
        false,
    ));
    if !result.ends_with(NEWLINE) {
        result.push(' ');
    }
    result.push('{');
    result.push_str(&format_inline_comment(&open_brace, true));

    let mut meta_section_str = String::new();
    let mut parameter_meta_section_str = String::new();
    let mut input_section_str = String::new();
    let mut declaration_section_str = String::new();
    let mut command_section_str = String::new();
    let mut output_section_str = String::new();
    let mut runtime_section_str = String::new();
    for item in task_def.items() {
        match item {
            TaskItem::Metadata(m) => {
                meta_section_str.push_str(&format_meta_section(m));
            }
            TaskItem::ParameterMetadata(pm) => {
                parameter_meta_section_str.push_str(&format_parameter_meta_section(pm));
            }
            TaskItem::Input(i) => {
                input_section_str.push_str(&format_input_section(i));
            }
            TaskItem::Declaration(d) => {
                declaration_section_str.push_str(&format_declaration(&Decl::Bound(d), 1));
            }
            TaskItem::Command(c) => {
                command_section_str.push_str(&format_command_section(c));
            }
            TaskItem::Output(o) => {
                output_section_str.push_str(&format_output_section(o));
            }
            TaskItem::Runtime(r) => {
                runtime_section_str.push_str(&format_runtime_section(r));
            }
        }
    }

    if !meta_section_str.is_empty() {
        result.push_str(&meta_section_str);
        result.push_str(NEWLINE);
    }
    if !parameter_meta_section_str.is_empty() {
        result.push_str(&parameter_meta_section_str);
        result.push_str(NEWLINE);
    }
    if !input_section_str.is_empty() {
        result.push_str(&input_section_str);
        result.push_str(NEWLINE);
    }
    if !declaration_section_str.is_empty() {
        result.push_str(&declaration_section_str);
        result.push_str(NEWLINE);
    }
    if !command_section_str.is_empty() {
        result.push_str(&command_section_str);
        if !output_section_str.is_empty() || !runtime_section_str.is_empty() {
            result.push_str(NEWLINE);
        }
    }
    if !output_section_str.is_empty() {
        result.push_str(&output_section_str);
        if !runtime_section_str.is_empty() {
            result.push_str(NEWLINE);
        }
    }
    if !runtime_section_str.is_empty() {
        result.push_str(&runtime_section_str);
    }

    let close_brace = task_def
        .syntax()
        .children_with_tokens()
        .find(|c| c.kind() == SyntaxKind::CloseBrace)
        .expect("Task definition should have a close brace");
    result.push_str(&format_preceding_comments(
        &close_brace,
        0,
        false,
        false,
    ));
    if !result.ends_with(NEWLINE) {
        result.push_str(NEWLINE);
    }
    result.push('}');
    result.push_str(&format_inline_comment(&close_brace, true));
    result.push_str(NEWLINE);

    result
}
