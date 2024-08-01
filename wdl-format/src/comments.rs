//! Format comments in a WDL file.
//!
//! All comments will be treated as either "preceding" or "inline" comments.
//! Every comment will "belong" or "be owned" by a specific element in the
//! syntax tree. If that element is moved from one place to another, the
//! comment will move with it. Only syntax elements that are either the first
//! element of a line or the last element of a line can own comments. Elements
//! may span multiple lines, only the beginning (in the case of preceding
//! comments) or the end (in the case of inline comments) of the element
//! are considered.
//!
//! A preceding comment is a comment that appears on a line before an element.
//! There may be any number of preceding comments and they may be separated
//! by any number of blank lines. Blank lines will be included in the formatted
//! output, but multiple blank lines in a row will be consolidated to one.
//! Preceding comments should be indented to the same level as the element which
//! they belong to.
//!
//! An inline comment is a comment that appears on the same line as an element,
//! if and only if that element is the last element of its line. Inline comments
//! should always appear immediately after the element they are commenting on.

use wdl_ast::AstToken;
use wdl_ast::Comment;
use wdl_ast::SyntaxElement;
use wdl_ast::SyntaxKind;

use super::Formattable;
use super::State;
use super::NEWLINE;

/// Inline comment space constant used for formatting.
pub const INLINE_COMMENT_SPACE: &str = "  ";

impl Formattable for Comment {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, _state: &mut State) -> std::fmt::Result {
        let comment = self.as_str().trim();
        write!(writer, "{}{}", comment, NEWLINE)
    }
}

/// Format comments that preceed a node.
pub fn format_preceding_comments<T: std::fmt::Write>(
    element: &SyntaxElement,
    writer: &mut T,
    state: &mut State,
    would_be_interrupting: bool,
) -> std::fmt::Result {
    // This walks _backwards_ through the syntax tree to find comments
    // so we must collect them in a vector and later reverse them to get them in the
    // correct order.
    let mut reversed_text = Vec::new();
    let mut inner_buffer = String::new();
    let began_interrupted = state.interrupted();
    let mut comments_found = false;

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
                                comments_found = true;

                                if would_be_interrupting {
                                    state.interrupt();
                                }

                                let comment = Comment::cast(
                                    cur.as_token().expect("Comment should be a token").clone(),
                                )
                                .expect("Comment should cast to a comment");

                                state.indent(&mut inner_buffer)?;
                                comment.format(&mut inner_buffer, state)?;
                                reversed_text.push(inner_buffer.clone());
                                inner_buffer.clear();
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
                // Ignore whitespace until a comment is found.
                // Ignore whitespace that doesn't contain at least two newlines.
                // (Two newlines indicates a blank line)
                if cur.to_string().matches(NEWLINE).count() > 1 && comments_found {
                    inner_buffer.push_str(NEWLINE);
                    reversed_text.push(inner_buffer.clone());
                    inner_buffer.clear();
                }
            }
            _ => {
                // We've backed up to non-trivia, so we can stop
                break;
            }
        }
        prev = cur.prev_sibling_or_token()
    }

    if comments_found && would_be_interrupting && !began_interrupted {
        write!(writer, "{}", NEWLINE)?;
    }

    // Skip any whitespace before comments start.
    let mut comment_processed = false;
    for line in reversed_text.iter().rev() {
        if line.contains('#') {
            comment_processed = true;
        }
        if comment_processed {
            write!(writer, "{}", line)?;
        }
    }
    Ok(())
}

/// Format a comment on the same line as an element.
pub fn format_inline_comment<T: std::fmt::Write>(
    element: &SyntaxElement,
    writer: &mut T,
    state: &mut State,
    would_be_interrupting: bool,
) -> std::fmt::Result {
    let mut next = element.next_sibling_or_token();
    while let Some(cur) = next {
        match cur.kind() {
            SyntaxKind::Comment => {
                write!(writer, "{}", INLINE_COMMENT_SPACE)?;
                let comment =
                    Comment::cast(cur.as_token().expect("Comment should be a token").clone())
                        .expect("Comment should cast to a comment");
                if would_be_interrupting {
                    state.interrupt();
                }
                comment.format(writer, state)?;
                return Ok(());
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

    if !would_be_interrupting {
        write!(writer, "{}", NEWLINE)?;
    }
    Ok(())
}
