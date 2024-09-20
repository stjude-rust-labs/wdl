//! A lint rule for flagging malformed lint directives.

use wdl_ast::Comment;
use wdl_ast::Diagnostic;
use wdl_ast::Diagnostics;
use wdl_ast::Document;
use wdl_ast::Span;
use wdl_ast::SupportedVersion;
use wdl_ast::SyntaxElement;
use wdl_ast::SyntaxKind;
use wdl_ast::ToSpan;
use wdl_ast::VisitReason;
use wdl_ast::Visitor;
use crate::Rule;
use crate::Tag;
use crate::TagSet;

/// The identifier for the Malformed Lint Directive rule.
const ID: &str = "MalformedLintDirective";

/// Creates a "Malformed Lint Directive" diagnostic.
fn malformed_lint_directive(
    id: &str,
    span: Span,
    wrong_element: &SyntaxElement,
    exceptable_nodes: &[SyntaxKind],
) -> Diagnostic {
    let locations = exceptable_nodes
        .iter()
        .map(|node| node.describe())
        .collect::<Vec<_>>()
        .join(", ");

    Diagnostic::error(format!(
        "lint directive `{id}` is malformed"
    ))
        .with_label("cannot make an exception for this rule", span)
        .with_label(
            "invalid element for this lint directive",
            wrong_element.text_range().to_span(),
        )
        .with_fix(format!(
            "valid locations for this directive are above: {locations}"
        ))
}

/// Detects a malformed lint directive.
#[derive(Default, Debug, Clone, Copy)]
pub struct MalformedLintDirectiveRule;

impl Rule for MalformedLintDirectiveRule {
    fn id(&self) -> &'static str {
        ID
    }

    fn description(&self) -> &'static str {
        "Flags malformed lint directives"
    }

    fn explanation(&self) -> &'static str {
        todo!()
    }


    fn tags(&self) -> TagSet {
        TagSet::new(&[Tag::Clarity, Tag::Correctness])
    }

    fn exceptable_nodes(&self) -> Option<&'static [wdl_ast::SyntaxKind]> {
        Some(&[SyntaxKind::VersionStatementNode])
    }
}

impl Visitor for MalformedLintDirectiveRule {
    type State = Diagnostics;

    fn document(&mut self, _: &mut Self::State, _: VisitReason, _: &Document, _: SupportedVersion) {
        // This is intentionally empty, as this rule has no state.
    }

    fn comment(&mut self, _state: &mut Self::State, comment: &Comment) {
        println!("Comment: {:?}", comment);
        // if let Some(ids) = comment.as_str().strip_prefix(EXCEPT_COMMENT_PREFIX) {
        //     let start: usize = comment.span().start();
        //     let mut offset = EXCEPT_COMMENT_PREFIX.len();
        //
        //     let excepted_element = comment
        //         .syntax()
        //         .siblings_with_tokens(rowan::Direction::Next)
        //         .find_map(|s| {
        //             if s.kind() == SyntaxKind::Whitespace || s.kind() == SyntaxKind::Comment {
        //                 None
        //             } else {
        //                 Some(s)
        //             }
        //         });
        //
        //     for id in ids.split(',') {
        //         // First trim the start so we can determine how much whitespace was removed
        //         let trimmed_start = id.trim_start();
        //         // Next trim the end
        //         let trimmed: &str = trimmed_start.trim_end();
        //
        //         // Update the offset to account for the whitespace that was removed
        //         offset += id.len() - trimmed.len();
        //
        //         if let Some(elem) = &excepted_element {
        //             if let Some(Some(exceptable_nodes)) = RULE_MAP.get(trimmed) {
        //                 if !exceptable_nodes.contains(&elem.kind()) {
        //                     state.add(crate::rules::misplaced_lint_directive::misplaced_lint_directive(
        //                         trimmed,
        //                         Span::new(start + offset, trimmed.len()),
        //                         elem,
        //                         exceptable_nodes,
        //                     ));
        //                 }
        //             }
        //         }
        //
        //         // Update the offset to account for the rule id and comma
        //         offset += trimmed.len() + 1;
        //     }
        // }
    }


}