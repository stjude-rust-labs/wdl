/// Format a workflow definition.
use wdl_ast::v1::CallStatement;
use wdl_ast::v1::ConditionalStatement;
use wdl_ast::v1::ScatterStatement;
use wdl_ast::v1::WorkflowDefinition;
use wdl_ast::v1::WorkflowItem;
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
        false,
        false,
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
        false,
        false,
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
                    result.push_str(&format_preceding_comments(
                        &child,
                        next_num_indents,
                        false,
                        false,
                    ));
                    if result.ends_with(NEWLINE) {
                        result.push_str(&next_indents);
                    } else {
                        result.push(' ');
                    }
                    result.push_str("as");
                    result.push_str(&format_inline_comment(&child, false))
                }
                SyntaxKind::Ident => {
                    result.push_str(&format_preceding_comments(
                        &child,
                        next_num_indents,
                        false,
                        false,
                    ));
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
                    // If this whitespace contains 2 or more newlines, we should
                    // include a single blank line in the
                    // result let newline_count =
                    // child.to_string().matches(NEWLINE).count();
                    // if newline_count >= 2 {
                    //     result.push_str(&format!("{}{}", NEWLINE, NEWLINE));
                    // }
                }
                SyntaxKind::Comment => {
                    // Any comment on the same line as the 'AsKeyword' or
                    // 'Ident' will be handled by a
                    // 'format_inline_comment' call.
                    // Check if this comment is on it's own line. If so, it
                    // should be included in the result.
                    // if let Some(before_child) = child.prev_sibling_or_token()
                    // {     match before_child.kind() {
                    //         SyntaxKind::Whitespace => {
                    //             if before_child.to_string().contains('\n') {
                    //                 if !result.ends_with(INDENT) {
                    //                     //
                    // result.push_str(INLINE_COMMENT_SPACE);
                    //                 }
                    //
                    // result.push_str(child.to_string().trim());
                    //                 result.push_str(NEWLINE);
                    //                 result.push_str(&next_indents);
                    //             }
                    //         }
                    //         _ => {
                    //             // The comment is on the same line as the
                    //             // 'AsKeyword' or 'Ident'
                    //             // and will be handled by a
                    //             // 'format_inline_comment' call.
                    //         }
                    //     }
                    // }
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
                        false,
                        false,
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
                    result.push_str(&format_preceding_comments(
                        &child,
                        next_num_indents,
                        false,
                        false,
                    ));
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
                    // If this whitespace contains 2 or more newlines, we should
                    // include a single blank line in the
                    // result let newline_count =
                    // child.to_string().matches(NEWLINE).count();
                    // if newline_count >= 2 {
                    //     result.push_str(&format!("{}{}", NEWLINE, NEWLINE));
                    // }
                }
                SyntaxKind::Comment => {
                    // Any comment on the same line as 'AfterKeyword' or 'Ident'
                    // will be handled by a 'format_inline_comment' call.
                    // Check if this comment is on it's own line. If so, it
                    // should be included in the result.
                    // if let Some(before_child) = child.prev_sibling_or_token()
                    // {     match before_child.kind() {
                    //         SyntaxKind::Whitespace => {
                    //             if before_child.to_string().contains('\n') {
                    //                 if !result.ends_with(INDENT) {
                    //                     //
                    // result.push_str(INLINE_COMMENT_SPACE);
                    //                 }
                    //
                    // result.push_str(child.to_string().trim());
                    //                 result.push_str(NEWLINE);
                    //                 result.push_str(&next_indents);
                    //             }
                    //         }
                    //         _ => {
                    //             // The comment is on the same line as
                    //             // 'AfterKeyword' or 'Ident'
                    //             // and will be handled by a
                    //             // 'format_inline_comment' call.
                    //         }
                    //     }
                    // }
                }
                _ => {
                    unreachable!("Unexpected syntax kind: {:?}", child.kind());
                }
            }
        }
    }

    for child in call.syntax().children_with_tokens() {
        match child.kind() {
            SyntaxKind::CallKeyword
            | SyntaxKind::CallTargetNode
            | SyntaxKind::CallAfterNode
            | SyntaxKind::CallAliasNode => {
                // Handled above
            }
            SyntaxKind::OpenBrace => {
                result.push_str(&format_preceding_comments(
                    &child,
                    next_num_indents,
                    false,
                    false,
                ));
                if result.ends_with(NEWLINE) {
                    result.push_str(&next_indents);
                } else {
                    result.push(' ');
                }
                result.push('{');
                result.push_str(&format_inline_comment(&child, false));
            }
            SyntaxKind::InputKeyword => {
                result.push_str(&format_preceding_comments(
                    &child,
                    next_num_indents,
                    false,
                    false,
                ));
                if result.ends_with(NEWLINE) {
                    result.push_str(&next_indents);
                } else {
                    result.push(' ');
                }
                result.push_str("input");
                result.push_str(&format_inline_comment(&child, false));
            }
            SyntaxKind::Colon => {
                result.push_str(&format_preceding_comments(
                    &child,
                    next_num_indents,
                    false,
                    false,
                ));
                if result.ends_with(NEWLINE) {
                    result.push_str(&next_indents);
                }
                result.push(':');
                result.push_str(&format_inline_comment(&child, true));
            }
            SyntaxKind::CallInputItemNode => {
                result.push_str(&format_preceding_comments(
                    &child,
                    next_num_indents,
                    false,
                    false,
                ));
                result.push_str(&next_indents);
                result.push_str(&child.to_string()); // TODO: format inputs
                result.push_str(&format_inline_comment(&child, false));
            }
            SyntaxKind::Comma => {
                result.push_str(&format_preceding_comments(
                    &child,
                    next_num_indents,
                    false,
                    false,
                ));
                result.push_str(",");
                result.push_str(&format_inline_comment(&child, true));
            }
            SyntaxKind::CloseBrace => {
                // Should be last processed if present
                result.push_str(&format_preceding_comments(
                    &child,
                    next_num_indents,
                    false,
                    false,
                ));
                if !result.ends_with(NEWLINE) {
                    result.push_str(NEWLINE);
                }
                result.push_str(&cur_indents);
                result.push('}');
                // inline comments handled outside loop
            }
            SyntaxKind::Whitespace => {
                // If this whitespace contains 2 or more newlines, we should
                // include a single blank line in the result
                // let newline_count =
                // child.to_string().matches(NEWLINE).count();
                // if newline_count >= 2 {
                //     result.push_str(&format!("{}{}", NEWLINE, NEWLINE));
                // }
            }
            SyntaxKind::Comment => {
                // Any comment on the same line as an element in this match
                // statement will be handled by a call to
                // 'format_inline_comment'. Check if this
                // comment is on it's own line. If so, it should be
                // included in the result.
                // if let Some(before_child) = child.prev_sibling_or_token() {
                //     match before_child.kind() {
                //         SyntaxKind::Whitespace => {
                //             if before_child.to_string().contains('\n') {
                //                 if result.ends_with(NEWLINE) {
                //                     result.push_str(&next_indents);
                //                 } else if !result.ends_with(INDENT) {
                //                     // result.push_str(INLINE_COMMENT_SPACE);
                //                 }
                //                 result.push_str(child.to_string().trim());
                //                 result.push_str(NEWLINE);
                //             }
                //         }
                //         _ => {
                //             // The comment is on the same line as another
                //             // element and will be
                //             // handled by a 'format_inline_comment'
                //             // call.
                //         }
                //     }
                // }
            }
            _ => {
                unreachable!("Unexpected syntax kind: {:?}", child.kind());
            }
        }
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
        false,
        false,
    ));

    for child in conditional.syntax().children_with_tokens() {
        match child.kind() {
            SyntaxKind::IfKeyword => {
                // This should always be the first child processed
                result.push_str(&cur_indents);
                result.push_str("if");
                result.push_str(&format_inline_comment(&child, false));
            }
            SyntaxKind::OpenParen => {
                result.push_str(&format_preceding_comments(
                    &child,
                    num_indents,
                    false,
                    false,
                ));
                if result.ends_with(NEWLINE) {
                    result.push_str(&cur_indents);
                } else {
                    result.push(' ');
                }

                let mut paren_on_same_line = true;
                result.push('(');
                result.push_str(&format_inline_comment(&child, false));
                if result.ends_with(NEWLINE) {
                    paren_on_same_line = false;
                    result.push_str(&next_indents);
                }

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
                    .unwrap();

                result.push_str(&format_preceding_comments(
                    &close_paren,
                    num_indents,
                    false,
                    false,
                ));

                if !paren_on_same_line && result.ends_with(&conditional_expr) {
                    // No comments were added after the multi-line conditional expression
                    // So let's start a new line with the proper indentation
                    result.push_str(NEWLINE);
                    result.push_str(&cur_indents)
                } else if result.ends_with(NEWLINE) {
                    result.push_str(&cur_indents);
                }
                result.push(')');
                result.push_str(&format_inline_comment(&close_paren, false));
            }
            SyntaxKind::CloseParen => {
                // Handled by the OpenParen match arm
            }
            SyntaxKind::OpenBrace => {
                result.push_str(&format_preceding_comments(
                    &child,
                    next_num_indents,
                    false,
                    false,
                ));
                if result.ends_with(')') {
                    result.push(' ');
                } else {
                    result.push_str(&cur_indents);
                }
                result.push('{');
                result.push_str(&format_inline_comment(&child, true));
            }
            SyntaxKind::CallStatementNode => {
                result.push_str(&format_call_statement(
                    CallStatement::cast(child.as_node().unwrap().clone()).unwrap(),
                    next_num_indents,
                ));
            }
            SyntaxKind::ConditionalStatementNode => {
                result.push_str(&format_conditional(
                    ConditionalStatement::cast(child.as_node().unwrap().clone()).unwrap(),
                    next_num_indents,
                ));
            }
            SyntaxKind::ScatterStatementNode => {
                result.push_str(&format_scatter(
                    ScatterStatement::cast(child.as_node().unwrap().clone()).unwrap(),
                    next_num_indents,
                ));
            }
            SyntaxKind::BoundDeclNode | SyntaxKind::UnboundDeclNode => {
                // TODO
            }
            SyntaxKind::Whitespace => {
                // If this whitespace contains 2 or more newlines, we should include a single
                // blank line in the result
                let newline_count = child.to_string().matches(NEWLINE).count();
                if newline_count >= 2 {
                    result.push_str(&NEWLINE.repeat(2));
                }
            }
            SyntaxKind::Comment => {
                // Handled by another match arm
            }
            SyntaxKind::CloseBrace => {
                // Should be last processed
                result.push_str(&format_preceding_comments(
                    &child,
                    num_indents,
                    false,
                    false,
                ));
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
            }
            _ => {
                // Should be part of the conditional expression
            }
        }
    }

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
        false,
        false,
    ));

    let mut paren_on_same_line = true;
    for child in scatter.syntax().children_with_tokens() {
        match child.kind() {
            SyntaxKind::ScatterKeyword => {
                // This should always be the first child processed
                result.push_str(&cur_indents);
                result.push_str("scatter");
                result.push_str(&format_inline_comment(&child, false));
            }
            SyntaxKind::OpenParen => {
                result.push_str(&format_preceding_comments(
                    &child,
                    num_indents,
                    false,
                    false,
                ));
                if result.ends_with(NEWLINE) {
                    result.push_str(&cur_indents);
                } else {
                    result.push(' ');
                }
                result.push('(');
                result.push_str(&format_inline_comment(&child, false));
                if !result.ends_with('(') {
                    paren_on_same_line = false;
                    result.push_str(&next_indents);
                }

                result.push_str(&format_preceding_comments(
                    &SyntaxElement::Token(scatter.variable().syntax().clone()),
                    next_num_indents,
                    false,
                    false,
                ));
                result.push_str(scatter.variable().as_str());
                result.push_str(&format_inline_comment(
                    &SyntaxElement::Token(scatter.variable().syntax().clone()),
                    false,
                ));
                if result.ends_with(NEWLINE) {
                    paren_on_same_line = false;
                }
            }
            SyntaxKind::InKeyword => {
                if result.ends_with(NEWLINE) {
                    result.push_str(&next_indents);
                } else {
                    result.push(' ');
                }
                result.push_str("in");
                result.push_str(&format_inline_comment(&child, false));
                if result.ends_with(NEWLINE) {
                    paren_on_same_line = false;
                    result.push_str(&next_indents);
                } else {
                    result.push(' ');
                }

                let scatter_expr = scatter.expr().syntax().to_string();
                if scatter_expr.contains('\n') {
                    paren_on_same_line = false;
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
                    .unwrap();
                result.push_str(&format_preceding_comments(
                    &close_paren,
                    num_indents,
                    false,
                    false,
                ));

                if !paren_on_same_line && result.ends_with(&scatter_expr) {
                    // No comments were added after the scatter expression (which would reset the
                    // indentation) So let's start a new line with the proper
                    // indentation
                    result.push_str(NEWLINE);
                    result.push_str(&cur_indents)
                } else if result.ends_with(NEWLINE) {
                    result.push_str(&cur_indents);
                }
                result.push(')');

                result.push_str(&format_inline_comment(&close_paren, false));
            }
            SyntaxKind::CloseParen => {
                // Handled by the OpenParen match arm
            }
            SyntaxKind::OpenBrace => {
                result.push_str(&format_preceding_comments(
                    &child,
                    next_num_indents,
                    false,
                    false,
                ));
                if result.ends_with(')') {
                    result.push(' ');
                } else {
                    result.push_str(&cur_indents);
                }
                result.push('{');
                result.push_str(&format_inline_comment(&child, true));
            }
            SyntaxKind::CallStatementNode => {
                result.push_str(&format_call_statement(
                    CallStatement::cast(child.as_node().unwrap().clone()).unwrap(),
                    next_num_indents,
                ));
            }
            SyntaxKind::ConditionalStatementNode => {
                result.push_str(&format_conditional(
                    ConditionalStatement::cast(child.as_node().unwrap().clone()).unwrap(),
                    next_num_indents,
                ));
            }
            SyntaxKind::ScatterStatementNode => {
                result.push_str(&format_scatter(
                    ScatterStatement::cast(child.as_node().unwrap().clone()).unwrap(),
                    next_num_indents,
                ));
            }
            SyntaxKind::BoundDeclNode | SyntaxKind::UnboundDeclNode => {
                // TODO
            }
            SyntaxKind::Whitespace => {
                // If this whitespace contains 2 or more newlines, we should include a single
                // blank line in the result
                let newline_count = child.to_string().matches(NEWLINE).count();
                if newline_count >= 2 {
                    result.push_str(&format!("{}{}", NEWLINE, NEWLINE));
                }
            }
            SyntaxKind::Comment => {
                // Any comment on the same line as a prior element in this match
                // statement will be handled by a
                // 'format_inline_comment' call. Check if this
                // comment is on it's own line. If so, it should be
                // included in the result.
                // if let Some(before_child) = child.prev_sibling_or_token() {
                //     match before_child.kind() {
                //         SyntaxKind::Whitespace => {
                //             if before_child.to_string().contains('\n') {
                //                 if !result.ends_with(INDENT) {
                //                     if result.ends_with(NEWLINE) {
                //                         // TODO: Some cases might call for
                // different indentation
                // // How to detect this?
                // result.push_str(&next_indents);
                // } else {                         //
                // result.push_str(INLINE_COMMENT_SPACE);
                //                     }
                //                 }
                //                 result.push_str(child.to_string().trim());
                //                 result.push_str(NEWLINE);
                //             }
                //         }
                //         _ => {
                //             // The comment is on the same line as
                //             // another element
                //             // and will be handled by a
                //             // 'format_inline_comment' call.
                //         }
                //     }
                // }
            }
            SyntaxKind::CloseBrace => {
                // Should be last processed
                result.push_str(&format_preceding_comments(
                    &child,
                    num_indents,
                    false,
                    false,
                ));
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
            }
            _ => {
                // Should be part of the scatter expression
            }
        }
    }

    result
}

/// Format a workflow definition.
pub fn format_workflow(workflow_def: WorkflowDefinition) -> String {
    let mut result = String::new();
    result.push_str(&format_preceding_comments(
        &SyntaxElement::Node(workflow_def.syntax().clone()),
        0,
        false,
        false,
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
        false,
        false,
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
    result.push_str(&format_preceding_comments(&open_brace, 0, false, false));
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
        result.push_str(NEWLINE);
    }
    if !output_section_str.is_empty() {
        result.push_str(&output_section_str);
        result.push_str(NEWLINE);
    }

    let close_brace = workflow_def
        .syntax()
        .children_with_tokens()
        .find(|c| c.kind() == SyntaxKind::CloseBrace)
        .expect("Workflow definition should have a close brace");
    result.push_str(&format_preceding_comments(&close_brace, 0, false, false));
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
