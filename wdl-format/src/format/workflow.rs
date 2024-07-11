/// Format a workflow definition.
use wdl_ast::v1::CallStatement;
use wdl_ast::v1::ConditionalStatement;
use wdl_ast::v1::ScatterStatement;
use wdl_ast::v1::WorkflowDefinition;
use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::SyntaxElement;
use wdl_ast::SyntaxKind;

use super::comments::format_inline_comment;
use super::comments::format_preceding_comments;
use super::format_input_section;
use super::format_meta_section;
use super::format_parameter_meta_section;
use super::INDENT;
use super::NEWLINE;

fn format_call_statement(call: CallStatement, num_indents: usize) -> String {
    let mut result = String::new();
    let mut cur_indent_level = String::new();
    let mut next_indent_level = String::new();
    for _ in 0..num_indents {
        cur_indent_level.push_str(INDENT);
        next_indent_level.push_str(INDENT);
    }
    next_indent_level.push_str(INDENT);

    result.push_str(&format_preceding_comments(
        &SyntaxElement::Node(call.syntax().clone()),
        num_indents,
        false,
        false,
    ));
    result.push_str(&cur_indent_level);
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
        num_indents + 1,
        false,
        true,
    ));
    if result.ends_with("call") {
        result.push(' ');
    } else if result.ends_with(NEWLINE) {
        result.push_str(&next_indent_level);
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
                    result.push_str(&format_preceding_comments(&child, num_indents + 1, false, false));
                    if result.ends_with(NEWLINE) {
                        result.push_str(&next_indent_level);
                    } else {
                        result.push(' ');
                    }
                    result.push_str("as");
                    result.push_str(&format_inline_comment(&child, false))
                }
                SyntaxKind::Ident => {
                    result.push_str(&format_preceding_comments(&child, num_indents + 1, false, false));
                    // This will be the last child processed which means it won't have any "next"
                    // siblings. So we go up a level and check if there are
                    // siblings of the 'CallAliasNode'.
                    if result.ends_with(NEWLINE) {
                        result.push_str(&next_indent_level);
                    } else {
                        result.push(' ');
                    }
                    result.push_str(&child.to_string());
                    result.push_str(&format_inline_comment(
                        &SyntaxElement::Node(alias.syntax().clone()),
                        false,
                    ));
                }
                SyntaxKind::Whitespace => {
                    // If this whitespace contains 2 or more newlines, we should include a single
                    // blank line in the result
                    // let newline_count = child.to_string().matches(NEWLINE).count();
                    // if newline_count >= 2 {
                    //     result.push_str(&format!("{}{}", NEWLINE, NEWLINE));
                    // }
                }
                SyntaxKind::Comment => {
                    // Any comment on the same line as the 'AsKeyword' or 'Ident'
                    // will be handled by a 'format_inline_comment' call.
                    // Check if this comment is on it's own line. If so, it should be
                    // included in the result.
                    // if let Some(before_child) = child.prev_sibling_or_token() {
                    //     match before_child.kind() {
                    //         SyntaxKind::Whitespace => {
                    //             if before_child.to_string().contains('\n') {
                    //                 if !result.ends_with(INDENT) {
                    //                     // result.push_str(INLINE_COMMENT_SPACE);
                    //                 }
                    //                 result.push_str(child.to_string().trim());
                    //                 result.push_str(NEWLINE);
                    //                 result.push_str(&next_indent_level);
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
                    result.push_str(&format_preceding_comments(&child, num_indents + 1, false, false));
                    if result.ends_with(NEWLINE) {
                        result.push_str(&next_indent_level);
                    } else {
                        result.push(' ');
                    }
                    result.push_str("after");
                    result.push_str(&format_inline_comment(&child, false));
                }
                SyntaxKind::Ident => {
                    result.push_str(&format_preceding_comments(&child, num_indents + 1, false, false));
                    // This will be the last child processed which means it won't have any "next"
                    // siblings. So we go up a level and check if there are
                    // siblings of the 'CallAfterNode'.
                    if result.ends_with(NEWLINE) {
                        result.push_str(&next_indent_level);
                    } else {
                        result.push(' ');
                    }
                    result.push_str(&child.to_string());
                    result.push_str(&format_inline_comment(
                        &SyntaxElement::Node(after.syntax().clone()),
                        false,
                    ));
                }
                SyntaxKind::Whitespace => {
                    // If this whitespace contains 2 or more newlines, we should include a single
                    // blank line in the result
                    // let newline_count = child.to_string().matches(NEWLINE).count();
                    // if newline_count >= 2 {
                    //     result.push_str(&format!("{}{}", NEWLINE, NEWLINE));
                    // }
                }
                SyntaxKind::Comment => {
                    // Any comment on the same line as 'AfterKeyword' or 'Ident'
                    // will be handled by a 'format_inline_comment' call.
                    // Check if this comment is on it's own line. If so, it should be
                    // included in the result.
                    // if let Some(before_child) = child.prev_sibling_or_token() {
                    //     match before_child.kind() {
                    //         SyntaxKind::Whitespace => {
                    //             if before_child.to_string().contains('\n') {
                    //                 if !result.ends_with(INDENT) {
                    //                     // result.push_str(INLINE_COMMENT_SPACE);
                    //                 }
                    //                 result.push_str(child.to_string().trim());
                    //                 result.push_str(NEWLINE);
                    //                 result.push_str(&next_indent_level);
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
                result.push_str(&format_preceding_comments(&child, num_indents + 1, false, false));
                if result.ends_with(NEWLINE) {
                    result.push_str(&next_indent_level);
                } else {
                    result.push(' ');
                }
                result.push('{');
                result.push_str(&format_inline_comment(&child, true));
            }
            SyntaxKind::InputKeyword => {
                result.push_str(&format_preceding_comments(&child, num_indents + 1, false, false));
                if result.ends_with(NEWLINE) {
                    result.push_str(&next_indent_level);
                } else {
                    result.push(' ');
                }
                result.push_str("input");
                result.push_str(&format_inline_comment(&child, false));
            }
            SyntaxKind::Colon => {
                result.push_str(&format_preceding_comments(&child, num_indents + 1, false, false));
                if result.ends_with(NEWLINE) {
                    result.push_str(&next_indent_level);
                }
                result.push(':');
                result.push_str(&format_inline_comment(&child, false));
            }
            SyntaxKind::CallInputItemNode => {
                result.push_str(&format_preceding_comments(&child, num_indents + 1, false, false));
                result.push_str(&next_indent_level);
                result.push_str(&child.to_string());
                result.push_str(&format_inline_comment(&child, true));
            }
            SyntaxKind::CloseBrace => {
                // Should be last processed
                result.push_str(&format_preceding_comments(&child, num_indents + 1, false, false));
                if !result.ends_with(NEWLINE) {
                    result.push_str(NEWLINE);
                }
                result.push_str(&cur_indent_level);
                result.push('}');
                result.push_str(&format_inline_comment(
                    &SyntaxElement::Node(call.syntax().clone()),
                    true,
                ));
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
                // Any comment on the same line as an element in this match statement
                // will be handled by a call to 'format_inline_comment'.
                // Check if this comment is on it's own line. If so, it should be
                // included in the result.
                // if let Some(before_child) = child.prev_sibling_or_token() {
                //     match before_child.kind() {
                //         SyntaxKind::Whitespace => {
                //             if before_child.to_string().contains('\n') {
                //                 if result.ends_with(NEWLINE) {
                //                     result.push_str(&next_indent_level);
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
    if !result.ends_with(NEWLINE) {
        result.push_str(NEWLINE);
    }

    result
}

/// Format a conditional statement.
fn format_conditional(conditional: ConditionalStatement, num_indents: usize) -> String {
    let mut result = String::new();
    let mut cur_indent_level = String::new();
    let mut next_indent_level = String::new();
    for _ in 0..num_indents {
        cur_indent_level.push_str(INDENT);
        next_indent_level.push_str(INDENT);
    }
    next_indent_level.push_str(INDENT);

    // result.push_str(&format_preceding_comments(
    //     &SyntaxElement::Node(conditional.syntax().clone()),
    //     num_indents,
    // ));

    for child in conditional.syntax().children_with_tokens() {
        match child.kind() {
            SyntaxKind::IfKeyword => {
                // This should always be the first child processed
                result.push_str(&cur_indent_level);
                result.push_str("if");
                // result.push_str(&format_inline_comment(&child,
                // &next_indent_level, ""));
            }
            SyntaxKind::OpenParen => {
                let mut paren_on_same_line = true;
                if !result.ends_with(INDENT) {
                    result.push(' ');
                }
                result.push('(');
                // result.push_str(&format_inline_comment(&child, &next_indent_level, ""));
                let conditional_expr = conditional.expr().syntax().to_string();
                if conditional_expr.contains('\n') {
                    paren_on_same_line = false;
                }
                result.push_str(&conditional_expr);
                result.push_str(&format_inline_comment(
                    &SyntaxElement::Node(conditional.expr().syntax().clone()),
                    false,
                ));
                if !paren_on_same_line && result.ends_with(&conditional_expr) {
                    // No comments were added after the multi-line conditional expression
                    // So let's start a new line with the proper indentation
                    result.push_str(NEWLINE);
                    result.push_str(&cur_indent_level)
                }
                result.push(')');
            }
            SyntaxKind::CloseParen => {
                // A close paren was added by the OpenParen match arm
                // But comments of that token will be handled here
                // result.push_str(&format_inline_comment(&child,
                // &cur_indent_level, ""));
            }
            SyntaxKind::OpenBrace => {
                if result.ends_with(')') {
                    result.push_str(" ");
                }
                result.push('{');
                // result.push_str(&format_inline_comment(&child, "", NEWLINE));
            }
            SyntaxKind::CallStatementNode => {
                result.push_str(&format_call_statement(
                    CallStatement::cast(child.as_node().unwrap().clone()).unwrap(),
                    num_indents + 1,
                ));
            }
            SyntaxKind::ConditionalStatementNode => {
                result.push_str(&format_conditional(
                    ConditionalStatement::cast(child.as_node().unwrap().clone()).unwrap(),
                    num_indents + 1,
                ));
            }
            SyntaxKind::ScatterStatementNode => {
                result.push_str(&format_scatter(
                    ScatterStatement::cast(child.as_node().unwrap().clone()).unwrap(),
                    num_indents + 1,
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
                // Any comment on the same line as a prior element in this match statement
                // will be handled by a 'format_inline_comment' call.
                // Check if this comment is on it's own line. If so, it should be
                // included in the result.
                if let Some(before_child) = child.prev_sibling_or_token() {
                    match before_child.kind() {
                        SyntaxKind::Whitespace => {
                            if before_child.to_string().contains('\n') {
                                if !result.ends_with(INDENT) {
                                    if result.ends_with(NEWLINE) {
                                        // TODO: Some cases might call for different indentation
                                        // How to detect this?
                                        result.push_str(&next_indent_level);
                                    } else {
                                        // result.push_str(INLINE_COMMENT_SPACE);
                                    }
                                }
                                result.push_str(child.to_string().trim());
                                result.push_str(NEWLINE);
                            }
                        }
                        _ => {
                            // The comment is on the same line as
                            // another element
                            // and will be handled by a
                            // 'format_inline_comment' call.
                        }
                    }
                }
            }
            SyntaxKind::CloseBrace => {
                // Should be last processed
                if result.ends_with(NEWLINE) {
                    result.push_str(&cur_indent_level);
                } else {
                    result.push_str(NEWLINE);
                    result.push_str(&cur_indent_level);
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
    let mut cur_indent_level = String::new();
    let mut next_indent_level = String::new();
    for _ in 0..num_indents {
        cur_indent_level.push_str(INDENT);
        next_indent_level.push_str(INDENT);
    }
    next_indent_level.push_str(INDENT);

    // result.push_str(&format_preceding_comments(
    //     &SyntaxElement::Node(scatter.syntax().clone()),
    //     num_indents,
    // ));

    let mut paren_on_same_line = true;
    for child in scatter.syntax().children_with_tokens() {
        match child.kind() {
            SyntaxKind::ScatterKeyword => {
                // This should always be the first child processed
                result.push_str(&cur_indent_level);
                result.push_str("scatter");
                // result.push_str(&format_inline_comment(&child,
                // &next_indent_level, ""));
            }
            SyntaxKind::OpenParen => {
                if !result.ends_with(INDENT) {
                    result.push(' ');
                }
                result.push('(');
                // result.push_str(&format_inline_comment(&child, &next_indent_level, ""));
                if !result.ends_with('(') {
                    paren_on_same_line = false;
                }

                result.push_str(scatter.variable().as_str());
                result.push_str(&format_inline_comment(
                    &SyntaxElement::Token(scatter.variable().syntax().clone()),
                    false,
                ));
                if !result.ends_with(&scatter.variable().as_str()) {
                    paren_on_same_line = false;
                }
            }
            SyntaxKind::InKeyword => {
                if !result.ends_with(INDENT) {
                    result.push(' ');
                }
                result.push_str("in");
                // result.push_str(&format_inline_comment(&child, &next_indent_level, " "));
                if result.ends_with(INDENT) {
                    paren_on_same_line = false;
                }

                let scatter_expr = scatter.expr().syntax().to_string();
                if scatter_expr.contains('\n') {
                    paren_on_same_line = false;
                }
                result.push_str(&scatter_expr);
                result.push_str(&format_inline_comment(
                    &SyntaxElement::Node(scatter.expr().syntax().clone()),
                    false,
                ));
                if !paren_on_same_line && result.ends_with(&scatter_expr) {
                    // No comments were added after the scatter expression (which would reset the
                    // indentation) So let's start a new line with the proper
                    // indentation
                    result.push_str(NEWLINE);
                    result.push_str(&cur_indent_level)
                }
                result.push(')');
            }
            SyntaxKind::CloseParen => {
                // A close paren was added by the InKeyword match arm
                // But comments of that token will be handled here
                // result.push_str(&format_inline_comment(&child,
                // &cur_indent_level, ""));
            }
            SyntaxKind::OpenBrace => {
                if result.ends_with(')') {
                    result.push_str(" ");
                }
                result.push('{');
                // result.push_str(&format_inline_comment(&child, "", NEWLINE));
            }
            SyntaxKind::CallStatementNode => {
                result.push_str(&format_call_statement(
                    CallStatement::cast(child.as_node().unwrap().clone()).unwrap(),
                    num_indents + 1,
                ));
            }
            SyntaxKind::ConditionalStatementNode => {
                result.push_str(&format_conditional(
                    ConditionalStatement::cast(child.as_node().unwrap().clone()).unwrap(),
                    num_indents + 1,
                ));
            }
            SyntaxKind::ScatterStatementNode => {
                result.push_str(&format_scatter(
                    ScatterStatement::cast(child.as_node().unwrap().clone()).unwrap(),
                    num_indents + 1,
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
                // Any comment on the same line as a prior element in this match statement
                // will be handled by a 'format_inline_comment' call.
                // Check if this comment is on it's own line. If so, it should be
                // included in the result.
                if let Some(before_child) = child.prev_sibling_or_token() {
                    match before_child.kind() {
                        SyntaxKind::Whitespace => {
                            if before_child.to_string().contains('\n') {
                                if !result.ends_with(INDENT) {
                                    if result.ends_with(NEWLINE) {
                                        // TODO: Some cases might call for different indentation
                                        // How to detect this?
                                        result.push_str(&next_indent_level);
                                    } else {
                                        // result.push_str(INLINE_COMMENT_SPACE);
                                    }
                                }
                                result.push_str(child.to_string().trim());
                                result.push_str(NEWLINE);
                            }
                        }
                        _ => {
                            // The comment is on the same line as
                            // another element
                            // and will be handled by a
                            // 'format_inline_comment' call.
                        }
                    }
                }
            }
            SyntaxKind::CloseBrace => {
                // Should be last processed
                if result.ends_with(NEWLINE) {
                    result.push_str(&cur_indent_level);
                } else {
                    result.push_str(NEWLINE);
                    result.push_str(&cur_indent_level);
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

    let mut meta_section_str = String::new();
    let mut parameter_meta_section_str = String::new();
    let mut input_section_str = String::new();
    let mut body_str = String::new();
    let mut output_section_str = String::new();
    let mut closing_brace = String::new();
    for child in workflow_def.syntax().children_with_tokens() {
        match child.kind() {
            SyntaxKind::WorkflowKeyword => {
                // This should always be the first child processed
                result.push_str("workflow");
                result.push_str(&format_inline_comment(&child, false));
                result.push_str(&format_preceding_comments(
                    &SyntaxElement::Token(workflow_def.name().syntax().clone()),
                    1,
                    true,
                    false,
                ));
                if result.ends_with("workflow") {
                    result.push(' ');
                } else if result.ends_with(NEWLINE) {
                    result.push_str(INDENT);
                }
                result.push_str(workflow_def.name().as_str());
                result.push_str(&format_inline_comment(
                    &SyntaxElement::Token(workflow_def.name().syntax().clone()),
                    false,
                ));
            }
            SyntaxKind::Ident => {
                // This is the name of the workflow
                // It's handled by the WorkflowKeyword match arm
            }
            SyntaxKind::OpenBrace => {
                result.push_str(&format_preceding_comments(&child, 0, false, false));
                if !result.ends_with(NEWLINE) {
                    result.push(' ');
                }
                result.push('{');
                result.push_str(&format_inline_comment(&child, true));
            }
            SyntaxKind::CloseBrace => {
                result.push_str(&format_preceding_comments(&child, 0, false, false));
                if !result.ends_with(NEWLINE) {
                    result.push_str(NEWLINE);
                }
                closing_brace.push('}');

                // Should always be last child processed
                // As such any comments on the same line as this token
                // will not be siblings of the 'child' element,
                // so we will go up a level and check the siblings of the
                // 'WorkflowDefinitionNode'.
                closing_brace.push_str(&format_inline_comment(
                    &SyntaxElement::Node(workflow_def.syntax().clone()),
                    true,
                ));
                closing_brace.push_str(NEWLINE); // should always be two newlines
            }
            SyntaxKind::MetadataSectionNode => {
                meta_section_str.push_str(&format_meta_section(workflow_def.metadata().next()));
            }
            SyntaxKind::ParameterMetadataSectionNode => {
                parameter_meta_section_str.push_str(&format_parameter_meta_section(
                    workflow_def.parameter_metadata().next(),
                ));
            }
            SyntaxKind::InputSectionNode => {
                input_section_str.push_str(&format_input_section(workflow_def.inputs().next()));
            }
            SyntaxKind::OutputSectionNode => {
                // TODO
            }
            SyntaxKind::CallStatementNode => {
                body_str.push_str(&format_call_statement(
                    CallStatement::cast(child.as_node().unwrap().clone()).unwrap(),
                    1,
                ));
            }
            SyntaxKind::ConditionalStatementNode => {
                body_str.push_str(&format_conditional(
                    ConditionalStatement::cast(child.as_node().unwrap().clone()).unwrap(),
                    1,
                ));
            }
            SyntaxKind::ScatterStatementNode => {
                body_str.push_str(&format_scatter(
                    ScatterStatement::cast(child.as_node().unwrap().clone()).unwrap(),
                    1,
                ));
            }
            SyntaxKind::BoundDeclNode | SyntaxKind::UnboundDeclNode => {
                // TODO
            }
            SyntaxKind::Whitespace => {
                // How to respect blank lines?
            }
            SyntaxKind::Comment => {
                // This comment should be handled by another match arm
            }
            _ => {
                unreachable!("Unexpected syntax kind: {:?}", child.kind());
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
    result.push_str(&closing_brace);

    result
}
