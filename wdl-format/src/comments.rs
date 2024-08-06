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
//! by any number of blank lines. Internal blank lines (blanks _between_
//! comments) will be included in the formatted output, but blank lines before
//! or after all comments will not be includes. Multiple internal blank lines in
//! a row will be consolidated to one. Preceding comments should be indented to
//! the same level as the element which they belong to.
//!
//! An inline comment is a comment that appears on the same line as an element,
//! if and only if that element is the last element of its line. Inline comments
//! should always appear two spaces after the element they are commenting on.

use std::collections::VecDeque;

use wdl_ast::AstToken;
use wdl_ast::Comment;
use wdl_ast::SyntaxElement;
use wdl_ast::SyntaxKind;

use super::Formattable;
use super::Formatter;
use super::NEWLINE;

/// Inline comment space constant used for formatting.
///
/// Inline comments should start two spaces after the end of the element they
/// are commenting on.
pub const INLINE_COMMENT_SPACE: &str = "  ";
/// Misplaced preamble comment prefix constant used for formatting.
///
/// Preamble comments should start with two pound signs, but only if they are in
/// the preamble. **No other comments should start with two pound signs.**
/// This constant has a trailing space. This is to prevent trimming a pound sign
/// from the beginning of a comment that is not a preamble comment (such as
/// comments like `####### Section Header #######`).
/// TODO: This could be made smarter. Perhaps a REGEX that looks for a trailing
/// space OR `[a-zA-Z0-9]` could be used to determine if the comment is a
/// preamble comment.
const MISPLACED_PREAMBLE_COMMENT_PREFIX: &str = "## ";

impl Formattable for Comment {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        _formatter: &mut Formatter,
    ) -> std::fmt::Result {
        let comment = self.as_str().trim();
        write!(writer, "{}{}", comment, NEWLINE)
    }
}

/// Format comments that precede a node.
///
/// This function assumes we are _not_ in the preamble, and thus any
/// double-pound-sign comments should be converted to single-pound-sign
/// comments.
pub fn format_preceding_comments<T: std::fmt::Write>(
    element: &SyntaxElement,
    writer: &mut T,
    formatter: &mut Formatter,
    would_be_interrupting: bool,
) -> std::fmt::Result {
    // This walks _backwards_ through the syntax tree to find comments
    // so we must collect them in a vector and later reverse them to get them in the
    // correct order.
    let mut found_text = VecDeque::new();
    let mut inner_buffer = String::new();
    let began_interrupted = formatter.interrupted();
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
                                    formatter.interrupt();
                                }

                                let comment = Comment::cast(
                                    cur.as_token().expect("comment should be a token").clone(),
                                )
                                .expect("comment should cast to a comment");

                                comment.format(&mut inner_buffer, formatter)?;
                                if inner_buffer.starts_with(MISPLACED_PREAMBLE_COMMENT_PREFIX) {
                                    inner_buffer.remove(0);
                                }
                                found_text.push_front(inner_buffer.clone());
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
                // (Two newlines indicates a blank line). Since each comment is
                // followed by a newline, we only insert one newline here.
                if cur.to_string().matches(NEWLINE).count() > 1 && comments_found {
                    inner_buffer.push_str(NEWLINE);
                    found_text.push_front(inner_buffer.clone());
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
    for line in found_text.iter() {
        if line.contains('#') {
            comment_processed = true;
            formatter.indent(writer)?;
        }
        if comment_processed {
            write!(writer, "{}", line)?;
        }
    }
    Ok(())
}

/// Format a comment on the same line as an element.
///
/// A comment is only added if there's no non-trivia between the element and
/// the end of the line.
/// 
/// If 'would_be_interrupting' is false and there is no comment on the line, a
/// newline will be inserted.
pub fn format_inline_comment<T: std::fmt::Write>(
    element: &SyntaxElement,
    writer: &mut T,
    formatter: &mut Formatter,
    would_be_interrupting: bool,
) -> std::fmt::Result {
    let mut next = element.next_sibling_or_token();
    while let Some(cur) = next {
        match cur.kind() {
            SyntaxKind::Comment => {
                write!(writer, "{}", INLINE_COMMENT_SPACE)?;
                let comment =
                    Comment::cast(cur.as_token().expect("comment should be a token").clone())
                        .expect("comment should cast to a comment");
                if would_be_interrupting {
                    formatter.interrupt();
                }
                let mut tmp_buffer = String::new();
                comment.format(&mut tmp_buffer, formatter)?;
                if tmp_buffer.starts_with(MISPLACED_PREAMBLE_COMMENT_PREFIX) {
                    tmp_buffer.remove(0);
                }
                write!(writer, "{}", tmp_buffer)?;
                return Ok(());
            }
            SyntaxKind::Whitespace => {
                if cur.to_string().contains(NEWLINE) {
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
