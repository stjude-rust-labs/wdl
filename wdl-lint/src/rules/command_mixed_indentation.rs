// A lint rule for checking mixed indentation in command text and throughout
// the document.

use std::fmt;

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
use wdl_ast::v1::CommandSection;

use crate::Rule;
use crate::tags::Tag;
use crate::tags::TagSet;
use crate::util::lines_with_offset;

/// The identifier for the command section mixed indentation rule.
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
#[derive(Default, Debug, Clone)]
pub struct MixedIndentationRule {
    /// The visitor that does the actual work.
    visitor: MixedIndentationVisitor,
}

impl Visitor for MixedIndentationRule {
    type State = Diagnostics;

    fn document(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        document: &Document,
        version: SupportedVersion,
    ) {
        self.visitor.document(state, reason, document, version);
    }

    fn whitespace(&mut self, state: &mut Self::State, whitespace: &wdl_ast::Whitespace) {
        self.visitor.whitespace(state, whitespace);
    }

    fn command_section(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        section: &CommandSection,
    ) {
        self.visitor.command_section(state, reason, section);
    }
}

impl Rule for MixedIndentationRule {
    fn id(&self) -> &'static str {
        ID
    }

    fn description(&self) -> &'static str {
        "Ensures consistent indentation throughout the document and command sections"
    }

    fn explanation(&self) -> &'static str {
        "Whitespace in indentation should be consistent throughout a document. Do not mix tabs and \
         spaces for indentation as this can lead to inconsistent rendering across platforms and \
         editors. Command sections should especially use consistent indentation to ensure proper \
         script execution."
    }

    fn tags(&self) -> TagSet {
        TagSet::new(&[Tag::Clarity, Tag::Correctness, Tag::Spacing])
    }

    fn exceptable_nodes(&self) -> Option<&'static [SyntaxKind]> {
        None
    }

    fn related_rules(&self) -> &[&'static str] {
        &["Whitespace"]
    }
}

/// A visitor that checks for mixed indentation in a document.
#[derive(Default, Debug, Clone)]
struct MixedIndentationVisitor {
    /// Whether or not we're currently in a command section.
    in_command_section: bool,
    /// The indentation kind found for the document, if mixed indentation was
    /// detected.
    document_indentation_kind: Option<IndentationKind>,
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
            // Only emit a diagnostic if mixed indentation was actually found
            if let Some(kind) = self.document_indentation_kind {
                // Check if we've found mixed indentation (kind is set when we detect a
                // mismatch)
                state.add(mixed_document_indentation(Span::new(0, 0), kind));
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

        if let Some(text) = section.text() {
            for line in text.text().lines() {
                if line.is_empty() {
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
                    let diagnostic = Diagnostic::warning("mixed indentation in command section")
                        .with_rule(ID)
                        .with_highlight(section.span())
                        .with_fix(
                            "use either tabs or spaces exclusively for indentation in command \
                             sections",
                        );
                    state.add(diagnostic);
                    break;
                }
            }
        }
    }
}

impl MixedIndentationVisitor {
    /// Check document text for mixed indentation
    fn check_document_text(&mut self, text: &str, _span: Span) {
        let mut doc_indent_kind = self.document_indentation_kind;
        for (line, _start, _) in lines_with_offset(text) {
            let mut line_indent_kind = None;

            for b in line.as_bytes().iter() {
                match b {
                    b' ' | b'\t' => {
                        let current = IndentationKind::from(*b);

                        // Set line indentation kind if not yet set
                        let line_kind = line_indent_kind.get_or_insert(current);

                        // Check if this line's indentation is consistent within itself
                        if current != *line_kind {
                            // Found mixed indentation within a line
                            self.document_indentation_kind = Some(current);
                            return;
                        }

                        // If document indentation kind is not set, set it to the current line's
                        // kind
                        if doc_indent_kind.is_none() {
                            doc_indent_kind = Some(current);
                        }
                        // Check if this line's indentation differs from the document's
                        else if let Some(doc_kind) = doc_indent_kind {
                            if current != doc_kind {
                                // Found mixed indentation between lines
                                self.document_indentation_kind = Some(current);
                                return;
                            }
                        }
                    }
                    _ => break,
                }
            }
        }
        // Only update document_indentation_kind if we found mixed indentation
        // If we reach this point, no mixed indentation was found
        self.document_indentation_kind = None;
    }
}
