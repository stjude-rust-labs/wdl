//! A lint rule for checking mixed indentation in command text.

use wdl_ast::experimental::v1::CommandPart;
use wdl_ast::experimental::v1::CommandSection;
use wdl_ast::experimental::v1::Visitor;
use wdl_ast::experimental::AstToken;
use wdl_ast::experimental::Diagnostic;
use wdl_ast::experimental::Diagnostics;
use wdl_ast::experimental::Span;
use wdl_ast::experimental::VisitReason;

use super::Rule;
use crate::util::lines_with_offset;
use crate::Tag;
use crate::TagSet;

/// The identifier for the mixed indentation rule.
const ID: &str = "MixedIndentation";

/// Creates a "mixed indentation" diagnostic.
fn mixed_indentation(span: Span) -> Diagnostic {
    Diagnostic::warning("mixed indentation within a command")
        .with_rule(ID)
        .with_label("spaces mixed with tabs here", span)
        .with_fix("use the same whitespace character for indentation")
}

/// Detects mixed indentation in command text.
#[derive(Debug, Clone, Copy)]
pub struct MixedIndentationRule;

impl Rule for MixedIndentationRule {
    fn id(&self) -> &'static str {
        ID
    }

    fn description(&self) -> &'static str {
        "Ensures that lines within a command do not mix spaces and tabs."
    }

    fn explanation(&self) -> &'static str {
        "Mixing indentation (tab and space) characters within the command line causes leading \
         whitespace stripping to be skipped. Commands may be whitespace sensitive, and skipping \
         the whitespace stripping step may cause unexpected behavior."
    }

    fn tags(&self) -> TagSet {
        TagSet::new(&[Tag::Style, Tag::Spacing, Tag::Clarity])
    }

    fn visitor(&self) -> Box<dyn Visitor<State = Diagnostics>> {
        Box::new(MixedIndentationVisitor)
    }
}

/// Implements the visitor for the mixed indentation rule.
struct MixedIndentationVisitor;

impl Visitor for MixedIndentationVisitor {
    type State = Diagnostics;

    fn command_section(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        section: &CommandSection,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        let mut skip_first_line = false;
        for part in section.parts() {
            match part {
                CommandPart::Text(text) => {
                    for (i, (line, start, _)) in lines_with_offset(text.as_str()).enumerate() {
                        // Check to see if we should skip the first line
                        // This happens after we encounter a placeholder
                        if i == 0 && skip_first_line {
                            skip_first_line = false;
                            continue;
                        }

                        // Otherwise, count the leading whitespace on the line and whether tabs
                        // and/or spaces are used
                        let mut spaces = false;
                        let mut tabs = false;
                        let mut len = 0;
                        for c in line.chars() {
                            match c {
                                ' ' => spaces = true,
                                '\t' => tabs = true,
                                _ => break,
                            }

                            len += 1;
                        }

                        // If both spaces and tabs were present, the indentation is mixed
                        if spaces && tabs {
                            state.add(mixed_indentation(Span::new(
                                text.span().start() + start,
                                len,
                            )));
                        }
                    }
                }
                CommandPart::Placeholder(_) => {
                    // Encountered a placeholder, skip the next first line of text as it's
                    // really a part of the same line
                    skip_first_line = true;
                }
            }
        }
    }
}
