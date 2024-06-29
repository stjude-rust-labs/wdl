//! A lint rule for spacing of call inputs.

use std::borrow::Borrow;

use wdl_ast::v1::CallStatement;
use wdl_ast::AstNode;
use wdl_ast::Diagnostic;
use wdl_ast::Diagnostics;
use wdl_ast::Span;
use wdl_ast::SyntaxKind;
use wdl_ast::ToSpan;
use wdl_ast::VisitReason;
use wdl_ast::Visitor;

use crate::Rule;
use crate::Tag;
use crate::TagSet;

/// The identifier for the input not sorted rule.
const ID: &str = "CallInputSpacing";

/// Creates a input spacing diagnostic.
fn call_input_keyword_spacing(span: Span) -> Diagnostic {
    Diagnostic::warning("call input keyword not properly spaced")
        .with_rule(ID)
        .with_highlight(span)
        .with_fix(
            "input keyword must be separated from the opening brace (\"{\") by a single space"
        )
}

/// Creates an input call spacing diagnostic.
fn call_input_missing_newline(span: Span) -> Diagnostic {
    Diagnostic::warning("call inputs must be separated by newline")
        .with_rule(ID)
        .with_highlight(span)
        .with_fix("add newline before the input")
}

/// Creates call input assignment diagnostic.
fn call_input_assignment(span: Span) -> Diagnostic {
    Diagnostic::warning("call inputs assignments must be surrounded with whitespace")
        .with_rule(ID)
        .with_highlight(span)
        .with_fix("surround '=' with whitespace on each side")
}

/// Detects unsorted input declarations.
#[derive(Debug, Clone, Copy)]
pub struct CallInputSpacingRule;

impl Rule for CallInputSpacingRule {
    fn id(&self) -> &'static str {
        ID
    }

    fn description(&self) -> &'static str {
        "Ensures that call inputs are spaced appropriately."
    }

    fn explanation(&self) -> &'static str {
        "When making calls from a workflow, it is more readable and easier to edit if the supplied \
         inputs are each on their own line. When there is more than one input to a call statement, \
         the `input:` keyword should follow the opening brace ({) and a single space, then each \
         input specification should occupy its own line. This does inflate the line count of a WDL \
         document, but it is worth it for the consistent readability. An exception can be made \
         (but does not have to be made), for calls with only a single parameter. In those cases, \
         it is permissable to keep the input on the same line as the call."
    }

    fn tags(&self) -> TagSet {
        TagSet::new(&[Tag::Style, Tag::Clarity, Tag::Spacing])
    }
}

impl Visitor for CallInputSpacingRule {
    type State = Diagnostics;

    fn call_statement(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        call: &CallStatement,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        let inputs = call.inputs().count();

        if inputs == 0 {
            return;
        }

        // Check for "{ input:" spacing
        let mut spacing_errors = 0;
        let input_keyword = call
            .syntax()
            .children_with_tokens()
            .find(|c| c.kind() == SyntaxKind::InputKeyword);

        if let Some(rbrace) = call
            .syntax()
            .children_with_tokens()
            .find(|c| c.kind() == SyntaxKind::OpenBrace)
        {
            match input_keyword.borrow() {
                Some(t) => {
                    if let Some(next) = rbrace.next_sibling_or_token() {
                        if next.as_token().expect("should be a token").text().eq(" ") {
                            if let Some(second) = next.next_sibling_or_token() {
                                if second != *t {
                                    spacing_errors += 1;
                                }
                            }
                        } else {
                            spacing_errors += 1;
                        }
                    }
                }
                None => {
                    // input keyword will be optional in the future
                }
            }
            if spacing_errors > 0 {
                state.add(call_input_keyword_spacing(Span::new(
                    rbrace.text_range().to_span().start(),
                    input_keyword.unwrap().text_range().to_span().start()
                        - rbrace.text_range().to_span().end()
                        + 1,
                )));
            }
        }

        call.inputs().for_each(|input| {
            // Check for assignment spacing
            if let Some(assign) = input
                .syntax()
                .children_with_tokens()
                .find(|c| c.kind() == SyntaxKind::Assignment)
            {
                match (
                    assign.next_sibling_or_token().unwrap().kind(),
                    assign.prev_sibling_or_token().unwrap().kind(),
                ) {
                    (SyntaxKind::Whitespace, SyntaxKind::Whitespace) => {}
                    _ => {
                        state.add(call_input_assignment(assign.text_range().to_span()));
                    }
                }
            }
        });

        // Check for one input per line
        let mut newline_seen = 0;
        call.syntax()
            .children_with_tokens()
            .for_each(|c| match c.kind() {
                SyntaxKind::Whitespace => {
                    if c.to_string().contains('\n') {
                        newline_seen += 1;
                    }
                }
                SyntaxKind::CallInputItemNode => {
                    if newline_seen == 0 && inputs > 1 {
                        state.add(call_input_missing_newline(c.text_range().to_span()));
                    }
                    newline_seen = 0;
                }
                _ => {}
            });
    }
}
