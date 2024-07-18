//! Format comments in a WDL file.
//!
//! All comments will be treated as either "preceding" or "inline" comments.
//! Every comment will "belong" or "be owned" by a specific element in the
//! syntax tree. If that element is moved from one place to another, the
//! comment will move with it. Only syntax elemnts that are either the first
//! element of a line or the last element of a line can own comments. Elemnts
//! may span multiple lines, only the beginning (in the case of preceding
//! comments) or the end (in the case of inline comments) of the element
//! are considered.
//!
//! A preceding comment is a comment that appears on a line before an element.
//! There may be any number of preceding comments and they may be separated
//! by any number of blank lines. All blank lines will be discarded.
//! Preceding comments should be indented to the same level as the element which
//! they belong to.
//!
//! An inline comment is a comment that appears on the same line as an element,
//! if and only if that element is the last element of its line. Inline comments
//! should always appear immediately after the element they are commenting on.

use std::fmt::Write;

use anyhow::Error;
use anyhow::Ok;
use anyhow::Result;
use wdl_ast::AstToken;
use wdl_ast::Comment;
use wdl_ast::SyntaxElement;
use wdl_ast::SyntaxKind;

use super::FormatState;
use super::Formattable;

/// Inline comment space constant used for formatting.
pub const INLINE_COMMENT_SPACE: &str = "  ";

/// Format comments that preceed a node.
pub fn format_preceding_comments(
    element: &SyntaxElement,
    buffer: &mut String,
    state: &mut FormatState,
) -> Result<bool, Error> {
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
                                let comment = Comment::cast(
                                    cur.as_token().expect("Comment should be a token").clone(),
                                )
                                .expect("Comment should cast to a comment");
                                preceding_comments.push(comment);
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

    for comment in preceding_comments.iter().rev() {
        state.indent(buffer)?;
        comment.format(buffer, state)?;
    }
    return Ok(!preceding_comments.is_empty());
}

/// Format a comment on the same line as an element.
pub fn format_inline_comment(
    element: &SyntaxElement,
    buffer: &mut String,
    state: &mut FormatState,
) -> Result<bool, Error> {
    let mut next = element.next_sibling_or_token();
    while let Some(cur) = next {
        match cur.kind() {
            SyntaxKind::Comment => {
                write!(buffer, "{}", INLINE_COMMENT_SPACE)?;
                let comment =
                    Comment::cast(cur.as_token().expect("Comment should be a token").clone())
                        .expect("Comment should cast to a comment");
                comment.format(buffer, state)?;
                return Ok(true);
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
    return Ok(false);
}
