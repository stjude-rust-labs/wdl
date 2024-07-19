//! A lint rule for spacing in comments.

use std::cmp::Ordering;

use regex::Regex;
use wdl_ast::AstToken;
use wdl_ast::Comment;
use wdl_ast::Diagnostic;
use wdl_ast::Diagnostics;
use wdl_ast::Document;
use wdl_ast::Span;
use wdl_ast::SupportedVersion;
use wdl_ast::SyntaxKind;
use wdl_ast::VisitReason;
use wdl_ast::Visitor;

use crate::Rule;
use crate::Tag;
use crate::TagSet;

/// Set indentation string
const INDENT: &str = "    ";

/// The identifier for the comment spacing rule.
const ID: &str = "CommentWhitespace";

/// Creates a diagnostic when an in-line comment is not preceded by two spaces.
fn inline_preceding_whitespace(span: Span) -> Diagnostic {
    Diagnostic::note("in-line comments should be preceded by two spaces")
        .with_rule(ID)
        .with_highlight(span)
        .with_fix("this comment must be preceded with two spaces")
}

/// Creates a diagnostic when the comment token is not followed by a single
/// space.
fn following_whitespace(span: Span) -> Diagnostic {
    Diagnostic::note("comment delimiter should be followed by a single space")
        .with_rule(ID)
        .with_highlight(span)
        .with_fix("follow this comment delimiter with a single space")
}

/// Creates a diagnostic when non-inline comment has insufficient indentation.
fn insufficient_indentation(span: Span, expected: usize, actual: usize) -> Diagnostic {
    Diagnostic::note("comment not sufficiently indented")
        .with_rule(ID)
        .with_highlight(span)
        .with_fix(format!(
            "this comment has {actual} levels of indentation. It should have {expected} levels of \
             indentation."
        ))
}

/// Creates a diagnostic when non-inline comment has excess indentation.
fn excess_indentation(span: Span, expected: usize, actual: usize) -> Diagnostic {
    Diagnostic::note("comment has too much indentation")
        .with_rule(ID)
        .with_highlight(span)
        .with_fix(format!(
            "this comment has {actual} levels of indentation. It should have {expected} levels of \
             indentation."
        ))
}

/// Detects improperly spaced comments.
#[derive(Default, Debug, Clone, Copy)]
pub struct CommentWhitespaceRule;

impl Rule for CommentWhitespaceRule {
    fn id(&self) -> &'static str {
        ID
    }

    fn description(&self) -> &'static str {
        "Ensures that WDL comments have the proper spacing."
    }

    fn explanation(&self) -> &'static str {
        "Comments on the same line as code should have 2 spaces before the # and one space before \
         the comment text. Comments on their own line should match the indentation level around \
         them and have one space between the # and the comment text. Keep in mind that even \
         comments must be kept below the 90 character width limit."
    }

    fn tags(&self) -> TagSet {
        TagSet::new(&[Tag::Spacing])
    }
}

impl Visitor for CommentWhitespaceRule {
    type State = Diagnostics;

    fn document(
        &mut self,
        _: &mut Self::State,
        reason: VisitReason,
        _: &Document,
        _: SupportedVersion,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        // Reset the visitor upon document entry
        *self = Default::default();
    }

    fn comment(&mut self, state: &mut Self::State, comment: &Comment) {
        let re = Regex::new(r"^#+@?").unwrap();
        if is_inline_comment(comment) {
            // check preceding whitespace for two spaces
            if let Some(prior) = comment.syntax().prev_sibling_or_token() {
                if prior.kind() != SyntaxKind::Whitespace || prior.to_string() != "  " {
                    // Report a diagnostic if there are not two spaces before the comment delimiter
                    state.add(inline_preceding_whitespace(comment.span()))
                }
            }
        } else {
            // Not an in-line comment, so check indentation level
            let ancestors = comment.syntax().parent_ancestors().collect::<Vec<_>>();
            let expected_indentation = INDENT.repeat(ancestors.len() - 1);

            if let Some(leading_whitespace) = comment.syntax().prev_sibling_or_token() {
                let this_whitespace = leading_whitespace.to_string();
                let this_indentation = this_whitespace
                    .split('\n')
                    .last()
                    .expect("should have prior whitespace");
                if this_indentation != expected_indentation {
                    // Report a diagnostic if the comment is not indented properly
                    match this_indentation.len().cmp(&expected_indentation.len()) {
                        Ordering::Greater => state.add(excess_indentation(
                            comment.span(),
                            expected_indentation.len() / INDENT.len(),
                            this_indentation.len() / INDENT.len(),
                        )),
                        Ordering::Less => state.add(insufficient_indentation(
                            comment.span(),
                            expected_indentation.len() / INDENT.len(),
                            this_indentation.len() / INDENT.len(),
                        )),
                        Ordering::Equal => {}
                    }
                }
            } else {
                // If there is no prior whitespace, this comment must be at the
                // start of the file.
            }
        }

        // check the comment for one space following the comment delimiter
        if let Some(delimiter) = re.captures(comment.as_str()) {
            let d = delimiter.get(0).unwrap();
            let rest = &comment.as_str()[d.len()..];
            let without_spaces = rest.trim_start_matches(' ');

            if !rest.is_empty() && rest.len() - without_spaces.len() != 1 {
                // Report a diagnostic if there is not one space after the comment delimiter
                state.add(following_whitespace(Span::new(
                    comment.span().start(),
                    d.len(),
                )));
            }
        } else {
            unreachable!("A comment must start with a #")
        }
    }
}

/// Detect is a comment is in-line or not by looking for `\n` in the prior
/// whitespace.
fn is_inline_comment(token: &Comment) -> bool {
    if let Some(prior) = token.syntax().prev_sibling_or_token() {
        return prior.kind() != SyntaxKind::Whitespace || !prior.to_string().contains('\n');
    }
    false
}
