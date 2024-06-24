//! Ensures that lines do not exceed a certain width.

use wdl_ast::AstToken;
use wdl_ast::Diagnostic;
use wdl_ast::Diagnostics;
use wdl_ast::Span;
use wdl_ast::Visitor;

use crate::Rule;
use crate::Tag;
use crate::TagSet;

/// The maximum width of a line.
const MAX_WIDTH: usize = 90;

/// The identifier for the line width rule.
const ID: &str = "LineWidth";

/// Creates a diagnostic for when a line exceeds the maximum width.
fn line_too_long(span: Span) -> Diagnostic {
    Diagnostic::note(format!("line exceeds maximum width of {}", MAX_WIDTH))
        .with_rule(ID)
        .with_highlight(span)
        .with_fix("split the line into multiple lines")
}

/// Detects lines that exceed a certain width.
#[derive(Debug, Clone, Copy)]
pub struct LineWidthRule;

impl Rule for LineWidthRule {
    fn id(&self) -> &'static str {
        ID
    }

    fn description(&self) -> &'static str {
        "Ensures that lines do not exceed a certain width."
    }

    fn explanation(&self) -> &'static str {
        "Lines should not exceed a certain width to make it easier to read and understand the \
         code. This rule ensures that lines do not exceed a certain width."
    }

    fn tags(&self) -> TagSet {
        TagSet::new(&[Tag::Style, Tag::Clarity])
    }

    fn visitor(&self) -> Box<dyn Visitor<State = Diagnostics>> {
        Box::new(LineWidthVisitor::default())
    }
}

/// A visitor that detects lines that exceed a certain width.
#[derive(Debug, Clone, Copy, Default)]
struct LineWidthVisitor {
    /// The offset of the previous newline.
    prev_newline_offset: usize,
    /// Whether we are in a section that should be ignored.
    should_ignore: bool,
}

impl LineWidthVisitor {
    /// Detects lines that exceed a certain width.
    fn detect_line_too_long(&mut self, state: &mut Diagnostics, text: &str, start: usize) {
        let mut cur_newline_offset = start;
        text.char_indices().for_each(|(i, c)| {
            if c == '\n' {
                cur_newline_offset = start + i;
                if self.should_ignore {
                    self.prev_newline_offset = cur_newline_offset + 1;
                    return;
                }

                if cur_newline_offset - self.prev_newline_offset > MAX_WIDTH {
                    state.add(line_too_long(Span::new(
                        self.prev_newline_offset,
                        cur_newline_offset - self.prev_newline_offset,
                    )));
                }
                self.prev_newline_offset = cur_newline_offset + 1;
            }
        });
    }
}

impl Visitor for LineWidthVisitor {
    type State = Diagnostics;

    fn whitespace(&mut self, state: &mut Self::State, whitespace: &wdl_ast::Whitespace) {
        self.detect_line_too_long(state, whitespace.as_str(), whitespace.span().start());
    }

    fn command_text(&mut self, state: &mut Self::State, text: &wdl_ast::v1::CommandText) {
        self.detect_line_too_long(state, text.as_str(), text.span().start())
    }

    fn metadata_section(
        &mut self,
        _: &mut Self::State,
        reason: wdl_ast::VisitReason,
        _: &wdl_ast::v1::MetadataSection,
    ) {
        match reason {
            wdl_ast::VisitReason::Enter => {
                self.should_ignore = true;
            }
            wdl_ast::VisitReason::Exit => {
                self.should_ignore = false;
            }
        }
    }

    fn parameter_metadata_section(
        &mut self,
        _: &mut Self::State,
        reason: wdl_ast::VisitReason,
        _: &wdl_ast::v1::ParameterMetadataSection,
    ) {
        match reason {
            wdl_ast::VisitReason::Enter => {
                self.should_ignore = true;
            }
            wdl_ast::VisitReason::Exit => {
                self.should_ignore = false;
            }
        }
    }
}
