//! A lint rule for flagging misplaced lint directives.

use wdl_ast::AstToken;
use wdl_ast::Comment;
use wdl_ast::Diagnostic;
use wdl_ast::Diagnostics;
use wdl_ast::Document;
use wdl_ast::Span;
use wdl_ast::SupportedVersion;
use wdl_ast::SyntaxKind;
use wdl_ast::VisitReason;
use wdl_ast::Visitor;
use wdl_ast::EXCEPT_COMMENT_PREFIX;

use crate::rules;
use crate::Rule;
use crate::Tag;
use crate::TagSet;

/// The identifier for the unknown rule rule.
const ID: &str = "MisplacedLintDirective";

/// Creates an "unknown rule" diagnostic.
fn misplaced_lint_directive(id: &str, span: Span, exceptable_nodes: &[SyntaxKind]) -> Diagnostic {
    let locations = exceptable_nodes
        .iter()
        .map(|node| node.describe())
        .collect::<Vec<_>>()
        .join(", ");

    Diagnostic::note(format!("lint directive `{id}` above incorrect location"))
        .with_rule(ID)
        .with_label("cannot make an exception for this rule", span)
        .with_fix(format!(
            "move the lint directive to a valid location. Valid locations for this rule are \
             above: {locations}"
        ))
}

/// Detects unknown rules within lint directives.
#[derive(Default, Debug, Clone, Copy)]
pub struct MisplacedLintDirective;

impl Rule for MisplacedLintDirective {
    fn id(&self) -> &'static str {
        ID
    }

    fn description(&self) -> &'static str {
        "Flags misplaced lint directives which will have no effect."
    }

    fn explanation(&self) -> &'static str {
        "TODO"
    }

    fn tags(&self) -> TagSet {
        TagSet::new(&[Tag::Clarity, Tag::Correctness])
    }

    fn exceptable_nodes(&self) -> Option<Vec<wdl_ast::SyntaxKind>> {
        Some(vec![SyntaxKind::VersionStatementNode])
    }
}

impl Visitor for MisplacedLintDirective {
    type State = Diagnostics;

    fn document(&mut self, _: &mut Self::State, _: VisitReason, _: &Document, _: SupportedVersion) {
        // This is intentionally empty, as this rule has no state.
    }

    fn comment(&mut self, state: &mut Self::State, comment: &Comment) {
        if let Some(ids) = comment.as_str().strip_prefix(EXCEPT_COMMENT_PREFIX) {
            let start: usize = comment.span().start();
            let mut offset = EXCEPT_COMMENT_PREFIX.len();

            let excepted_element = comment
                .syntax()
                .siblings_with_tokens(rowan::Direction::Next)
                .find_map(|s| {
                    if s.kind() == SyntaxKind::Whitespace || s.kind() == SyntaxKind::Comment {
                        None
                    } else {
                        Some(s)
                    }
                });

            for id in ids.split(',') {
                // First trim the start so we can determine how much whitespace was removed
                let trimmed_start = id.trim_start();
                // Next trim the end
                let trimmed: &str = trimmed_start.trim_end();

                // Update the offset to account for the whitespace that was removed
                offset += id.len() - trimmed.len();

                if let Some(rule) = rules().iter().find(|r| r.id() == trimmed) {
                    if let Some(elem) = &excepted_element {
                        if let Some(exceptable_nodes) = rule.exceptable_nodes() {
                            if !exceptable_nodes.contains(&elem.kind()) {
                                state.add(misplaced_lint_directive(
                                    trimmed,
                                    Span::new(start + offset, trimmed.len()),
                                    &exceptable_nodes,
                                ));
                            }
                        }
                    }
                }

                // Update the offset to account for the rule id and comma
                offset += trimmed.len() + 1;
            }
        }
    }
}
