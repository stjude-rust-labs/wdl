//! Validation for command sections.

use std::fmt;

use rowan::ast::support;
use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::Diagnostic;
use wdl_ast::Span;
use wdl_ast::SyntaxKind;
use wdl_ast::v1::CommandPart;
use wdl_ast::v1::CommandSection;

use crate::Diagnostics;
use crate::COMMAND_MIXED_INDENTATION_RULE_ID;
use crate::VisitReason;
use crate::Visitor;
use crate::lines_with_offset;

/// Represents the indentation kind.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum IndentationKind {
    /// Spaces are used for the indentation.
    Spaces,
    /// Tabs are used for the indentation.
    Tabs,
}

impl fmt::Display for IndentationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spaces => write!(f, "spaces"),
            Self::Tabs => write!(f, "tabs"),
        }
    }
}

impl From<u8> for IndentationKind {
    fn from(b: u8) -> Self {
        match b {
            b' ' => Self::Spaces,
            b'\t' => Self::Tabs,
            _ => panic!("not indentation"),
        }
    }
}

/// Creates a "mixed indentation" diagnostic.
fn mixed_indentation(command: Span, span: Span, kind: IndentationKind) -> Diagnostic {
    Diagnostic::warning("mixed indentation within a command")
        .with_rule(COMMAND_MIXED_INDENTATION_RULE_ID)
        .with_label(
            format!(
                "indented with {kind} until this {anti}",
                anti = match kind {
                    IndentationKind::Spaces => "tab",
                    IndentationKind::Tabs => "space",
                }
            ),
            span,
        )
        .with_label(
            "this command section uses both tabs and spaces in leading whitespace",
            command,
        )
        .with_fix("use either tabs or spaces exclusively for indentation")
}

/// Detects mixed indentation in a command section.
#[derive(Default, Debug, Clone, Copy)]
pub struct CommandVisitor;

impl Visitor for CommandVisitor {
    fn reset(&mut self) {
        *self = Self;
    }

    fn command_section(
        &mut self,
        diagnostics: &mut Diagnostics,
        reason: VisitReason,
        section: &CommandSection,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        let mut kind = None;
        let mut mixed_span = None;
        let mut skip_next_line = false;
        'outer: for part in section.parts() {
            match part {
                CommandPart::Text(text) => {
                    for (line, start, _) in lines_with_offset(text.text()) {
                        // Check to see if we should skip the next line
                        // This happens after we encounter a placeholder
                        if skip_next_line {
                            skip_next_line = false;
                            continue;
                        }

                        // Otherwise, check the leading whitespace
                        for (i, b) in line.as_bytes().iter().enumerate() {
                            match b {
                                b' ' | b'\t' => {
                                    let current = IndentationKind::from(*b);
                                    let kind = kind.get_or_insert(current);
                                    if current != *kind {
                                        // Mixed indentation, store the span of the first mixed
                                        // character
                                        mixed_span =
                                            Some(Span::new(text.span().start() + start + i, 1));
                                        break 'outer;
                                    }
                                }
                                _ => break,
                            }
                        }
                    }
                }
                CommandPart::Placeholder(_) => {
                    // Encountered a placeholder, skip the next line of text as it's
                    // really a part of the same line
                    skip_next_line = true;
                }
            }
        }

        if let Some(span) = mixed_span {
            let command_keyword = support::token(section.inner(), SyntaxKind::CommandKeyword)
                .expect("should have a command keyword token");

            diagnostics.add(mixed_indentation(
                command_keyword.text_range().into(),
                span,
                kind.expect("an indentation kind should be present"),
            ));
        }
    }
}
