/// Format a workflow definition.
use wdl_ast::v1::CallStatement;
use wdl_ast::v1::ConditionalStatement;
use wdl_ast::v1::ScatterStatement;
use wdl_ast::v1::WorkflowDefinition;
use wdl_ast::v1::WorkflowItem;
use wdl_ast::v1::WorkflowStatement;
use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::SyntaxElement;
use wdl_ast::SyntaxKind;

use super::comments::format_inline_comment;
use super::comments::format_preceding_comments;
use super::INDENT;
use super::NEWLINE;
use super::*;

fn format_call_statement(call: CallStatement, num_indents: usize) -> String {
    let mut result = String::new();
    let next_num_indents = num_indents + 1;
    let cur_indents = INDENT.repeat(num_indents);
    let next_indents = INDENT.repeat(next_num_indents);

    result.push_str(&format_preceding_comments(
        &SyntaxElement::Node(call.syntax().clone()),
        num_indents,
    ));
    result.push_str(&cur_indents);
    result.push_str("call");
    result.push_str(&format_inline_comment(
        &call
            .syntax()
            .first_child_or_token()
            .expect("Call statement should have a child"),
        false,
    ));

    result.push_str(&format_preceding_comments(
        &SyntaxElement::Node(call.target().syntax().clone()),
        next_num_indents,
    ));
    if result.ends_with("call") {
        result.push(' ');
    } else if result.ends_with(NEWLINE) {
        result.push_str(&next_indents);
    }
    result.push_str(&call.target().syntax().to_string());
    result.push_str(&format_inline_comment(
        &SyntaxElement::Node(call.target().syntax().clone()),
        false,
    ));

    if let Some(alias) = call.alias() {
        for child in alias.syntax().children_with_tokens() {
            match child.kind() {
                SyntaxKind::AsKeyword => {
                    result.push_str(&format_preceding_comments(&child, next_num_indents));
                    if result.ends_with(NEWLINE) {
                        result.push_str(&next_indents);
                    } else {
                        result.push(' ');
                    }
                    result.push_str("as");
                    result.push_str(&format_inline_comment(&child, false))
                }
                SyntaxKind::Ident => {
                    result.push_str(&format_preceding_comments(&child, next_num_indents));
                    if result.ends_with(NEWLINE) {
                        result.push_str(&next_indents);
                    } else {
                        result.push(' ');
                    }
                    result.push_str(&child.to_string());

                    // This will be the last child processed which means it won't have any "next"
                    // siblings. So we go up a level and check if there are
                    // siblings of the 'CallAliasNode'.
                    result.push_str(&format_inline_comment(
                        &SyntaxElement::Node(alias.syntax().clone()),
                        false,
                    ));
                }
                SyntaxKind::Whitespace => {
                    // Ignore
                }
                SyntaxKind::Comment => {
                    // Handled by another match arm
                }
                _ => {
                    unreachable!("Unexpected syntax kind: {:?}", child.kind());
                }
            }
        }
    }

    for after in call.after() {
        for child in after.syntax().children_with_tokens() {
            match child.kind() {
                SyntaxKind::AfterKeyword => {
                    result.push_str(&format_preceding_comments(
                        &SyntaxElement::Node(after.syntax().clone()),
                        next_num_indents,
                    ));
                    if result.ends_with(NEWLINE) {
                        result.push_str(&next_indents);
                    } else {
                        result.push(' ');
                    }
                    result.push_str("after");
                    result.push_str(&format_inline_comment(&child, false));
                }
                SyntaxKind::Ident => {
                    result.push_str(&format_preceding_comments(&child, next_num_indents));
                    if result.ends_with(NEWLINE) {
                        result.push_str(&next_indents);
                    } else {
                        result.push(' ');
                    }
                    result.push_str(&child.to_string());

                    // This will be the last child processed which means it won't have any "next"
                    // siblings. So we go up a level and check if there are
                    // siblings of the 'CallAfterNode'.
                    result.push_str(&format_inline_comment(
                        &SyntaxElement::Node(after.syntax().clone()),
                        false,
                    ));
                }
                SyntaxKind::Whitespace => {
                    // Ignore
                }
                SyntaxKind::Comment => {
                    // Handled by another match arm
                }
                _ => {
                    unreachable!("Unexpected syntax kind: {:?}", child.kind());
                }
            }
        }
    }

    let inputs: Vec<_> = call.inputs().collect();
    // TODO handle inputs of length 1 differently
    if !inputs.is_empty() {
        let open_brace = call
            .syntax()
            .children_with_tokens()
            .find(|c| c.kind() == SyntaxKind::OpenBrace)
            .expect("Call statement should have an open brace");
        result.push_str(&format_preceding_comments(&open_brace, next_num_indents));
        if result.ends_with(NEWLINE) {
            result.push_str(&next_indents);
        } else {
            result.push(' ');
        }
        result.push('{');
        result.push_str(&format_inline_comment(&open_brace, false));

        let input_keyword = call
            .syntax()
            .children_with_tokens()
            .find(|c| c.kind() == SyntaxKind::InputKeyword)
            .expect("Call statement should have an input keyword");
        result.push_str(&format_preceding_comments(&input_keyword, next_num_indents));
        if result.ends_with(NEWLINE) {
            result.push_str(&next_indents);
        } else {
            result.push(' ');
        }
        result.push_str("input");
        result.push_str(&format_inline_comment(&input_keyword, false));

        let colon = call
            .syntax()
            .children_with_tokens()
            .find(|c| c.kind() == SyntaxKind::Colon)
            .expect("Call statement should have a colon");
        result.push_str(&format_preceding_comments(&colon, next_num_indents));
        if result.ends_with(NEWLINE) {
            result.push_str(&next_indents);
        }
        result.push(':');
        result.push_str(&format_inline_comment(&colon, true));

        let mut commas = call
            .syntax()
            .children_with_tokens()
            .filter(|c| c.kind() == SyntaxKind::Comma);
        for item in inputs {
            result.push_str(&format_preceding_comments(
                &SyntaxElement::Node(item.syntax().clone()),
                next_num_indents,
            ));

            result.push_str(&next_indents);
            result.push_str(item.name().as_str());
            result.push_str(&format_inline_comment(
                &SyntaxElement::Token(item.name().syntax().clone()),
                false,
            ));

            if let Some(expr) = item.expr() {
                let equal_sign = item
                    .syntax()
                    .children_with_tokens()
                    .find(|c| c.kind() == SyntaxKind::Assignment)
                    .expect("Call input should have an equal sign");
                result.push_str(&format_preceding_comments(&equal_sign, next_num_indents));
                if result.ends_with(NEWLINE) {
                    result.push_str(&next_indents);
                } else {
                    result.push(' ');
                }
                result.push('=');
                result.push_str(&format_inline_comment(&equal_sign, false));

                result.push_str(&format_preceding_comments(
                    &SyntaxElement::Node(expr.syntax().clone()),
                    next_num_indents,
                ));
                if !result.ends_with(NEWLINE) {
                    result.push(' ');
                } else {
                    result.push_str(&next_indents);
                }
                result.push_str(&expr.syntax().to_string()); // TODO: format expressions
            }

            result.push_str(&format_inline_comment(
                &SyntaxElement::Node(item.syntax().clone()),
                false,
            ));

            if let Some(cur_comma) = commas.next() {
                result.push_str(&format_preceding_comments(&cur_comma, next_num_indents));
                result.push(',');
                result.push_str(&format_inline_comment(&cur_comma, false));
            } else {
                result.push(',');
            }
            if !result.ends_with(NEWLINE) {
                result.push_str(NEWLINE);
            }
        }

        let close_brace = call
            .syntax()
            .children_with_tokens()
            .find(|c| c.kind() == SyntaxKind::CloseBrace)
            .expect("Call statement should have a close brace");
        result.push_str(&format_preceding_comments(&close_brace, num_indents));
        if result.ends_with(NEWLINE) {
            result.push_str(&cur_indents);
        } else {
            result.push_str(NEWLINE);
            result.push_str(&cur_indents);
        }
        result.push('}');
    }

    result.push_str(&format_inline_comment(
        &SyntaxElement::Node(call.syntax().clone()),
        true,
    ));

    result
}

/// Format a conditional statement.
fn format_conditional(conditional: ConditionalStatement, num_indents: usize) -> String {
    let mut result = String::new();
    let next_num_indents = num_indents + 1;
    let cur_indents = INDENT.repeat(num_indents);
    let next_indents = INDENT.repeat(next_num_indents);

    result.push_str(&format_preceding_comments(
        &SyntaxElement::Node(conditional.syntax().clone()),
        num_indents,
    ));

    let if_keyword = conditional
        .syntax()
        .children_with_tokens()
        .find(|c| c.kind() == SyntaxKind::IfKeyword)
        .expect("Conditional statement should have an if keyword");
    result.push_str(&cur_indents);
    result.push_str("if");
    result.push_str(&format_inline_comment(&if_keyword, false));

    let open_paren = conditional
        .syntax()
        .children_with_tokens()
        .find(|c| c.kind() == SyntaxKind::OpenParen)
        .expect("Conditional statement should have an open paren");
    result.push_str(&format_preceding_comments(&open_paren, num_indents));
    if result.ends_with(NEWLINE) {
        result.push_str(&cur_indents);
    } else {
        result.push(' ');
    }
    result.push('(');
    result.push_str(&format_inline_comment(&open_paren, false));
    let mut paren_on_same_line = true;
    if result.ends_with(NEWLINE) {
        paren_on_same_line = false;
        result.push_str(&next_indents);
    }

    result.push_str(&format_preceding_comments(
        &SyntaxElement::Node(conditional.expr().syntax().clone()),
        next_num_indents,
    ));
    let conditional_expr = conditional.expr().syntax().to_string();
    if conditional_expr.contains('\n') {
        paren_on_same_line = false;
    }
    result.push_str(&conditional_expr);
    result.push_str(&format_inline_comment(
        &SyntaxElement::Node(conditional.expr().syntax().clone()),
        false,
    ));

    let close_paren = conditional
        .syntax()
        .children_with_tokens()
        .find(|c| c.kind() == SyntaxKind::CloseParen)
        .expect("Conditional statement should have a close paren");
    result.push_str(&format_preceding_comments(&close_paren, num_indents));
    if !paren_on_same_line && result.ends_with(&conditional_expr) {
        // No comments were added after the multi-line conditional expression
        // So let's start a new line with the proper indentation
        result.push_str(NEWLINE);
        result.push_str(&cur_indents);
    } else if result.ends_with(NEWLINE) {
        result.push_str(&cur_indents);
    }
    result.push(')');
    result.push_str(&format_inline_comment(&close_paren, false));

    let open_brace = conditional
        .syntax()
        .children_with_tokens()
        .find(|c| c.kind() == SyntaxKind::OpenBrace)
        .expect("Conditional statement should have an open brace");
    result.push_str(&format_preceding_comments(&open_brace, next_num_indents));
    if result.ends_with(')') {
        result.push(' ');
    } else {
        result.push_str(&cur_indents);
    }
    result.push('{');
    result.push_str(&format_inline_comment(&open_brace, true));

    for statement in conditional.statements() {
        match statement {
            WorkflowStatement::Call(c) => {
                result.push_str(&format_call_statement(c, next_num_indents));
            }
            WorkflowStatement::Conditional(c) => {
                result.push_str(&format_conditional(c, next_num_indents));
            }
            WorkflowStatement::Scatter(s) => {
                result.push_str(&format_scatter(s, next_num_indents));
            }
            WorkflowStatement::Declaration(d) => {
                result.push_str(&format_declaration(&Decl::Bound(d), next_num_indents));
            }
        }
    }

    let close_brace = conditional
        .syntax()
        .children_with_tokens()
        .find(|c| c.kind() == SyntaxKind::CloseBrace)
        .expect("Conditional statement should have a close brace");
    result.push_str(&format_preceding_comments(&close_brace, num_indents));
    if result.ends_with(NEWLINE) {
        result.push_str(&cur_indents);
    } else {
        result.push_str(NEWLINE);
        result.push_str(&cur_indents);
    }
    result.push('}');
    result.push_str(&format_inline_comment(
        &SyntaxElement::Node(conditional.syntax().clone()),
        true,
    ));

    result
}

/// Format a scatter statement
fn format_scatter(scatter: ScatterStatement, num_indents: usize) -> String {
    let mut result = String::new();
    let next_num_indents = num_indents + 1;
    let cur_indents = INDENT.repeat(num_indents);
    let next_indents = INDENT.repeat(next_num_indents);

    result.push_str(&format_preceding_comments(
        &SyntaxElement::Node(scatter.syntax().clone()),
        num_indents,
    ));

    let scatter_keyword = scatter
        .syntax()
        .children_with_tokens()
        .find(|c| c.kind() == SyntaxKind::ScatterKeyword)
        .expect("Scatter statement should have a scatter keyword");
    result.push_str(&cur_indents);
    result.push_str("scatter");
    result.push_str(&format_inline_comment(&scatter_keyword, false));

    let open_paren = scatter
        .syntax()
        .children_with_tokens()
        .find(|c| c.kind() == SyntaxKind::OpenParen)
        .expect("Scatter statement should have an open paren");
    result.push_str(&format_preceding_comments(&open_paren, num_indents));
    if result.ends_with(NEWLINE) {
        result.push_str(&cur_indents);
    } else {
        result.push(' ');
    }
    result.push('(');
    result.push_str(&format_inline_comment(&open_paren, false));

    result.push_str(&format_preceding_comments(
        &SyntaxElement::Token(scatter.variable().syntax().clone()),
        next_num_indents,
    ));
    if result.ends_with(NEWLINE) {
        result.push_str(&next_indents);
    }
    result.push_str(scatter.variable().as_str());
    result.push_str(&format_inline_comment(
        &SyntaxElement::Token(scatter.variable().syntax().clone()),
        false,
    ));

    let in_keyword = scatter
        .syntax()
        .children_with_tokens()
        .find(|c| c.kind() == SyntaxKind::InKeyword)
        .expect("Scatter statement should have an in keyword");
    result.push_str(&format_preceding_comments(&in_keyword, next_num_indents));

    if result.ends_with(NEWLINE) {
        result.push_str(&next_indents);
    } else {
        result.push(' ');
    }
    result.push_str("in");
    result.push_str(&format_inline_comment(&in_keyword, false));

    let mut paren_on_same_line = true;
    let scatter_expr = scatter.expr().syntax().to_string();
    if scatter_expr.contains('\n') {
        paren_on_same_line = false;
    }

    result.push_str(&format_preceding_comments(
        &SyntaxElement::Node(scatter.expr().syntax().clone()),
        next_num_indents,
    ));
    if result.ends_with(NEWLINE) {
        result.push_str(&next_indents);
    } else {
        result.push(' ');
    }
    result.push_str(&scatter_expr); // TODO: format expr
    result.push_str(&format_inline_comment(
        &SyntaxElement::Node(scatter.expr().syntax().clone()),
        false,
    ));

    let close_paren = scatter
        .syntax()
        .children_with_tokens()
        .find(|c| c.kind() == SyntaxKind::CloseParen)
        .expect("Scatter statement should have a close paren");
    result.push_str(&format_preceding_comments(&close_paren, num_indents));
    if !paren_on_same_line && result.ends_with(&scatter_expr) {
        // No comments were added after the scatter expression (which would reset the
        // indentation) So let's start a new line with the proper
        // indentation
        result.push_str(NEWLINE);
        result.push_str(&cur_indents);
    } else if result.ends_with(NEWLINE) {
        result.push_str(&cur_indents);
    }
    result.push(')');
    result.push_str(&format_inline_comment(&close_paren, false));

    let open_brace = scatter
        .syntax()
        .children_with_tokens()
        .find(|c| c.kind() == SyntaxKind::OpenBrace)
        .expect("Scatter statement should have an open brace");
    result.push_str(&format_preceding_comments(&open_brace, next_num_indents));
    if result.ends_with(')') {
        result.push(' ');
    } else {
        result.push_str(&cur_indents);
    }
    result.push('{');
    result.push_str(&format_inline_comment(&open_brace, true));

    for statement in scatter.statements() {
        match statement {
            WorkflowStatement::Call(c) => {
                result.push_str(&format_call_statement(c, next_num_indents));
            }
            WorkflowStatement::Conditional(c) => {
                result.push_str(&format_conditional(c, next_num_indents));
            }
            WorkflowStatement::Scatter(s) => {
                result.push_str(&format_scatter(s, next_num_indents));
            }
            WorkflowStatement::Declaration(d) => {
                result.push_str(&format_declaration(&Decl::Bound(d), next_num_indents));
            }
        }
    }

    let close_brace = scatter
        .syntax()
        .children_with_tokens()
        .find(|c| c.kind() == SyntaxKind::CloseBrace)
        .expect("Scatter statement should have a close brace");
    result.push_str(&format_preceding_comments(&close_brace, num_indents));
    if result.ends_with(NEWLINE) {
        result.push_str(&cur_indents);
    } else {
        result.push_str(NEWLINE);
        result.push_str(&cur_indents);
    }
    result.push('}');
    result.push_str(&format_inline_comment(
        &SyntaxElement::Node(scatter.syntax().clone()),
        true,
    ));

    result
}

/// Format a workflow definition.
pub fn format_workflow(workflow_def: &WorkflowDefinition) -> String {
    let mut result = String::new();
    result.push_str(&format_preceding_comments(
        &SyntaxElement::Node(workflow_def.syntax().clone()),
        0,
    ));
    result.push_str("workflow");
    result.push_str(&format_inline_comment(
        &workflow_def
            .syntax()
            .first_child_or_token()
            .expect("Workflow definition should have a child"),
        false,
    ));

    result.push_str(&format_preceding_comments(
        &SyntaxElement::Token(workflow_def.name().syntax().clone()),
        1,
    ));
    if result.ends_with("workflow") {
        result.push(' ');
    } else {
        result.push_str(INDENT);
    }
    result.push_str(workflow_def.name().as_str());
    result.push_str(&format_inline_comment(
        &SyntaxElement::Token(workflow_def.name().syntax().clone()),
        false,
    ));

    let open_brace = workflow_def
        .syntax()
        .children_with_tokens()
        .find(|c| c.kind() == SyntaxKind::OpenBrace)
        .expect("Workflow definition should have an open brace");
    result.push_str(&format_preceding_comments(&open_brace, 0));
    if !result.ends_with(NEWLINE) {
        result.push(' ');
    }
    result.push('{');
    result.push_str(&format_inline_comment(&open_brace, true));

    let mut meta_section_str = String::new();
    let mut parameter_meta_section_str = String::new();
    let mut input_section_str = String::new();
    let mut body_str = String::new();
    let mut output_section_str = String::new();
    for item in workflow_def.items() {
        match item {
            WorkflowItem::Metadata(m) => {
                meta_section_str.push_str(&format_meta_section(m));
            }
            WorkflowItem::ParameterMetadata(pm) => {
                parameter_meta_section_str.push_str(&format_parameter_meta_section(pm));
            }
            WorkflowItem::Input(i) => {
                input_section_str.push_str(&format_input_section(i));
            }
            WorkflowItem::Output(o) => {
                output_section_str.push_str(&format_output_section(o));
            }
            WorkflowItem::Call(c) => {
                body_str.push_str(&format_call_statement(c, 1));
            }
            WorkflowItem::Conditional(c) => {
                body_str.push_str(&format_conditional(c, 1));
            }
            WorkflowItem::Scatter(s) => {
                body_str.push_str(&format_scatter(s, 1));
            }
            WorkflowItem::Declaration(d) => {
                body_str.push_str(&format_declaration(&Decl::Bound(d), 1));
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
    if !body_str.is_empty() {
        result.push_str(&body_str);
        if !output_section_str.is_empty() {
            result.push_str(NEWLINE);
        }
    }
    if !output_section_str.is_empty() {
        result.push_str(&output_section_str);
    }

    let close_brace = workflow_def
        .syntax()
        .children_with_tokens()
        .find(|c| c.kind() == SyntaxKind::CloseBrace)
        .expect("Workflow definition should have a close brace");
    result.push_str(&format_preceding_comments(&close_brace, 0));
    if !result.ends_with(NEWLINE) {
        result.push_str(NEWLINE);
    }
    result.push('}');
    result.push_str(&format_inline_comment(
        &SyntaxElement::Node(workflow_def.syntax().clone()),
        true,
    ));
    result.push_str(NEWLINE);

    result
}
