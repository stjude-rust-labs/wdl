//! A lint rule for ensuring that newlines are consistent.

use wdl_ast::AstNode;
use wdl_ast::Diagnostic;
use wdl_ast::Diagnostics;
use wdl_ast::Span;
use wdl_ast::SyntaxKind;
use wdl_ast::ToSpan;
use wdl_ast::VisitReason;
use wdl_ast::Visitor;

use crate::Rule;
use crate::Tag;
use crate::TagSet;

/// The identifier for the inconsistent newlines rule.
const ID: &str = "InconsistentNewlines";

/// Creates an inconsistent newlines diagnostic.
fn inconsistent_newlines(span: Span) -> Diagnostic {
    Diagnostic::note("inconsistent newlines detected")
        .with_rule(ID)
        .with_highlight(span)
        .with_fix("use either \"\\n\" or \"\\r\\n\" consistently in the file")
}

/// Detects imports that are not sorted lexicographically.
#[derive(Debug, Clone, Copy)]
pub struct InconsistentNewlinesRule;

impl Rule for InconsistentNewlinesRule {
    fn id(&self) -> &'static str {
        ID
    }

    fn description(&self) -> &'static str {
        "Ensures that newline usage is consistent."
    }

    fn explanation(&self) -> &'static str {
        "Files should not mix \\n and \\r\\n line breaks. Pick one and use it consistently in your \
         project."
    }

    fn tags(&self) -> TagSet {
        TagSet::new(&[Tag::Style, Tag::Clarity])
    }

    fn visitor(&self) -> Box<dyn Visitor<State = Diagnostics>> {
        Box::new(InconsistentNewlinesVisitor)
    }
}

/// Implements the visitor for the import sort rule.
struct InconsistentNewlinesVisitor;

impl Visitor for InconsistentNewlinesVisitor {
    type State = Diagnostics;

    fn document(&mut self, state: &mut Self::State, reason: VisitReason, doc: &wdl_ast::Document) {
        if reason == VisitReason::Exit {
            return;
        }

        let mut newline = 0;
        let mut carriage_return = 0;

        doc.syntax()
            .children_with_tokens()
            .filter(|c| c.kind() == SyntaxKind::Whitespace)
            .for_each(|w| {
                if w.to_string().contains("\r\n") {
                    carriage_return += 1;
                } else if w.to_string().contains('\n') {
                    newline += 1;
                }
            });

        if newline > 0 && carriage_return > 0 {
            state.add(inconsistent_newlines(doc.syntax().text_range().to_span()));
        }
    }
}
