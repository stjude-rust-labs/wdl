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
use wdl_ast::SyntaxKind;
use wdl_ast::VisitReason;
use wdl_ast::Visitor;
use wdl_ast::v1::CommandPart;
use wdl_ast::v1::CommandSection;

use crate::Rule;
use crate::Tag;
use crate::TagSet;
use crate::util::lines_with_offset;

/// The identifier for the command section mixed indentation rule.
const ID: &str = "CommandSectionMixedIndentation";

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

/// Creates a "mixed indentation within command" warning diagnostic.
fn mixed_command_indentation(command: Span, span: Span, kind: IndentationKind) -> Diagnostic {
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

/// Creates a "mixed indentation in document" note diagnostic.
fn mixed_document_indentation(span: Span, kind: IndentationKind) -> Diagnostic {
    Diagnostic::note("mixed indentation throughout document")
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
        .with_fix("use either tabs or spaces exclusively for indentation throughout the document")
}

/// Detects mixed indentation in a command section and throughout the document.
#[derive(Default, Debug, Clone, Copy)]
pub struct CommandSectionMixedIndentationRule {
    /// The kind of indentation used in the document (outside command sections).
    document_indent_kind: Option<IndentationKind>,
    /// Whether mixed indentation has been found in the document.
    document_mixed_found: bool,
    /// The span of the first mixed document indentation, if found.
    document_mixed_span: Option<Span>,
    /// Whether we're currently checking inside a command section.
    in_command_section: bool,
}

impl Rule for CommandSectionMixedIndentationRule {
    fn id(&self) -> &'static str {
        ID
    }

    fn description(&self) -> &'static str {
        "Ensures that lines within the document and especially within command sections do not mix \
         spaces and tabs."
    }

    fn explanation(&self) -> &'static str {
        "Mixing indentation (tab and space) characters within the command line causes leading \
         whitespace stripping to be skipped. Commands may be whitespace sensitive, and skipping \
         the whitespace stripping step may cause unexpected behavior. Outside of command sections, \
         mixed indentation should be avoided for consistency and clarity."
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

impl Visitor for CommandSectionMixedIndentationRule {
    type State = Diagnostics;

    fn document(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        _: &Document,
        _: SupportedVersion,
    ) {
        if reason == VisitReason::Exit {
            // When exiting the document, check if we found mixed indentation
            // throughout the document (outside command sections)
            if self.document_mixed_found && self.document_mixed_span.is_some() {
                // Always emit the mixed indentation note for document-level issues
                state.add(mixed_document_indentation(
                    self.document_mixed_span.unwrap(),
                    self.document_indent_kind.unwrap(),
                ));
            }
            return;
        }

        // Reset the visitor upon document entry
        *self = Default::default();
    }

    fn whitespace(&mut self, _state: &mut Self::State, whitespace: &wdl_ast::Whitespace) {
        // Skip if we're inside a command section (handled separately)
        if self.in_command_section {
            return;
        }

        // Check whitespace nodes for mixed indentation when not in a command section
        // even if we already found mixed indentation elsewhere
        let text = whitespace.text();
        self.check_document_text(text, whitespace.span());
    }

    fn command_section(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        section: &CommandSection,
    ) {
        match reason {
            VisitReason::Enter => {
                self.in_command_section = true;

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
                                                // Mixed indentation, store the span of the first
                                                // mixed
                                                // character
                                                mixed_span = Some(Span::new(
                                                    text.span().start() + start + i,
                                                    1,
                                                ));
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
                    let command_keyword =
                        support::token(section.inner(), SyntaxKind::CommandKeyword)
                            .expect("should have a command keyword token");

                    state.add(mixed_command_indentation(
                        command_keyword.text_range().into(),
                        span,
                        kind.expect("an indentation kind should be present"),
                    ));
                }
            }
            VisitReason::Exit => {
                self.in_command_section = false;
            }
        }
    }
}

impl CommandSectionMixedIndentationRule {
    /// Check document text for mixed indentation
    fn check_document_text(&mut self, text: &str, span: Span) {
        for (line, start, _) in lines_with_offset(text) {
            let mut line_indent_kind = None;

            for (i, b) in line.as_bytes().iter().enumerate() {
                match b {
                    b' ' | b'\t' => {
                        let current = IndentationKind::from(*b);

                        // Set document indentation kind if not yet set
                        if self.document_indent_kind.is_none() {
                            self.document_indent_kind = Some(current);
                        }

                        // Set line indentation kind if not yet set
                        let line_kind = line_indent_kind.get_or_insert(current);

                        // Check if this line's indentation matches the document's
                        if let Some(doc_kind) = self.document_indent_kind {
                            if current != doc_kind {
                                self.document_mixed_found = true;
                                // Only store the first mixed indentation span if not already found
                                if self.document_mixed_span.is_none() {
                                    self.document_mixed_span =
                                        Some(Span::new(span.start() + start + i, 1));
                                }
                                // Continue checking other lines instead of
                                // returning early
                            }
                        }

                        // Check if this line's indentation is consistent
                        if current != *line_kind {
                            self.document_mixed_found = true;
                            // Only store the first mixed indentation span if not already found
                            if self.document_mixed_span.is_none() {
                                self.document_mixed_span =
                                    Some(Span::new(span.start() + start + i, 1));
                            }
                            // Continue checking other lines instead of
                            // returning early
                        }
                    }
                    _ => break,
                }
            }
        }
    }
}
