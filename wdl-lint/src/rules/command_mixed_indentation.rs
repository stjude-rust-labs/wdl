//! A lint rule for checking mixed indentation in command text.

use std::fmt;
use wdl_ast::version::V1;

use rowan::ast::support;
use wdl_ast::{AstNode, AstToken, Diagnostic, Diagnostics, Document, Span, SupportedVersion, SyntaxElement, SyntaxKind, VisitReason, Visitor};
use wdl_ast::v1::{CommandPart, CommandSection};
use crate::{Rule, Tag, TagSet};
use crate::util::lines_with_offset;

/// The identifier for the command section mixed indentation rule.
const ID: &str = "CommandSectionMixedIndentation";

/// Represents the indentation kind.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
enum IndentationKind {
    Spaces,
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
            _ => panic!("Unexpected indentation character"),
        }
    }
}

/// Creates a "mixed indentation" diagnostic with WDL version handling.
fn mixed_indentation(
    command: Span,
    span: Span,
    kind: IndentationKind,
    version: SupportedVersion,
) -> Diagnostic {
    let version_msg = match version {
        SupportedVersion::V1(V1::One) => "WDL v1.0",
        SupportedVersion::V1(V1::Two) => "WDL v1.1",
        _ => "Unknown WDL version",  // Handle non-exhaustive cases
    };
    

    Diagnostic::warning(format!("Mixed indentation detected ({version_msg})"))
        .with_rule(ID)
        .with_label(
            format!(
                "Indented with {kind} until this {anti}",
                anti = match kind {
                    IndentationKind::Spaces => "tab",
                    IndentationKind::Tabs => "space",
                }
            ),
            span,
        )
        .with_label(
            "Command section uses both tabs and spaces in leading whitespace",
            command,
        )
        .with_fix("Use either tabs or spaces exclusively for indentation")
}

/// Detects mixed indentation in a command section.
#[derive(Default, Debug, Clone, Copy)]
pub struct CommandSectionMixedIndentationRule;

impl Rule for CommandSectionMixedIndentationRule {
    fn id(&self) -> &'static str {
        ID
    }

    fn description(&self) -> &'static str {
        "Ensures that lines within a command do not mix spaces and tabs, with WDL version handling."
    }

    fn explanation(&self) -> &'static str {
        "Mixing indentation (tabs and spaces) within the command section can cause unexpected \
         behavior, especially in WDL commands that are whitespace-sensitive."
    }

    fn tags(&self) -> TagSet {
        TagSet::new(&[Tag::Correctness, Tag::Spacing, Tag::Clarity])
    }

    fn exceptable_nodes(&self) -> Option<&'static [SyntaxKind]> {
        Some(&[
            SyntaxKind::VersionStatementNode,
            SyntaxKind::TaskDefinitionNode,
            SyntaxKind::CommandSectionNode,
        ])
    }
}

impl Visitor for CommandSectionMixedIndentationRule {
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

        *self = Default::default();
    }

    fn command_section(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        section: &CommandSection,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        let version = SupportedVersion::V1(V1::One);  



        let mut kind = None;
        let mut mixed_span = None;
        let mut skip_next_line = false;

        'outer: for part in section.parts() {
            match part {
                CommandPart::Text(text) => {
                    for (line, start, _) in lines_with_offset(text.text()) {
                        if skip_next_line {
                            skip_next_line = false;
                            continue;
                        }

                        for (i, b) in line.as_bytes().iter().enumerate() {
                            match b {
                                b' ' | b'\t' => {
                                    let current = IndentationKind::from(*b);
                                    let kind = kind.get_or_insert(current);
                                    if current != *kind {
                                        mixed_span = Some(Span::new(text.span().start() + start + i, 1));
                                        break 'outer;
                                    }
                                }
                                _ => break,
                            }
                        }
                    }
                }
                CommandPart::Placeholder(_) => {
                    skip_next_line = true;
                }
            }
        }

        if let Some(span) = mixed_span {
            let command_keyword = support::token(section.inner(), SyntaxKind::CommandKeyword)
                .expect("command keyword token expected");

            state.exceptable_add(
                mixed_indentation(
                    command_keyword.text_range().into(),
                    span,
                    kind.expect("Indentation kind expected"),
                    version,
                ),
                SyntaxElement::from(section.inner().clone()),
                &self.exceptable_nodes(),
            );
        }
    }
}
