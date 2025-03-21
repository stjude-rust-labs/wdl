use std::fmt;

use rowan::ast::support;
use wdl_ast::{
    AstNode, AstToken, Diagnostic, Diagnostics, Document, Span, SupportedVersion, SyntaxElement,
    SyntaxKind, VisitReason, Visitor, v1::CommandPart, v1::CommandSection,
};

use crate::{Rule, Tag, TagSet, util::lines_with_offset};

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
            _ => panic!("not indentation"),
        }
    }
}

/// Creates a "mixed indentation" diagnostic with different severity levels.
fn mixed_indentation(command: Span, span: Span, kind: IndentationKind, severity: &str) -> Diagnostic {
    let message = match severity {
        "warning" => "mixed indentation (warning)",
        "note" => "mixed indentation (note)",
        _ => "mixed indentation",
    };

    Diagnostic::new(severity, message)
        .with_rule(ID)
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
            "this section uses both tabs and spaces in leading whitespace",
            command,
        )
        .with_fix("use either tabs or spaces exclusively for indentation")
}

/// Detects mixed indentation in the entire WDL document and command sections.
#[derive(Default, Debug, Clone, Copy)]
pub struct CommandSectionMixedIndentationRule;

impl Rule for CommandSectionMixedIndentationRule {
    fn id(&self) -> &'static str {
        ID
    }

    fn description(&self) -> &'static str {
        "Ensures that lines within a command and document do not mix spaces and tabs."
    }

    fn explanation(&self) -> &'static str {
        "Mixing indentation (tab and space) characters within the document or command section \
         causes leading whitespace stripping to be skipped. Commands may be whitespace sensitive, \
         and skipping the whitespace stripping step may cause unexpected behavior."
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

    /// Check entire document for mixed indentation.
    fn document(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        document: &Document,
        version: SupportedVersion,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        let severity = match version {
            SupportedVersion::V1_0 | SupportedVersion::V1_1 => "warning",
            SupportedVersion::V1_2 => "note",
        };

        for line in document.text().lines() {
            if line.contains('\t') && line.contains(' ') {
                let start = document.text().find(line).unwrap_or(0);
                let span = Span::new(start, line.len());

                let kind = if line.find('\t').unwrap_or(0) < line.find(' ').unwrap_or(0) {
                    IndentationKind::Tabs
                } else {
                    IndentationKind::Spaces
                };

                state.add(
                    mixed_indentation(span, span, kind, severity),
                    SyntaxElement::from(document.syntax().clone()),
                );
            }
        }
    }

    /// Check mixed indentation in the command section.
    fn command_section(
        &mut self,
        state: &mut Self::State,
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
                .expect("should have a command keyword token");

            state.exceptable_add(
                mixed_indentation(
                    command_keyword.text_range().into(),
                    span,
                    kind.expect("an indentation kind should be present"),
                    "warning",
                ),
                SyntaxElement::from(section.inner().clone()),
                &self.exceptable_nodes(),
            );
        }
    }
}
