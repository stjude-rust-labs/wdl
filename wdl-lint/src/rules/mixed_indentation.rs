//! Rule for checking mixed indentation throughout the document.

use std::fmt;

use wdl_analysis::Diagnostics;
use wdl_analysis::VisitReason;
use wdl_analysis::Visitor;
use wdl_analysis::document::Document;
use wdl_analysis::lines_with_offset;
use wdl_ast::AstNode;
use wdl_ast::Diagnostic;
use wdl_ast::Span;
use wdl_ast::SupportedVersion;
use wdl_ast::SyntaxKind;

use crate::Rule;
use crate::Tag;
use crate::TagSet;

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

impl From<u8> for IndentationKind {
    fn from(value: u8) -> Self {
        match value {
            b' ' => Self::Spaces,
            b'\t' => Self::Tabs,
            _ => panic!("invalid indentation character"),
        }
    }
}

impl fmt::Display for IndentationKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Spaces => write!(f, "spaces"),
            Self::Tabs => write!(f, "tabs"),
        }
    }
}

/// Creates a "mixed indentation" diagnostic.
fn mixed_indentation(span: Span, kind: IndentationKind) -> Diagnostic {
    let anti_kind = match kind {
        IndentationKind::Spaces => IndentationKind::Tabs,
        IndentationKind::Tabs => IndentationKind::Spaces,
    };

    let fix_message = match kind {
        IndentationKind::Spaces => "use spaces consistently throughout the document",
        IndentationKind::Tabs => "use tabs consistently throughout the document",
    };

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
        .with_fix(fix_message)
}

/// Rule that checks for mixed indentation (tabs and spaces) throughout a
/// document, excluding command sections which are handled by wdl-analysis.
/// This rule helps ensure consistent indentation style for better readability.
#[derive(Default, Debug, Clone)]
pub struct MixedIndentationRule;

impl Rule for MixedIndentationRule {
    fn id(&self) -> &'static str {
        ID
    }

    fn description(&self) -> &'static str {
        "Ensures consistent indentation (no mixed spaces/tabs) throughout the document, excluding \
         command sections which are handled by the analysis `CommandMixedIndentation` rule."
    }

    fn explanation(&self) -> &'static str {
        "Mixing tabs and spaces in non-command sections reduces readability and can lead to \
         inconsistent rendering depending on editor settings."
    }

    fn tags(&self) -> TagSet {
        TagSet::new(&[Tag::Clarity, Tag::Correctness, Tag::Spacing])
    }

    fn exceptable_nodes(&self) -> Option<&'static [SyntaxKind]> {
        None
    }

    fn related_rules(&self) -> &[&'static str] {
        &[]
    }
}

impl Visitor for MixedIndentationRule {
    fn reset(&mut self) {
        *self = Default::default();
    }

    fn document(
        &mut self,
        diagnostics: &mut Diagnostics,
        reason: VisitReason,
        doc: &Document,
        _version: SupportedVersion,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        let mut first_style = None;
        let mut mixed_span = None;
        let text = doc.root().inner().text().to_string();

        // Collect all command section spans to avoid firing for them
        let command_spans: Vec<Span> = doc
            .root()
            .inner()
            .descendants()
            .filter(|node| node.kind() == SyntaxKind::CommandSectionNode)
            .map(|node| node.text_range().into())
            .collect();

        // First pass: detect mixed indentation
        for (line, start, _) in lines_with_offset(&text) {
            let mut line_indent_kind = None;

            for (i, b) in line.as_bytes().iter().enumerate() {
                match b {
                    b' ' => {
                        let current = IndentationKind::Spaces;
                        line_indent_kind = line_indent_kind.or(Some(current));
                    }
                    b'\t' => {
                        let current = IndentationKind::Tabs;
                        line_indent_kind = line_indent_kind.or(Some(current));
                    }
                    _ => break,
                }
            }

            if let Some(line_kind) = line_indent_kind {
                if let Some(first) = first_style {
                    if first != line_kind && mixed_span.is_none() {
                        // Found mixed indentation, remember position
                        let potential_span = Span::new(start, 1);
                        
                        // Check if this span is within any command section
                        let is_in_command = command_spans.iter().any(|cmd_span| {
                            potential_span.start() >= cmd_span.start() && 
                            potential_span.start() + potential_span.len() <= cmd_span.start() + cmd_span.len()
                        });
                        
                        // Only store the span if not in a command section
                        if !is_in_command {
                            mixed_span = Some(potential_span);
                        }
                        
                        break; // Exit once we find the first valid instance
                    }
                } else {
                    // Remember first indentation style encountered
                    first_style = Some(line_kind);
                }
            }
        }

        if let Some(span) = mixed_span {
            diagnostics.add(mixed_indentation(span, first_style.unwrap()));
        }
    }

    // Skip command sections as they are handled by wdl-analysis CommandVisitor
    fn command_section(
        &mut self,
        _diagnostics: &mut Diagnostics,
        _reason: VisitReason,
        _section: &wdl_ast::v1::CommandSection,
    ) {
        // Skip command sections as they are handled by wdl-analysis
    }
}
