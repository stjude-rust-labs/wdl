//! A lint rule for blank spacing between elements.

use wdl_ast::AstNode;
use wdl_ast::Diagnostic;
use wdl_ast::Diagnostics;
use wdl_ast::Document;
use wdl_ast::Span;
use wdl_ast::SyntaxKind;
use wdl_ast::SyntaxNode;
use wdl_ast::ToSpan;
use wdl_ast::VisitReason;
use wdl_ast::Visitor;

use crate::Rule;
use crate::Tag;
use crate::TagSet;

/// The identifier for the blanks between elements rule.
const ID: &str = "BlanksBetweenElements";

/// Creates an excessive blank line diagnostic.
fn excess_blank_line(span: Span) -> Diagnostic {
    Diagnostic::note("extra blank line(s) found")
        .with_rule(ID)
        .with_highlight(span)
        .with_fix("remove blank line(s)")
}

/// Creates a missing blank line diagnostic.
fn missing_blank_line(span: Span) -> Diagnostic {
    Diagnostic::note("missing blank line")
        .with_rule(ID)
        .with_highlight(span)
        .with_fix("add blank line before this element")
}

/// Detects unsorted input declarations.
#[derive(Default, Debug, Clone, Copy)]
pub struct BlanksBetweenElementsRule;

impl Rule for BlanksBetweenElementsRule {
    fn id(&self) -> &'static str {
        ID
    }

    fn description(&self) -> &'static str {
        "Ensures that WDL elements are spaced appropriately."
    }

    fn explanation(&self) -> &'static str {
        "There should be a blank line between each WDL element at the root indentation level (such \
         as the import block and any task/workflow definitions) and between sections of a WDL task \
         or workflow. Never have a blank line when indentation levels are changing (such as \
         between the opening of a workflow definition and the meta section). There should also \
         never be blanks within a meta, parameter meta, input, output, or runtime section. See \
         example for a complete WDL document with proper spacing between elements. Note the blank \
         lines between meta, parameter meta, input, the first call or first private declaration, \
         output, and runtime for the example task. The blank line between the workflow definition \
         and the task definition is also important."
    }

    fn tags(&self) -> TagSet {
        TagSet::new(&[Tag::Spacing])
    }
}

/// Check for blank lines between elements of `meta`, `parameter_meta`,
/// `input`, `output`, and `runtime` sections.
fn check_blank_lines(syntax: &SyntaxNode, state: &mut Diagnostics) {
    let mut newline_seen = 0;
    let mut blank_start = None;
    syntax
        .descendants_with_tokens()
        .for_each(|c| match c.kind() {
            SyntaxKind::Whitespace => {
                let count = c.to_string().chars().filter(|c| *c == '\n').count();
                blank_start = Some(c.text_range().to_span().start());
                newline_seen += count;
            }
            _ => {
                // Blank lines are not allowed between elements
                if newline_seen > 1 {
                    if let Some(start) = blank_start {
                        state.add(excess_blank_line(Span::new(
                            start,
                            c.text_range().to_span().start() - start,
                        )));
                    }
                }
                newline_seen = 0;
                blank_start = None;
            }
        });
}

/// Check blank lines in definitions (workflow and task).
fn check_blank_lines_definition(syntax: &SyntaxNode, state: &mut Diagnostics) {
    let mut newline_seen = 0;
    let mut blank_start = None;
    syntax.children_with_tokens().for_each(|c| match c.kind() {
        SyntaxKind::Whitespace => {
            let count = c.to_string().chars().filter(|c| *c == '\n').count();
            blank_start = Some(c.text_range().to_span().start());
            newline_seen += count;
        }
        SyntaxKind::OpenBrace => {
            if let Some(next) = c.next_sibling_or_token() {
                if next.kind() == SyntaxKind::Whitespace {
                    let count = next.to_string().chars().filter(|c| *c == '\n').count();
                    if count > 1 {
                        let end = c.text_range().to_span().end();
                        state.add(excess_blank_line(Span::new(
                            end,
                            next.text_range().to_span().end() - end,
                        )));
                    }
                }
            }
        }
        SyntaxKind::CloseBrace => {
            if newline_seen > 1 {
                if let Some(start) = blank_start {
                    state.add(excess_blank_line(Span::new(
                        start,
                        c.text_range().to_span().start() - start,
                    )));
                }
            }
            newline_seen = 0;
            blank_start = None;
        }
        SyntaxKind::BoundDeclNode => {
            if newline_seen > 2 {
                if let Some(start) = blank_start {
                    state.add(excess_blank_line(Span::new(
                        start,
                        c.text_range().to_span().start() - start,
                    )));
                }
            }
            newline_seen = 0;
            blank_start = None;
        }
        SyntaxKind::Comment => {
            newline_seen = 0;
            blank_start = None;
        }
        _ => {
                // 2 newlines == 1 blank line between elements
                // > 2 newlines / Consecutive blank lines will be handled by the Whitespace rule.
                if newline_seen == 1 {
                    let start = c.prev_sibling_or_token().unwrap().text_range().to_span().end();
                    state.add(missing_blank_line(
                        Span::new(start, c.text_range().to_span().start() - start),
                    ));
                }
                newline_seen = 0;
                blank_start = None;
            }
    });
}

impl Visitor for BlanksBetweenElementsRule {
    type State = Diagnostics;

    fn document(&mut self, state: &mut Self::State, reason: VisitReason, doc: &Document) {
        if reason == VisitReason::Exit {
            return;
        }

        if reason == VisitReason::Enter {
            // Reset the visitor upon document entry
            *self = Default::default();
        }

        let mut newline_seen = 0;
        let mut blank_start = None;

        let mut prior_comment = false;

        doc.syntax()
            .children_with_tokens()
            .for_each(|c| match c.kind() {
                // Don't require blank lines between `import` statements.
                SyntaxKind::ImportStatementNode => {
                    newline_seen = 0;
                    blank_start = None;
                    prior_comment = false;
                }
                SyntaxKind::Whitespace => {
                    let count = c.to_string().chars().filter(|c| *c == '\n').count();
                    blank_start = Some(c.text_range().to_span().start());
                    newline_seen += count;
                    // prior_comment = false;
                }
                SyntaxKind::Comment => {
                    newline_seen = 0;
                    blank_start = None;
                    prior_comment = true;
                }
                _ => {
                // 2 newlines == 1 blank line between elements
                // > 2 newlines / Consecutive blank lines will be handled by the Whitespace rule.
                if newline_seen == 1 && !prior_comment {
                    let start = c.prev_sibling_or_token().unwrap().text_range().to_span().end();
                    state.add(missing_blank_line(
                        Span::new(start, c.text_range().to_span().start() - start),
                    ));
                }
                newline_seen = 0;
                blank_start = None;
                prior_comment = false;
            }
            });
    }

    fn metadata_section(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        section: &wdl_ast::v1::MetadataSection,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        check_blank_lines(section.syntax(), state);
    }

    fn parameter_metadata_section(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        section: &wdl_ast::v1::ParameterMetadataSection,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        check_blank_lines(section.syntax(), state);
    }

    fn input_section(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        section: &wdl_ast::v1::InputSection,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        check_blank_lines(section.syntax(), state);
    }

    fn output_section(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        section: &wdl_ast::v1::OutputSection,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        check_blank_lines(section.syntax(), state);
    }

    fn runtime_section(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        section: &wdl_ast::v1::RuntimeSection,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        check_blank_lines(section.syntax(), state);
    }

    fn call_statement(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        stmt: &wdl_ast::v1::CallStatement,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        check_blank_lines(stmt.syntax(), state);
    }

    fn scatter_statement(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        stmt: &wdl_ast::v1::ScatterStatement,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        check_blank_lines(stmt.syntax(), state);
    }

    fn task_definition(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        task: &wdl_ast::v1::TaskDefinition,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        check_blank_lines_definition(task.syntax(), state);
    }

    fn workflow_definition(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        workflow: &wdl_ast::v1::WorkflowDefinition,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        check_blank_lines_definition(workflow.syntax(), state);
    }
}
