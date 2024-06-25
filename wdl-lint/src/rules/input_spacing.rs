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
const ID: &str = "InputSpacing";

/// Creates a "input spacing" diagnostic.
fn input_spacing(span: Span) -> Diagnostic {
    Diagnostic::warning("input not properly spaced")
        .with_rule(ID)
        .with_highlight(span)
        .with_fix(
            "input keyword must be separated from the opening brace (\"{\") by a single space"
                .to_string(),
        )
}

/// Creates an input keyword preceding newline diagnostic.
fn input_keyword_preceding_newline(span: Span) -> Diagnostic {
    Diagnostic::warning("input keyword may not be preceded by a newline")
        .with_rule(ID)
        .with_highlight(span)
        .with_fix(
            "input keyword must be separated from the opening brace (\"{\") by a single space \
             with no newlines",
        )
}

/// Creates an input call spacing diagnostic.
fn input_call_spacing(span: Span) -> Diagnostic {
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
pub struct InputSpacingRule;

impl Rule for InputSpacingRule {
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
        Box::new(InputSpacingVisitor)
    }
}

/// Implements the visitor for the input spacing rule.
struct InputSpacingVisitor;

impl Visitor for InputSpacingVisitor {
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
                        SyntaxKind::Whitespace => {
                            if next.to_string() != " " {
                                state.add(input_spacing(c.text_range().to_span()));
                            } else {
                                match next.next_sibling_or_token().unwrap().kind() {
                                    SyntaxKind::InputKeyword => {}
                                    _ => {
                                        state.add(input_spacing(c.text_range().to_span()));
                                    }
                                }
                            }
                        }
                        _ => {
                            state.add(input_spacing(c.text_range().to_span()));
                        }
                    }
                }
                SyntaxKind::InputKeyword => {
                    input_seen = true;
                    newline_seen = 0;
                }
                SyntaxKind::Whitespace => {
                    if c.to_string().contains("\n") {
                        if !input_seen {
                            state.add(input_keyword_preceding_newline(c.text_range().to_span()));
                        } else {
                            newline_seen += 1;
                        }
                    }
                }
                SyntaxKind::CallInputItemNode => {
                    if newline_seen > 0 {
                        // Empty lines will be detected by the Whitespace rule
                        newline_seen = 0;
                    } else if inputs > 1 {
                        state.add(input_call_spacing(c.text_range().to_span()));
                    }

                    if c.to_string().contains("=") && !c.to_string().contains(" = ") {
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
