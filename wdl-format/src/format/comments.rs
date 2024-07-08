/// Format comments in a WDL file.
/// All comments will be treated as either "preceeding" or "inline" comments.
/// A preceeding comment is a comment that appears on a line before an element,
/// if and only if that element is the first element of its line. Preceeding
/// comments should always appear, without any blank lines, immediately before
/// the element they are commenting on. Preceeding comments should be indented
/// to the same level as the element they are commenting on. An inline
/// comment is a comment that appears on the same line as an element, if and
/// only if that element is the last element of its line. Inline comments should
/// always appear immediately after the element they are commenting on.
use wdl_ast::SyntaxElement;
use wdl_ast::SyntaxKind;

use super::INDENT;
use super::NEWLINE;

/// Inline comment space constant used for formatting.
pub const INLINE_COMMENT_SPACE: &str = "  ";

/// Format comments that preceed a node.
/// If no comments are found this returns an empty string (regardless of the
/// value of 'trailing_indent'). 'trailing_indent' can be used to change the
/// start and end of the string. If 'false', the string will start with
/// 'num_indents' worth of indentation before the first comment and end in a
/// newline. If 'true', the string will not start with any whitespace (i.e.
/// beginning with '#') and will end with 'num_indents' worth of indentation.
pub fn format_preceeding_comments(
    element: &SyntaxElement,
    num_indents: usize,
    trailing_indent: bool,
) -> String {
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
    for (i, comment) in preceeding_comments.iter().rev().enumerate() {
        if (i > 0 && trailing_indent) || !trailing_indent {
            for _ in 0..num_indents {
                result.push_str(INDENT);
            }
        }
        result.push_str(comment);
        result.push_str(NEWLINE);
    }
    if !preceeding_comments.is_empty() && trailing_indent {
        for _ in 0..num_indents {
            result.push_str(INDENT);
        }
    }
    result
}

/// Format a comment on the same line as an element.
/// 'after_comment' is the text to insert _if a comment is found_.
/// 'instead_of_comment' is the text to insert _if no comment is found_.
/// Note that a newline is _always_ inserted after a found comment.
/// If no comments are found and 'instead_of_comment' is empty, this function
/// will return an empty string.
pub fn format_inline_comment(
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
                // Something is between the element and the end of the line
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
