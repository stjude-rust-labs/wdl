/// Format comments in a WDL file.
/// All comments will be treated as either "preceding" or "inline" comments.
/// A preceding comment is a comment that appears on a line before an element,
/// if and only if that element is the first element of its line. preceding
/// comments should always appear, without any blank lines, immediately before
/// the element they are commenting on. preceding comments should be indented
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
///
/// TODO write more
pub fn format_preceding_comments(
    element: &SyntaxElement,
    num_indents: usize,
    prepend_newline: bool,
) -> String {
    // This walks _backwards_ through the syntax tree to find comments
    // so we must collect them in a vector and later reverse them to get them in the
    // correct order.
    let mut preceding_comments = Vec::new();

    let mut prev = element.prev_sibling_or_token();
    while let Some(cur) = prev {
        match cur.kind() {
            SyntaxKind::Comment => {
                // Ensure this comment "belongs" to the root element.
                // A preceding comment on a blank line is considered to belong to the element.
                // Othewise, the comment "belongs" to whatever
                // else is on that line.
                if let Some(before_cur) = cur.prev_sibling_or_token() {
                    match before_cur.kind() {
                        SyntaxKind::Whitespace => {
                            if before_cur.to_string().contains('\n') {
                                // The 'cur' comment is on is on its own line.
                                // It "belongs" to the current element.
                                let trimmed_comment = cur.clone().to_string().trim().to_owned();
                                preceding_comments.push(trimmed_comment);
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
    if prepend_newline && !preceding_comments.is_empty() {
        result.push_str(NEWLINE);
    }
    for comment in preceding_comments.iter().rev() {
        for _ in 0..num_indents {
            result.push_str(INDENT);
        }
        result.push_str(comment);
        result.push_str(NEWLINE);
    }
    result
}

/// Format a comment on the same line as an element.
///
/// If no comments are found this returns an empty string unless
/// 'newline_needed' is true. If a comment is found, this will return the
/// comment with a newline. If a comment is not found, but 'newline_needed' is
/// true, this will return a newline. Else it will return the empty string.
pub fn format_inline_comment(element: &SyntaxElement, newline_needed: bool) -> String {
    let mut result = String::new();
    let mut next = element.next_sibling_or_token();
    while let Some(cur) = next {
        match cur.kind() {
            SyntaxKind::Comment => {
                result.push_str(INLINE_COMMENT_SPACE);
                result.push_str(cur.to_string().trim());
                result.push_str(NEWLINE);
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
    if result.is_empty() && newline_needed {
        result.push_str(NEWLINE);
    }
    result
}
