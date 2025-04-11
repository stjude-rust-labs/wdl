//! A lint rule for checking mixed indentation in command text and throughout
//! the document.

use std::fmt;

use rowan::ast::support;
use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::Diagnostic;
use wdl_ast::Diagnostics;
use wdl_ast::Document;
use wdl_ast::Span;
use wdl_ast::SupportedVersion;
use wdl_ast::SyntaxElement;
use wdl_ast::SyntaxKind;
use wdl_ast::VisitReason;
use wdl_ast::Visitor;
use wdl_ast::v1::CommandPart;
use wdl_ast::v1::CommandSection;

use crate::Rule;
use crate::Tag;
use crate::TagSet;
use crate::util::lines_with_offset;

/// The identifier for the mixed indentation rule.
const ID: &str = "MixedIndentation";

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

/// Creates a "mixed indentation" warning diagnostic for command sections.
fn mixed_indentation_warning(command: Span, span: Span, kind: IndentationKind) -> Diagnostic {
    Diagnostic::warning("mixed indentation within a command")
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
            "this command section uses both tabs and spaces in leading whitespace",
            command,
        )
        .with_fix("use either tabs or spaces exclusively for indentation")
}

/// Creates a "mixed indentation" note diagnostic for document text.
fn mixed_indentation_note(span: Span, kind: IndentationKind) -> Diagnostic {
    Diagnostic::note("mixed indentation in document")
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
        .with_fix("use either tabs or spaces exclusively for indentation")
}

/// Detects mixed indentation in a command section and throughout the document.
#[derive(Default, Debug, Clone)]
pub struct MixedIndentationRule {
    /// The text of the current document being processed
    document_text: Option<String>,

    /// Tracks command sections that have been checked to avoid duplicate
    /// diagnostics
    command_section_spans: Vec<Span>,
}

impl Rule for MixedIndentationRule {
    fn id(&self) -> &'static str {
        ID
    }

    fn description(&self) -> &'static str {
        "Ensures that lines within a document do not mix spaces and tabs."
    }

    fn explanation(&self) -> &'static str {
        "Mixing indentation (tab and space) characters within command sections causes leading \
         whitespace stripping to be skipped, which may cause unexpected behavior. In general, \
         mixing tabs and spaces throughout a document reduces readability and can lead to \
         inconsistent rendering depending on editor settings."
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

    fn related_rules(&self) -> &[&'static str] {
        &[]
    }
}

impl Visitor for MixedIndentationRule {
    type State = Diagnostics;

    fn document(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        doc: &Document,
        _: SupportedVersion,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        // Reset the visitor upon document entry
        *self = Default::default();

        // Store the document text for later use
        self.document_text = Some(doc.text().to_string());

        // Check the entire document for mixed indentation
        if let Some(ref text) = self.document_text {
            let mut kind = None;
            let mut mixed_span = None;

            'outer: for (line, start, _) in lines_with_offset(text) {
                // Check each line's leading whitespace
                for (i, b) in line.as_bytes().iter().enumerate() {
                    match b {
                        b' ' | b'\t' => {
                            let current = IndentationKind::from(*b);
                            let kind = kind.get_or_insert(current);
                            if current != *kind {
                                // Mixed indentation, store the span of the first mixed character
                                mixed_span = Some(Span::new(start + i, 1));
                                break 'outer;
                            }
                        }
                        _ => break,
                    }
                }
            }

            // If mixed indentation was found, add a note diagnostic
            // Command sections will be handled separately with warning diagnostics
            if let Some(span) = mixed_span {
                // Check if this span is within a command section we've already handled
                let in_command_section = self.command_section_spans.iter().any(|cmd_span| {
                    span.start() >= cmd_span.start()
                        && span.start() < cmd_span.start() + cmd_span.len()
                });

                // Only add a note diagnostic if not in a command section
                if !in_command_section {
                    state.add(mixed_indentation_note(
                        span,
                        kind.expect("an indentation kind should be present"),
                    ));
                }
            }
        }
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

        let command_keyword = support::token(section.inner(), SyntaxKind::CommandKeyword)
            .expect("should have a command keyword token");

        // Store the command section span to avoid duplicate diagnostics
        let command_span = command_keyword.text_range().into();
        self.command_section_spans.push(command_span);

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
            state.exceptable_add(
                mixed_indentation_warning(
                    command_span,
                    span,
                    kind.expect("an indentation kind should be present"),
                ),
                SyntaxElement::from(section.inner().clone()),
                &self.exceptable_nodes(),
            );
        }
    }
}
