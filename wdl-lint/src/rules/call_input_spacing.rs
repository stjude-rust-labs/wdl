//! A lint rule for spacing of call inputs.

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

/// Creates a "input spacing" diagnostic.
fn call_input_spacing(span: Span) -> Diagnostic {
    Diagnostic::warning("input not properly spaced")
        .with_rule(ID)
        .with_highlight(span)
        .with_fix(
            "input keyword must be separated from the opening brace (\"{\") by a single space"
                .to_string(),
        )
}

/// Creates an input keyword preceding newline diagnostic.
fn call_input_keyword_preceding_newline(span: Span) -> Diagnostic {
    Diagnostic::warning("input keyword may not be preceded by a newline")
        .with_rule(ID)
        .with_highlight(span)
        .with_fix(
            "input keyword must be separated from the opening brace (\"{\") by a single space \
             with no newlines",
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
         inputs are each on their own line. This does inflate the line count of a WDL document, \
         but it is worth it for the consistent readability. An exception can be made (but does not \
         have to be made), for calls with only a single parameter. In those cases, it is \
         permissable to keep the input on the same line as the call."
    }

    fn tags(&self) -> TagSet {
        TagSet::new(&[Tag::Style, Tag::Clarity, Tag::Spacing])
    }

    fn visitor(&self) -> Box<dyn Visitor<State = Diagnostics>> {
        Box::new(CallInputSpacingVisitor)
    }
}

/// Implements the visitor for the call input spacing rule.
struct CallInputSpacingVisitor;

impl Visitor for CallInputSpacingVisitor {
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

        let mut input_seen = false;
        let mut newline_seen = 0;
        call.syntax().children_with_tokens().for_each(|c| {
            match c.kind() {
                SyntaxKind::OpenBrace => {
                    let next = c.next_sibling_or_token().unwrap();
                    match next.kind() {
                        // The opening brace of a call block must be followed by exactly one space,
                        // the input keyword, a colon and a newline
                        SyntaxKind::Whitespace => {
                            if next.to_string() != " " {
                                // Check for a single space
                                state.add(call_input_spacing(c.text_range().to_span()));
                            } else {
                                // Check for the input keyword
                                match next.next_sibling_or_token().unwrap().kind() {
                                    SyntaxKind::InputKeyword => {}
                                    _ => {
                                        state.add(call_input_spacing(c.text_range().to_span()));
                                    }
                                }
                            }
                        }
                        _ => { // Opening brace is followed by something other than whitespace
                            state.add(call_input_spacing(c.text_range().to_span()));
                        }
                    }
                }
                SyntaxKind::InputKeyword => {
                    input_seen = true;
                    newline_seen = 0;
                }
                SyntaxKind::Whitespace => {
                    if c.to_string().contains('\n') {
                        if !input_seen {
                            // Newlines are not allowed before the input keyword
                            state.add(call_input_keyword_preceding_newline(
                                c.text_range().to_span(),
                            ));
                        } else {
                            newline_seen += 1;
                        }
                    }
                }
                SyntaxKind::CallInputItemNode => {
                    if newline_seen > 0 {
                        // Reset newlines seen, since this is an input
                        // Empty lines will be detected by the Whitespace rule
                        newline_seen = 0;
                    } else if inputs > 1 {
                        // Only check for newlines if there are multiple inputs
                        state.add(call_input_missing_newline(c.text_range().to_span()));
                    }

                    // Check for assignment spacing
                    if c.to_string().contains('=') && !c.to_string().contains(" = ") {
                        let i = c.to_string().find('=').unwrap();
                        state.add(call_input_assignment(Span::new(
                            c.text_range().to_span().start() + i,
                            1,
                        )));
                    }
                }
                _ => {}
            }
        });
    }
}