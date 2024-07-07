//! A module for formatting WDL code.

use std::collections::HashMap;
use std::result;

use anyhow::bail;
use anyhow::Result;
use wdl_ast::v1::CallStatement;
use wdl_ast::v1::ConditionalStatement;
use wdl_ast::v1::DocumentItem;
use wdl_ast::v1::ImportStatement;
use wdl_ast::v1::InputSection;
use wdl_ast::v1::MetadataSection;
use wdl_ast::v1::ParameterMetadataSection;
use wdl_ast::v1::ScatterStatement;
use wdl_ast::v1::WorkflowDefinition;
use wdl_ast::v1::WorkflowItem;
use wdl_ast::AstChildren;
use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::Direction;
use wdl_ast::Document;
use wdl_ast::SyntaxElement;
use wdl_ast::SyntaxKind;
use wdl_ast::Validator;
use wdl_ast::VersionStatement;

const NEWLINE: &str = "\n";
const INDENT: &str = "    ";
const INLINE_COMMENT_SPACE: &str = "  ";

/// Format comments that preceed a node.
fn format_preceeding_comments(element: &SyntaxElement, num_indents: usize) -> String {
    // This walks _backwards_ through the syntax tree to find comments
    // so we must collect them in a vector and later reverse them to get them in the
    // correct order.
    let mut preceeding_comments = Vec::new();

    let mut prev = element.prev_sibling_or_token();
    while let Some(cur) = prev {
        match cur.kind() {
            SyntaxKind::Comment => {
                // Ensure this comment "belongs" to the root element.
                // A preceeding comment on a blank line is considered to belong to the element.
                // Othewise, the comment "belongs" to whatever
                // else is on that line.
                if let Some(before_cur) = cur.prev_sibling_or_token() {
                    match before_cur.kind() {
                        SyntaxKind::Whitespace => {
                            if before_cur.to_string().contains('\n') {
                                // The 'cur' comment is on is on its own line.
                                // It "belongs" to the current element.
                                let trimmed_comment = cur.clone().to_string().trim().to_owned();
                                preceeding_comments.push(trimmed_comment);
                            }
                        }
                        _ => {
                            // The 'cur' comment is on the same line as this
                            // token. It "belongs"
                            // to whatever is currently being processed.
                        }
                    }
                }
            }
            SyntaxKind::Whitespace => {
                // Ignore
            }
            _ => {
                // We've backed up to non-trivia, so we can stop
                break;
            }
        }
        prev = cur.prev_sibling_or_token()
    }

    let mut result = String::new();
    for comment in preceeding_comments.iter().rev() {
        for _ in 0..num_indents {
            result.push_str(INDENT);
        }
        result.push_str(comment);
        result.push_str(NEWLINE);
    }
    result
}

/// Format a comment on the same line as an element.
/// 'after_comment' is the text to insert _if a comment is found_.
/// 'instead_of_comment' is the text to insert _if no comment is found_.
/// Note that a newline is _always_ inserted after a found comment.
/// If no comments are found and 'instead_of_comment' is empty, this function
/// will return an empty string.
fn format_inline_comment(
    element: &SyntaxElement,
    after_comment: &str,
    instead_of_comment: &str,
) -> String {
    let mut result = String::new();
    let mut next = element.next_sibling_or_token();
    while let Some(cur) = next {
        match cur.kind() {
            SyntaxKind::Comment => {
                result.push_str(INLINE_COMMENT_SPACE);
                result.push_str(cur.to_string().trim());
                result.push_str(NEWLINE);
                result.push_str(after_comment);
                break;
            }
            SyntaxKind::Whitespace => {
                if cur.to_string().contains('\n') {
                    // We've looked ahead past the current line, so we can stop
                    break;
                }
            }
            _ => {
                // Something is between the node and the end of the line
                break;
            }
        }
        next = cur.next_sibling_or_token();
    }
    if result.is_empty() {
        result.push_str(instead_of_comment);
    }
    result
}

/// Format a version statement.
fn format_version_statement(version_statement: VersionStatement) -> String {
    // Collect comments that preceed the version statement
    // Note as this must be the first element in the document,
    // the logic is simpler than the 'format_preceeding_comments' function.
    // We are walking backwards through the syntax tree, so we must collect
    // the comments in a vector and reverse them to get them in the correct order.
    let mut preceeding_comments = Vec::new();
    for sibling in version_statement
        .syntax()
        .siblings_with_tokens(Direction::Prev)
    {
        match sibling.kind() {
            SyntaxKind::Comment => {
                preceeding_comments.push(sibling.to_string().trim().to_owned());
            }
            SyntaxKind::Whitespace => {
                // Ignore
            }
            SyntaxKind::VersionStatementNode => {
                // Ignore the root node
            }
            _ => {
                unreachable!("Unexpected syntax kind: {:?}", sibling.kind());
            }
        }
    }

    let mut result = String::new();
    for comment in preceeding_comments.iter().rev() {
        result.push_str(comment);
        result.push_str(NEWLINE);
    }

    let mut trailing_comment = String::new();
    for child in version_statement.syntax().children_with_tokens() {
        match child.kind() {
            SyntaxKind::VersionKeyword => {
                // This should always be the first child processed
                if !result.is_empty() {
                    // If there are preamble comments, ensure a blank line is inserted
                    result.push_str(NEWLINE);
                }
                result.push_str("version ");
                result.push_str(version_statement.version().as_str());
            }
            SyntaxKind::Comment => {
                // This comment is in the middle of the version statement
                // It will be moved to after the version statement
                trailing_comment.push_str(child.to_string().trim());
                trailing_comment.push_str(NEWLINE);
            }
            SyntaxKind::Whitespace => {
                // Ignore
            }
            SyntaxKind::Version => {
                // Handled by the version keyword
            }
            SyntaxKind::VersionStatementNode => {
                // Ignore the root node
            }
            _ => {
                unreachable!("Unexpected syntax kind: {:?}", child.kind());
            }
        }
    }

    result.push_str(&format_inline_comment(
        &SyntaxElement::Node(version_statement.syntax().clone()),
        "",
        NEWLINE,
    ));
    result.push_str(&trailing_comment);

    result.push_str(NEWLINE);
    result
}

/// Format a list of import statements.
fn format_imports(imports: AstChildren<ImportStatement>) -> String {
    // Collect the imports into a map so we can sort them
    // The key is the "body" of the import statement (which we will sort on)
    // and the value is the formatted import statement _with any found comments_.
    let mut import_map: HashMap<String, String> = HashMap::new();
    for import in imports {
        // TODO: should 'key' get formatted before sorting?
        let key = import.syntax().to_string();
        let mut val = String::new();

        val.push_str(&format_preceeding_comments(
            &SyntaxElement::Node(import.syntax().clone()),
            0,
        ));

        for child in import.syntax().children_with_tokens() {
            match child.kind() {
                SyntaxKind::ImportKeyword => {
                    // This should always be the first child processed
                    val.push_str("import ");
                    let mut next = child.next_sibling_or_token();
                    while let Some(cur) = next {
                        match cur.kind() {
                            SyntaxKind::LiteralStringNode => {
                                cur.as_node().unwrap().children_with_tokens().for_each(
                                    |string_part| match string_part.kind() {
                                        SyntaxKind::DoubleQuote | SyntaxKind::SingleQuote => {
                                            val.push('"');
                                        }
                                        SyntaxKind::LiteralStringText => {
                                            val.push_str(&string_part.to_string());
                                        }
                                        _ => {
                                            unreachable!(
                                                "Unexpected syntax kind: {:?}",
                                                child.kind()
                                            );
                                        }
                                    },
                                );
                            }
                            SyntaxKind::AsKeyword => {
                                val.push_str(" as ");
                            }
                            SyntaxKind::Ident => {
                                val.push_str(&cur.to_string());
                            }
                            SyntaxKind::ImportAliasNode => {
                                cur.as_node().unwrap().children_with_tokens().for_each(
                                    |alias_part| match alias_part.kind() {
                                        SyntaxKind::AliasKeyword => {
                                            // This should always be the first child processed
                                            val.push_str(" alias ");
                                        }
                                        SyntaxKind::Ident => {
                                            val.push_str(&alias_part.to_string());
                                        }
                                        SyntaxKind::AsKeyword => {
                                            val.push_str(" as ");
                                        }
                                        SyntaxKind::ImportAliasNode => {
                                            // Ignore the root node
                                        }
                                        SyntaxKind::Whitespace => {
                                            // Ignore
                                        }
                                        SyntaxKind::Comment => {
                                            // This comment will cause a lint warning
                                            // But we'll include it anyway
                                            if !val.ends_with(" ") {
                                                val.push(' ');
                                            }
                                            val.push(' ');
                                            val.push_str(alias_part.to_string().trim());
                                            val.push_str(NEWLINE);
                                            val.push_str(INDENT);
                                        }
                                        _ => {
                                            unreachable!(
                                                "Unexpected syntax kind: {:?}",
                                                alias_part.kind()
                                            );
                                        }
                                    },
                                );
                            }
                            SyntaxKind::Whitespace => {
                                // Ignore
                            }
                            SyntaxKind::Comment => {
                                // This comment will cause a lint warning
                                // But we'll include it anyway
                                if !val.ends_with(" ") {
                                    val.push(' ');
                                }
                                val.push(' ');
                                val.push_str(cur.to_string().trim());
                                val.push_str(NEWLINE);
                                val.push_str(INDENT);
                            }
                            _ => {
                                unreachable!("Unexpected syntax kind: {:?}", cur.kind());
                            }
                        }
                        next = cur.next_sibling_or_token();
                    }
                }
                SyntaxKind::Whitespace => {
                    // Ignore
                }
                SyntaxKind::ImportStatementNode => {
                    // Ignore the root node
                }
                SyntaxKind::LiteralStringNode
                | SyntaxKind::Comment
                | SyntaxKind::AsKeyword
                | SyntaxKind::Ident
                | SyntaxKind::ImportAliasNode => {
                    // Handled by the import keyword
                }
                _ => {
                    unreachable!("Unexpected syntax kind: {:?}", child.kind());
                }
            }
        }

        val.push_str(&format_inline_comment(
            &SyntaxElement::Node(import.syntax().clone()),
            "",
            NEWLINE,
        ));

        import_map.insert(key, val);
    }

    let mut import_vec: Vec<_> = import_map.into_iter().collect();
    import_vec.sort_by(|a, b| a.0.cmp(&b.0));

    let mut result = String::new();
    for (_, val) in import_vec {
        result.push_str(&val);
    }
    if !result.is_empty() {
        // There should always be a blank line after the imports
        // (if they are present), so add a second newline here.
        result.push_str(NEWLINE);
    }
    result
}

/// Format a meta section.
fn format_meta_section(meta: Option<MetadataSection>) -> String {
    let mut result = String::new();
    let next_indent_level = format!("{}{}", INDENT, INDENT);

    if meta.is_none() {
        // result.push_str(INDENT);
        // result.push_str("meta {");
        // result.push_str(NEWLINE);
        // result.push_str(INDENT);
        // result.push('}');
        // result.push_str(NEWLINE);
        return result;
    }
    let meta = meta.unwrap();

    result.push_str(&format_preceeding_comments(
        &SyntaxElement::Node(meta.syntax().clone()),
        1,
    ));

    for child in meta.syntax().children_with_tokens() {
        match child.kind() {
            SyntaxKind::MetaKeyword => {
                // This should always be the first child processed
                result.push_str(INDENT);
                result.push_str("meta");
                result.push_str(&format_inline_comment(&child, INDENT, " "));
                let mut next = child.next_sibling_or_token();
                while let Some(cur) = next {
                    match cur.kind() {
                        SyntaxKind::OpenBrace => {
                            result.push('{');
                            result.push_str(&format_inline_comment(&cur, "", NEWLINE));
                        }
                        SyntaxKind::Whitespace => {
                            // Ignore
                        }
                        SyntaxKind::Comment => {
                            // This will be handled by a call to either
                            // 'format_preceeding_comments'
                            // or 'format_inline_comment'.
                        }
                        SyntaxKind::MetadataObjectItemNode => {
                            result.push_str(&format_preceeding_comments(&cur, 2));
                            result.push_str(&next_indent_level);
                            result.push_str(&cur.to_string());
                            result.push_str(&format_inline_comment(&cur, "", NEWLINE));
                        }
                        SyntaxKind::CloseBrace => {
                            // Should always be last child processed
                            result.push_str(INDENT);
                            result.push('}');
                            result.push_str(&format_inline_comment(&cur, "", NEWLINE));
                        }
                        _ => {
                            unreachable!("Unexpected syntax kind: {:?}", cur.kind());
                        }
                    }
                    next = cur.next_sibling_or_token();
                }
            }
            SyntaxKind::Whitespace => {
                // Ignore
            }
            SyntaxKind::OpenBrace
            | SyntaxKind::MetadataObjectItemNode
            | SyntaxKind::Comment
            | SyntaxKind::CloseBrace => {
                // Handled by the meta keyword
            }
            _ => {
                unreachable!("Unexpected syntax kind: {:?}", child.kind());
            }
        }
    }

    result.push_str(NEWLINE);
    result
}

/// Format a parameter meta section.
/// TODO: refactor to share code with `format_meta_section`.
fn format_parameter_meta_section(parameter_meta: Option<ParameterMetadataSection>) -> String {
    let mut result = String::new();
    let next_indent_level = format!("{}{}", INDENT, INDENT);

    if parameter_meta.is_none() {
        // result.push_str(INDENT);
        // result.push_str("parameter_meta {");
        // result.push_str(NEWLINE);
        // result.push_str(INDENT);
        // result.push('}');
        // result.push_str(NEWLINE);
        return result;
    }
    let parameter_meta = parameter_meta.unwrap();

    result.push_str(&format_preceeding_comments(
        &SyntaxElement::Node(parameter_meta.syntax().clone()),
        1,
    ));

    for child in parameter_meta.syntax().children_with_tokens() {
        match child.kind() {
            SyntaxKind::ParameterMetaKeyword => {
                // This should always be the first child processed
                result.push_str(INDENT);
                result.push_str("parameter_meta");
                result.push_str(&format_inline_comment(&child, INDENT, " "));
                let mut next = child.next_sibling_or_token();
                while let Some(cur) = next {
                    match cur.kind() {
                        SyntaxKind::OpenBrace => {
                            result.push('{');
                            result.push_str(&format_inline_comment(&cur, "", NEWLINE));
                        }
                        SyntaxKind::Whitespace => {
                            // Ignore
                        }
                        SyntaxKind::Comment => {
                            // This will be handled by a call to either
                            // 'format_preceeding_comments'
                            // or 'format_inline_comment'.
                        }
                        SyntaxKind::MetadataObjectItemNode => {
                            result.push_str(&format_preceeding_comments(&cur, 2));
                            result.push_str(&next_indent_level);
                            result.push_str(&cur.to_string());
                            result.push_str(&format_inline_comment(&cur, "", NEWLINE));
                        }
                        SyntaxKind::CloseBrace => {
                            // Should always be last child processed
                            result.push_str(INDENT);
                            result.push('}');
                            result.push_str(&format_inline_comment(&cur, "", NEWLINE));
                        }
                        _ => {
                            unreachable!("Unexpected syntax kind: {:?}", cur.kind());
                        }
                    }
                    next = cur.next_sibling_or_token();
                }
            }
            SyntaxKind::Whitespace => {
                // Ignore
            }
            SyntaxKind::OpenBrace
            | SyntaxKind::MetadataObjectItemNode
            | SyntaxKind::Comment
            | SyntaxKind::CloseBrace => {
                // Handled by the parameter meta keyword
            }
            _ => {
                unreachable!("Unexpected syntax kind: {:?}", child.kind());
            }
        }
    }

    result.push_str(NEWLINE);
    result
}

/// Format an input section.
fn format_input_section(input: Option<InputSection>) -> String {
    let mut result = String::new();

    if input.is_none() {
        // result.push_str(INDENT);
        // result.push_str("input {");
        // result.push_str(NEWLINE);
        // result.push_str(INDENT);
        // result.push('}');
        // result.push_str(NEWLINE);
        return result;
    }
    let input = input.unwrap();

    result.push_str(&format_preceeding_comments(
        &SyntaxElement::Node(input.syntax().clone()),
        1,
    ));

    result.push_str(INDENT);
    result.push_str("input {");
    result.push_str(&format_inline_comment(
        &input
            .syntax()
            .first_child_or_token()
            .expect("Input section should have a child"),
        "",
        NEWLINE,
    ));

    for item in input.declarations() {
        result.push_str(&format_preceeding_comments(
            &SyntaxElement::Node(item.syntax().clone()),
            2,
        ));
        result.push_str(INDENT);
        result.push_str(INDENT);
        result.push_str(&item.syntax().to_string()); // TODO: Format the declaration
        result.push_str(&format_inline_comment(
            &SyntaxElement::Node(item.syntax().clone()),
            "",
            NEWLINE,
        ));
    }
    result.push_str(INDENT);
    result.push('}');
    result.push_str(&format_inline_comment(
        &input
            .syntax()
            .last_child_or_token()
            .expect("Input section should have a child"),
        "",
        NEWLINE,
    ));

    result.push_str(NEWLINE);
    result
}

fn format_call_statement(call: CallStatement, num_indents: usize) -> String {
    let mut result = String::new();
    let mut cur_indent_level = String::new();
    let mut next_indent_level = String::new();
    for _ in 0..num_indents {
        cur_indent_level.push_str(INDENT);
        next_indent_level.push_str(INDENT);
    }
    next_indent_level.push_str(INDENT);

    result.push_str(&format_preceeding_comments(
        &SyntaxElement::Node(call.syntax().clone()),
        num_indents,
    ));
    result.push_str(&cur_indent_level);
    result.push_str("call");
    result.push_str(&format_inline_comment(
        &call
            .syntax()
            .first_child_or_token()
            .expect("Call statement should have a child"),
        &next_indent_level,
        " ",
    ));

    result.push_str(&call.target().syntax().to_string());
    result.push_str(&format_inline_comment(
        &SyntaxElement::Node(call.target().syntax().clone()),
        &next_indent_level,
        "",
    ));

    if let Some(alias) = call.alias() {
        for child in alias.syntax().children_with_tokens() {
            match child.kind() {
                SyntaxKind::AsKeyword => {
                    // This should always be the first child processed
                    if !result.ends_with(INDENT) {
                        result.push(' ');
                    }
                    result.push_str("as");
                    result.push_str(&format_inline_comment(&child, &next_indent_level, ""))
                }
                SyntaxKind::Ident => {
                    // This will be the last child processed which means it won't have any "next"
                    // siblings. So we go up a level and check if there are
                    // siblings of the 'CallAliasNode'.
                    if !result.ends_with(INDENT) {
                        result.push(' ');
                    }
                    result.push_str(&child.to_string());
                    result.push_str(&format_inline_comment(
                        &SyntaxElement::Node(alias.syntax().clone()),
                        &next_indent_level,
                        "",
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
                    // Any comment on the same line as the 'AsKeyword' or 'Ident'
                    // will be handled by a 'format_inline_comment' call.
                    // Check if this comment is on it's own line. If so, it should be
                    // included in the result.
                    if let Some(before_child) = child.prev_sibling_or_token() {
                        match before_child.kind() {
                            SyntaxKind::Whitespace => {
                                if before_child.to_string().contains('\n') {
                                    if !result.ends_with(INDENT) {
                                        result.push_str(INLINE_COMMENT_SPACE);
                                    }
                                    result.push_str(child.to_string().trim());
                                    result.push_str(NEWLINE);
                                    result.push_str(&next_indent_level);
                                }
                            }
                            _ => {
                                // The comment is on the same line as the
                                // 'AsKeyword' or 'Ident'
                                // and will be handled by a
                                // 'format_inline_comment' call.
                            }
                        }
                    }
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
                    // This should always be the first child processed
                    if !result.ends_with(INDENT) {
                        result.push(' ');
                    }
                    result.push_str("after");
                    result.push_str(&format_inline_comment(&child, &next_indent_level, ""));
                }
                SyntaxKind::Ident => {
                    // This will be the last child processed which means it won't have any "next"
                    // siblings. So we go up a level and check if there are
                    // siblings of the 'CallAfterNode'.
                    if !result.ends_with(INDENT) {
                        result.push(' ');
                    }
                    result.push_str(&child.to_string());
                    result.push_str(&format_inline_comment(
                        &SyntaxElement::Node(after.syntax().clone()),
                        &next_indent_level,
                        "",
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
                    // Any comment on the same line as 'AfterKeyword' or 'Ident'
                    // will be handled by a 'format_inline_comment' call.
                    // Check if this comment is on it's own line. If so, it should be
                    // included in the result.
                    if let Some(before_child) = child.prev_sibling_or_token() {
                        match before_child.kind() {
                            SyntaxKind::Whitespace => {
                                if before_child.to_string().contains('\n') {
                                    if !result.ends_with(INDENT) {
                                        result.push_str(INLINE_COMMENT_SPACE);
                                    }
                                    result.push_str(child.to_string().trim());
                                    result.push_str(NEWLINE);
                                    result.push_str(&next_indent_level);
                                }
                            }
                            _ => {
                                // The comment is on the same line as
                                // 'AfterKeyword' or 'Ident'
                                // and will be handled by a
                                // 'format_inline_comment' call.
                            }
                        }
                    }
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
                if !result.ends_with(INDENT) {
                    result.push(' ');
                }
                result.push('{');
                result.push_str(&format_inline_comment(&child, &next_indent_level, ""));
            }
            SyntaxKind::InputKeyword => {
                if !result.ends_with(INDENT) {
                    result.push(' ');
                }
                result.push_str("input");
                result.push_str(&format_inline_comment(&child, &next_indent_level, ""));
            }
            SyntaxKind::Colon => {
                result.push(':');
                result.push_str(&format_inline_comment(&child, "", NEWLINE));
            }
            SyntaxKind::CallInputItemNode => {
                result.push_str(&format_preceeding_comments(&child, num_indents + 1));
                result.push_str(&next_indent_level);
                result.push_str(&child.to_string());
                result.push_str(&format_inline_comment(&child, "", NEWLINE));
            }
            SyntaxKind::CloseBrace => {
                // Should be last processed
                if !result.ends_with(INDENT) {
                    result.push_str(&cur_indent_level);
                }
                result.push('}');
                result.push_str(&format_inline_comment(
                    &SyntaxElement::Node(call.syntax().clone()),
                    "",
                    NEWLINE,
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
                if let Some(before_child) = child.prev_sibling_or_token() {
                    match before_child.kind() {
                        SyntaxKind::Whitespace => {
                            if before_child.to_string().contains('\n') {
                                if result.ends_with(NEWLINE) {
                                    result.push_str(&next_indent_level);
                                } else if !result.ends_with(INDENT) {
                                    result.push_str(INLINE_COMMENT_SPACE);
                                }
                                result.push_str(child.to_string().trim());
                                result.push_str(NEWLINE);
                            }
                        }
                        _ => {
                            // The comment is on the same line as another
                            // element and will be
                            // handled by a 'format_inline_comment'
                            // call.
                        }
                    }
                }
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

    result.push_str(&format_preceeding_comments(
        &SyntaxElement::Node(conditional.syntax().clone()),
        num_indents,
    ));

    for child in conditional.syntax().children_with_tokens() {
        match child.kind() {
            SyntaxKind::IfKeyword => {
                // This should always be the first child processed
                result.push_str(&cur_indent_level);
                result.push_str("if");
                result.push_str(&format_inline_comment(&child, &next_indent_level, ""));
            }
            SyntaxKind::OpenParen => {
                let mut paren_on_same_line = true;
                if !result.ends_with(INDENT) {
                    result.push(' ');
                }
                result.push('(');
                result.push_str(&format_inline_comment(&child, &next_indent_level, ""));
                let conditional_expr = conditional.expr().syntax().to_string();
                if conditional_expr.contains('\n') {
                    paren_on_same_line = false;
                }
                result.push_str(&conditional_expr);
                result.push_str(&format_inline_comment(
                    &SyntaxElement::Node(conditional.expr().syntax().clone()),
                    &cur_indent_level,
                    "",
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
                result.push_str(&format_inline_comment(&child, &cur_indent_level, ""));
            }
            SyntaxKind::OpenBrace => {
                if result.ends_with(')') {
                    result.push_str(" ");
                }
                result.push('{');
                result.push_str(&format_inline_comment(&child, "", NEWLINE));
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
                                        result.push_str(INLINE_COMMENT_SPACE);
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
                    "",
                    NEWLINE,
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

    result.push_str(&format_preceeding_comments(
        &SyntaxElement::Node(scatter.syntax().clone()),
        num_indents,
    ));

    let mut paren_on_same_line = true;
    for child in scatter.syntax().children_with_tokens() {
        match child.kind() {
            SyntaxKind::ScatterKeyword => {
                // This should always be the first child processed
                result.push_str(&cur_indent_level);
                result.push_str("scatter");
                result.push_str(&format_inline_comment(&child, &next_indent_level, ""));
            }
            SyntaxKind::OpenParen => {
                if !result.ends_with(INDENT) {
                    result.push(' ');
                }
                result.push('(');
                result.push_str(&format_inline_comment(&child, &next_indent_level, ""));
                if !result.ends_with('(') {
                    paren_on_same_line = false;
                }

                result.push_str(scatter.variable().as_str());
                result.push_str(&format_inline_comment(
                    &SyntaxElement::Token(scatter.variable().syntax().clone()),
                    &next_indent_level,
                    "",
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
                result.push_str(&format_inline_comment(&child, &next_indent_level, " "));
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
                    &cur_indent_level,
                    "",
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
                result.push_str(&format_inline_comment(&child, &cur_indent_level, ""));
            }
            SyntaxKind::OpenBrace => {
                if result.ends_with(')') {
                    result.push_str(" ");
                }
                result.push('{');
                result.push_str(&format_inline_comment(&child, "", NEWLINE));
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
                                        result.push_str(INLINE_COMMENT_SPACE);
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
                    "",
                    NEWLINE,
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
fn format_workflow(workflow_def: WorkflowDefinition) -> String {
    let mut result = String::new();
    result.push_str(&format_preceeding_comments(
        &SyntaxElement::Node(workflow_def.syntax().clone()),
        0,
    ));

    result.push_str("workflow ");
    result.push_str(workflow_def.name().as_str());
    result.push_str(" {");
    result.push_str(&format_inline_comment(
        &workflow_def
            .syntax()
            .first_child_or_token()
            .expect("Workflow definition should have a child"),
        "",
        NEWLINE,
    ));

    result.push_str(&format_meta_section(workflow_def.metadata().next()));

    result.push_str(&format_parameter_meta_section(
        workflow_def.parameter_metadata().next(),
    ));

    result.push_str(&format_input_section(workflow_def.inputs().next()));

    let indent_level = 1;
    for item in workflow_def.items() {
        match item {
            WorkflowItem::Call(call) => {
                result.push_str(&format_call_statement(call, indent_level));
            }
            WorkflowItem::Conditional(conditional) => {
                result.push_str(&format_conditional(conditional, indent_level));
            }
            WorkflowItem::Scatter(scatter) => {
                result.push_str(&format_scatter(scatter, indent_level));
            }
            WorkflowItem::Declaration(decl) => {
                // TODO
            }
            WorkflowItem::Metadata(_)
            | WorkflowItem::ParameterMetadata(_)
            | WorkflowItem::Input(_) => {
                // Already handled
            }
            WorkflowItem::Output(_) => {
                // TODO
            }
        }
    }

    result.push('}');
    result.push_str(NEWLINE);
    result
}

/// Format a WDL document.
pub fn format_document(code: &str) -> Result<String> {
    let parse = Document::parse(code).into_result();
    if let Err(diagnostics) = parse {
        for diagnostic in diagnostics.into_iter() {
            eprintln!("{}", diagnostic.message());
        }
        bail!("The document is not valid, so it cannot be formatted.")
    }
    let document = parse.unwrap();
    let validator = Validator::default();
    match validator.validate(&document) {
        Ok(_) => {
            // The document is valid, so we can format it.
        }
        Err(diagnostics) => {
            for diagnostic in diagnostics.into_iter() {
                eprintln!("{}", diagnostic.message());
            }
            bail!("The document is not valid, so it cannot be formatted.")
        }
    }

    let mut result = String::new();
    result.push_str(&format_version_statement(
        document.version_statement().unwrap(),
    ));

    let ast = document.ast();
    let ast = ast.as_v1().unwrap();
    result.push_str(&format_imports(ast.imports()));

    ast.items().for_each(|item| {
        match item {
            DocumentItem::Import(_) => {
                // Imports have already been formatted
            }
            DocumentItem::Workflow(workflow_def) => {
                result.push_str(&format_workflow(workflow_def));
            }
            DocumentItem::Task(task_def) => {
                // TODO: Format the task
            }
            DocumentItem::Struct(struct_def) => {
                // TODO: Format the struct type
            }
        }
    });

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_with_comments() {
        let code = "\n\n    ## preamble comment  \nversion # weird comment\n1.1 # inline \
                    comment\nworkflow test {}";
        let formatted = format_document(code).unwrap();
        assert_eq!(
            formatted,
            "## preamble comment\n\nversion 1.1  # inline comment\n# weird comment\n\nworkflow \
             test {\n}\n"
        );
    }

    #[test]
    fn test_format_without_comments() {
        let code = "version 1.1\nworkflow test {}";
        let formatted = format_document(code).unwrap();
        assert_eq!(formatted, "version 1.1\n\nworkflow test {\n}\n");
    }

    #[test]
    fn test_format_with_imports() {
        let code = "
        version 1.1

        # this comment belongs to fileB
        import \"fileB.wdl\" as foo # also fileB
        import \"fileA.wdl\" as bar # middle of fileA
            alias qux as Qux
        workflow test {}
        # this comment belongs to fileC
        import \"fileC.wdl\"";
        let formatted = format_document(code).unwrap();
        assert_eq!(
            formatted,
            "version 1.1\n\nimport \"fileA.wdl\" as bar  # middle of fileA\n     alias qux as \
             Qux\n# this comment belongs to fileB\nimport \"fileB.wdl\" as foo  # also fileB\n# \
             this comment belongs to fileC\nimport \"fileC.wdl\"\n\nworkflow test {\n}\n"
        );
    }

    #[test]
    fn test_format_with_meta() {
        let code = "
        version 1.1

        workflow test { # workflow comment
        # meta comment
            meta {
        author: \"me\"  # author comment
        # email comment
        email: \"me@stjude.org\"
        }
        }";
        let formatted = format_document(code).unwrap();
        assert_eq!(
            formatted,
            "version 1.1\n\nworkflow test {\n    # meta comment\n    meta {\n        author: \
             \"me\"  # author comment\n        # email comment\n        email: \
             \"me@stjude.org\"\n    }\n\n}\n"
        );
    }

    #[test]
    fn test_format_with_parameter_metadata() {
        let code = "
        version 1.1
        # workflow comment
        workflow test {
            input {
            String foo
            }
        # parameter_meta comment
            parameter_meta { # parameter_meta comment
            foo: \"bar\" # foo comment
            }
    }
        
            ";
        let formatted = format_document(code).unwrap();
        assert_eq!(
            formatted,
            "version 1.1\n\n# workflow comment\nworkflow test {\n    # parameter_meta comment\n    parameter_meta {  # parameter_meta comment\n        foo: \"bar\"  # foo comment\n    }\n\n    input {\n        String foo\n    }\n\n}\n"
        );
    }

    #[test]
    fn test_format_with_inputs() {
        let code = "
        version 1.1

        workflow test {
        input {
        # foo comment
        String foo # another foo comment
        Int # mid-bar comment
        bar
        }
        }";
        let formatted = format_document(code).unwrap();
        assert_eq!(
            formatted,
            "version 1.1\n\nworkflow test {\n    input {\n        # foo comment\n        String \
             foo  # another foo comment\n        Int # mid-bar comment\n        bar\n    }\n\n}\n"
        );
    }

    #[test]
    fn test_format_with_calls() {
        let code = "
        version 1.1

        workflow test {
        # foo comment
        call foo

        # bar comment
        call bar as baz
        call qux # mid-qux inline comment
        # mid-qux full-line comment
        after baz # after qux
        call lorem after ipsum { input: # after input token
        }
        }";
        let formatted = format_document(code).unwrap();
        assert_eq!(
            formatted,
            "version 1.1\n\nworkflow test {\n    # foo comment\n    call foo\n    # bar comment\n    call bar as baz\n    call qux  # mid-qux inline comment\n        after baz  # mid-qux full-line comment\n    call lorem after ipsum { input:  # after input token\n    }\n}\n"
        );
    }

    #[test]
    fn test_format_with_conditionals_and_scatters() {
        let code = "
        version 1.1

        workflow test {
        if (true) {
        call foo
        scatter (abc in bar) {
        if (false) {
        call bar
        }
        if (a >
        b) {
        scatter (x in [
        1, 2, 3
        ]) {
        call baz
    }}
    }}
        }";
        let formatted = format_document(code).unwrap();
        assert_eq!(
            formatted,
            "version 1.1\n\nworkflow test {\n    if (true) {\n        call foo\n    }\n    if \
             (false) {\n        call bar\n    }\n}\n"
        );
    }
}
