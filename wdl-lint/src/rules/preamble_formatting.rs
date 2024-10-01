//! A lint rule that checks the formatting of the preamble.

use wdl_ast::AstToken;
use wdl_ast::Comment;
use wdl_ast::Diagnostic;
use wdl_ast::Diagnostics;
use wdl_ast::Document;
use wdl_ast::EXCEPT_COMMENT_PREFIX;
use wdl_ast::Span;
use wdl_ast::SupportedVersion;
use wdl_ast::SyntaxKind;
use wdl_ast::VersionStatement;
use wdl_ast::VisitReason;
use wdl_ast::Visitor;
use wdl_ast::Whitespace;

use crate::Rule;
use crate::Tag;
use crate::TagSet;

/// The identifier for the preamble formatting rule.
const ID: &str = "PreambleFormatting";

/// Creates an "invalid preamble comment" diagnostic.
fn invalid_preamble_comment(span: Span) -> Diagnostic {
    Diagnostic::note("preamble comments must start with `##` followed by a space")
        .with_rule(ID)
        .with_highlight(span)
}

/// Creates a "directive after preamble comment" diagnostic.
fn directive_after_preamble_comment(span: Span) -> Diagnostic {
    Diagnostic::note("lint directives must come before preamble comments")
        .with_rule(ID)
        .with_highlight(span)
}

/// Creates an "unnecessary whitespace" diagnostic.
fn unnecessary_whitespace(span: Span) -> Diagnostic {
    Diagnostic::note("unnecessary whitespace in document preamble")
        .with_rule(ID)
        .with_highlight(span)
}

/// Creates an "expected a blank line before" diagnostic.
fn expected_blank_line_before_version(span: Span) -> Diagnostic {
    Diagnostic::note("expected exactly one blank line before the version statement")
        .with_rule(ID)
        .with_highlight(span)
}

/// Creates an "expected a blank line before preamble comment" diagnostic.
fn expected_blank_line_before_preamble_comment(span: Span) -> Diagnostic {
    Diagnostic::note(
        "expected exactly one blank line between lint directives and preamble comments",
    )
    .with_rule(ID)
    .with_highlight(span)
}

/// Detects if a comment is a lint directive.
fn is_lint_directive(text: &str) -> bool {
    text.starts_with(EXCEPT_COMMENT_PREFIX)
}

/// Detects if a comment is a preamble comment.
fn is_preamble_comment(text: &str) -> bool {
    text == "##" || text.starts_with("## ")
}

/// The state of preamble processing.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
enum PreambleState {
    /// The preamble is not being processed.
    #[default]
    Start,
    /// We are processing the lint directive block.
    LintDirectiveBlock,
    /// We are processing the preamble comment block.
    PreambleCommentBlock,
    /// The preamble is finished
    Finished,
}

/// An enum that represents the type of diagnostic to extend.
enum ExtendDiagnostic {
    /// Extend a lint directive diagnostic.
    LintDirective,
    /// Extend an invalid comment diagnostic.
    InvalidComment,
}

/// Detects incorrect comments in a document preamble.
#[derive(Default, Debug, Clone, Copy)]
pub struct PreambleFormattingRule {
    /// The current state of preamble processing.
    state: PreambleState,
    /// The number of comment tokens to skip.
    ///
    /// This is used to skip comments that were consolidated in a prior
    /// diagnostic.
    skip_count: usize,
}

impl Rule for PreambleFormattingRule {
    fn id(&self) -> &'static str {
        ID
    }

    fn description(&self) -> &'static str {
        "Ensures that documents have correct formatting in the preamble."
    }

    fn explanation(&self) -> &'static str {
        "The document preamble is defined as anything before the version declaration statement and \
         the version declaration statement itself. Only comments and whitespace are permitted \
         before the version declaration.
         
         All comments in the preamble should conform to one of two special formats:

            1. \"lint directives\" are special comments that begin with `#@ except:` followed by a \
         comma-delimited list of rule IDs. These comments are used to disable specific lint rules \
         for a specific section of the document. When a lint directive is encountered in the \
         preamble, it will disable the specified rules for the entire document.
            2. double-pound-sign comments (beginning with `##`) are special comments that are used \
         for documentation that doesn't fit within any of the WDL-defined documentation elements \
         (i.e. `meta` and `parameter_meta` sections). These comments may provide context for a \
         collection of tasks or structs, or they may provide a high-level overview of the \
         workflow. We refer to these special double-pound-sign comments as \"preamble comments\". \
         Lint directives are not considered preamble comments.

         Both of these comments are expected to be full line comments (i.e. they should not have \
         any whitespace before the comment).  If lint directives are present, they should be the \
         absolute beginning of the document. Multiple lint directives are permitted, but they \
         should not be interleaved with preamble comments or blank lines.
         
         A space should follow the double-pound-sign if there is any text within the preamble \
         comment. \"Empty\" preamble comments are permitted and should not have any whitespace \
         following the `##`. Comments beginning with 3 or more pound signs before the version \
         declaration are not permitted. All preamble comments should be in a single block without \
         blank lines. Following this block, there should always be a blank line before the version \
         statement.
         
         Both lint directives and preamble comments are optional, and if they are not present, \
         there should be no comments or whitespace before the version declaration."
    }

    fn tags(&self) -> TagSet {
        TagSet::new(&[Tag::Spacing, Tag::Style, Tag::Clarity])
    }

    fn exceptable_nodes(&self) -> Option<&'static [SyntaxKind]> {
        Some(&[SyntaxKind::VersionStatementNode])
    }
}

impl Visitor for PreambleFormattingRule {
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

    fn whitespace(&mut self, state: &mut Self::State, whitespace: &Whitespace) {
        // Since this rule can only be excepted in a document-wide fashion,
        // if the rule is running we can directly add the diagnostic
        // without checking for the exceptable nodes

        if self.state == PreambleState::Finished {
            return;
        }

        // If the next sibling is the version statement, let the VersionFormatting rule
        // handle this particular whitespace
        if whitespace
            .syntax()
            .next_sibling_or_token()
            .map(|s| s.kind() == SyntaxKind::VersionStatementNode)
            .unwrap_or(false)
        {
            return;
        }

        let s = whitespace.as_str();
        // If there is a previous token, it must be a comment
        if let Some(prev_comment) = whitespace.syntax().prev_token() {
            let prev_text = prev_comment.text();
            let prev_is_lint_directive = is_lint_directive(prev_text);
            let prev_is_preamble_comment = is_preamble_comment(prev_text);

            let next_token = whitespace
                .syntax()
                .next_token()
                .expect("should have a next token");
            if next_token.kind() != SyntaxKind::Comment {
                // The next token must be part of the version statement
                // and since we've already established there's a prior comment,
                // this whitespace must be _exactly_ two newlines.
                if s != "\r\n\r\n" && s != "\n\n" {
                    state.add(expected_blank_line_before_version(whitespace.span()));
                }
                return;
            }

            let next_text = next_token.text();
            let next_is_lint_directive = is_lint_directive(next_text);
            let next_is_preamble_comment = is_preamble_comment(next_text);

            let expect_single_blank = match (
                prev_is_lint_directive,
                prev_is_preamble_comment,
                next_is_lint_directive,
                next_is_preamble_comment,
            ) {
                (true, false, true, false) => {
                    // Lint directive followed by lint directive
                    false
                }
                (true, false, false, true) => {
                    // Lint directive followed by preamble comment
                    true
                }
                (false, true, false, true) => {
                    // Preamble comment followed by preamble comment
                    false
                }
                (false, true, true, false) => {
                    // Preamble comment followed by lint directive
                    // Handled by comment visitor
                    return;
                }
                (_, _, false, false) => {
                    // anything followed by invalid comment
                    // Handled by comment visitor
                    return;
                }
                (false, false, ..) => {
                    // Invalid comment followed by anything
                    // Handled by comment visitor
                    return;
                }
                _ => {
                    unreachable!()
                }
            };

            let span = whitespace.span();
            if expect_single_blank {
                if s != "\r\n\r\n" && s != "\n\n" {
                    state.add(expected_blank_line_before_preamble_comment(span));
                }
            } else if s != "\r\n" && s != "\n" {
                // Don't include the newline separating the previous comment from the
                // leading whitespace
                let offset = if s.starts_with("\r\n") {
                    2
                } else if s.starts_with('\n') {
                    1
                } else {
                    0
                };

                state.add(unnecessary_whitespace(Span::new(
                    span.start() + offset,
                    span.len() - offset,
                )));
            } else {
                return;
            }
        } else {
            // Whitespace is not allowed to start the document.
            state.add(unnecessary_whitespace(whitespace.span()));
        }
    }

    fn comment(&mut self, state: &mut Self::State, comment: &Comment) {
        if self.state == PreambleState::Finished {
            return;
        }

        // Skip this comment if necessary; this occurs if we've consolidated multiple
        // comments in a row into a single diagnostic
        if self.skip_count > 0 {
            self.skip_count -= 1;
            return;
        }

        let text = comment.as_str();
        let lint_directive = is_lint_directive(text);
        let preamble_comment = is_preamble_comment(text);

        let mut extend = None;

        if !lint_directive && !preamble_comment {
            extend = Some(ExtendDiagnostic::InvalidComment);
        } else if self.state == PreambleState::Start {
            if lint_directive {
                self.state = PreambleState::LintDirectiveBlock;
            }
            if preamble_comment {
                self.state = PreambleState::PreambleCommentBlock;
            }
            return;
        } else if self.state == PreambleState::LintDirectiveBlock {
            if lint_directive {
                return;
            }
            if preamble_comment {
                // We are switching from the lint directive block to the preamble comment block
                // Whitespace will be handled by the whitespace visitor.
                self.state = PreambleState::PreambleCommentBlock;
                return;
            }
        } else if self.state == PreambleState::PreambleCommentBlock {
            if preamble_comment {
                return;
            }
            if lint_directive {
                extend = Some(ExtendDiagnostic::LintDirective);
            }
        }

        // Otherwise, look for the next siblings that might also be problematic;
        // if so, consolidate them into a single diagnostic
        let mut span = comment.span();
        let mut current = comment.syntax().next_sibling_or_token();
        while let Some(sibling) = current {
            match sibling.kind() {
                SyntaxKind::Comment => {
                    let sibling_text = sibling.as_token().expect("should be a token").text();
                    let sibling_is_lint_directive = is_lint_directive(sibling_text);
                    let sibling_is_preamble_comment = is_preamble_comment(sibling_text);

                    match extend {
                        Some(ExtendDiagnostic::LintDirective) => {
                            if sibling_is_lint_directive {
                                // As we're processing this sibling comment here, increment the skip
                                // count
                                self.skip_count += 1;

                                span = Span::new(
                                    span.start(),
                                    usize::from(sibling.text_range().end()) - span.start(),
                                );
                            } else {
                                // Sibling should not be part of this diagnostic
                                break;
                            }
                        }
                        Some(ExtendDiagnostic::InvalidComment) => {
                            if !sibling_is_lint_directive && !sibling_is_preamble_comment {
                                // As we're processing this sibling comment here, increment the skip
                                // count
                                self.skip_count += 1;

                                span = Span::new(
                                    span.start(),
                                    usize::from(sibling.text_range().end()) - span.start(),
                                );
                            } else {
                                // Sibling should not be part of this diagnostic
                                break;
                            }
                        }
                        None => {
                            unreachable!();
                        }
                    }
                }
                SyntaxKind::Whitespace => {
                    // Skip whitespace
                }
                _ => break,
            }

            current = sibling.next_sibling_or_token();
        }

        // Since this rule can only be excepted in a document-wide fashion,
        // if the rule is running we can directly add the diagnostic
        // without checking for the exceptable nodes
        match extend {
            Some(ExtendDiagnostic::LintDirective) => {
                state.add(directive_after_preamble_comment(span));
            }
            Some(ExtendDiagnostic::InvalidComment) => {
                state.add(invalid_preamble_comment(span));
            }
            None => {
                unreachable!()
            }
        }
    }

    fn version_statement(
        &mut self,
        _state: &mut Self::State,
        reason: VisitReason,
        _stmt: &VersionStatement,
    ) {
        if reason == VisitReason::Exit {
            return;
        }
        self.state = PreambleState::Finished;
    }
}
