//! A lint rule for flagging malformed lint directives.
use wdl_ast::AstToken;
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
use crate::util::is_inline_comment;

/// The identifier for the Malformed Lint Directive rule.
const ID: &str = "MalformedLintDirective";
/// The accepted lint directives.
const ACCEPTED_LINT_DIRECTIVES: [&str; 1] = ["except"];

/// Creates an "Inline Lint Directive" diagnostic.
fn inline_lint_directive(
    span: Span,
) -> Diagnostic {
    Diagnostic::warning(
        "lint directive must be on its own line")
        .with_rule(ID)
        .with_label("malformed lint directive", span,
        )
        .with_fix("place lint directive on new line")
}

/// Creates an "Invalid Lint Directive" diagnostic.
fn invalid_lint_directive(
    span: Span,
) -> Diagnostic {
    let accepted_directives = ACCEPTED_LINT_DIRECTIVES.join(", ");
    Diagnostic::warning(
        "lint directive not recognized")
        .with_rule(ID)
        .with_label("lint directive not recognized", span)
        .with_fix(format!("remove unrecognized lint directives, use any of the directives included: [{:#?}]", accepted_directives))
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
        "Comments which begin with `#@` must only contain valid lint directives. Lint directives \
        must be on their own line, only preceded by whitespace."
    }


    fn tags(&self) -> TagSet {
        TagSet::new(&[Tag::Clarity, Tag::Correctness])
    }

    fn exceptable_nodes(&self) -> Option<&'static [wdl_ast::SyntaxKind]> {
        // Some(&[SyntaxKind::VersionStatementNode])
        None
    }
}

impl Visitor for MalformedLintDirectiveRule {
    type State = Diagnostics;

    fn document(&mut self, _: &mut Self::State, _: VisitReason, _: &Document, _: SupportedVersion) {
        // This is intentionally empty, as this rule has no state.
    }

    fn comment(&mut self, state: &mut Self::State, comment: &Comment) {
        if comment.as_str().starts_with("#@") {
            if is_inline_comment(comment) {
                state.add(inline_lint_directive(
                    comment.span(),
                ));
            } else if let Some(directive) = comment.as_str().strip_prefix("#@") {
                if !ACCEPTED_LINT_DIRECTIVES.contains(&directive) {
                    state.add(invalid_lint_directive(
                        comment.span(),
                    ));
                }
            } else {
                return;
            }
        } else {
            return;
        }
    }
}
