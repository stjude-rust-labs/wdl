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
pub struct MixedIndentationRule;

impl Rule for MixedIndentationRule {
    fn name(&self) -> &'static str {
        "MixedIndentation"
    }

    fn description(&self) -> &'static str {
        "Ensures consistent indentation throughout the document and command sections"
    }

    fn tags(&self) -> &[&'static str] {
        &["clarity", "correctness", "spacing"]
    }

    fn visitor(&self) -> Box<dyn Visitor> {
        Box::new(MixedIndentationVisitor::default())
    }
}

#[derive(Default)]
struct MixedIndentationVisitor {
    in_command_section: bool,
    document_indentation_kind: Option<IndentationKind>,
    command_indentation_kind: Option<IndentationKind>,
}

impl Visitor for MixedIndentationVisitor {
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
            if self.document_indentation_kind.is_some() {
                // Always emit the mixed indentation note for document-level issues
                state.add(mixed_document_indentation(
                    Span::new(0, 0),
                    self.document_indentation_kind.unwrap(),
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
            }
            VisitReason::Exit => {
                self.in_command_section = false;
                return;
            }
        }

        // Check for mixed indentation in command section
        let mut has_spaces = false;
        let mut has_tabs = false;
        let mut line_number = section.span().start().line_number();

        for line in section.text().lines() {
            if line.is_empty() {
                line_number += 1;
                continue;
            }

            let leading_whitespace = line
                .chars()
                .take_while(|c| c.is_whitespace())
                .collect::<String>();

            if leading_whitespace.contains(' ') {
                has_spaces = true;
            }
            if leading_whitespace.contains('\t') {
                has_tabs = true;
            }

            if has_spaces && has_tabs {
                state.add(
                    section.span(),
                    "mixed indentation in command section",
                    "use either tabs or spaces exclusively for indentation in command sections",
                    Severity::Warning,
                );
                break;
            }

            line_number += 1;
        }
    }
}

impl MixedIndentationVisitor {
    /// Check document text for mixed indentation
    fn check_document_text(&mut self, text: &str, span: Span) {
        for (line, start, _) in lines_with_offset(text) {
            let mut line_indent_kind = None;

            for (i, b) in line.as_bytes().iter().enumerate() {
                match b {
                    b' ' | b'\t' => {
                        let current = IndentationKind::from(*b);

                        // Set document indentation kind if not yet set
                        if self.document_indentation_kind.is_none() {
                            self.document_indentation_kind = Some(current);
                        }

                        // Set line indentation kind if not yet set
                        let line_kind = line_indent_kind.get_or_insert(current);

                        // Check if this line's indentation matches the document's
                        if let Some(doc_kind) = self.document_indentation_kind {
                            if current != doc_kind {
                                self.document_indentation_kind = Some(current);
                            }
                        }

                        // Check if this line's indentation is consistent
                        if current != *line_kind {
                            self.document_indentation_kind = Some(current);
                        }
                    }
                    _ => break,
                }
            }
        }
    }
}
