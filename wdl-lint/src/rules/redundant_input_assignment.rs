//! A lint rule for flagging redundant input assignments
use wdl_ast::AstToken;
use wdl_ast::Comment;
use wdl_ast::Diagnostic;
use wdl_ast::Diagnostics;
use wdl_ast::Document;
use wdl_ast::Span;
use wdl_ast::SupportedVersion;
use wdl_ast::VisitReason;
use wdl_ast::Visitor;

use crate::Rule;
use crate::rules::MalformedLintDirectiveRule;
use crate::Tag;
use crate::TagSet;
use crate::util::is_inline_comment;

/// The identifier for the Redundant Input Assignment rule.
const ID: &str = "RedundantInputAssignment";

/// Create a "Redundant Input Assignment" diagnostic.
fn redundant_input_assignment(span: Span) -> Diagnostic {
    Diagnostic::note("redundant input assignments can be shortened")
        .with_rule(ID)
        .with_label("redundant input assignments can be shortened", span)
        .with_fix("remove the redundant input assignment")
}

/// Detects a malformed lint directive.
#[derive(Default, Debug, Clone, Copy)]
pub struct RedundantInputAssignment;

impl Rule for RedundantInputAssignment {
    fn id(&self) -> &'static str {
        ID
    }

    fn description(&self) -> &'static str {
        "Flags malformed lint directives."
    }

    fn explanation(&self) -> &'static str {
        "Comments which begin with `#@` must only contain valid lint directives. Lint directives \
         must be on their own line, only preceded by whitespace. Lint directives should follow the \
         pattern `#@ <directive>: <value>` _exactly_. Currently the only accepted lint directive \
         is `except`. For example, `#@ except: MalformedLintDirective`."
    }

    fn tags(&self) -> TagSet {
        TagSet::new(&[Tag::Clarity, Tag::Correctness])
    }

    fn exceptable_nodes(&self) -> Option<&'static [wdl_ast::SyntaxKind]> {
        None
    }
}

impl Visitor for MalformedLintDirectiveRule {
    type State = Diagnostics;

    fn document(&mut self, _: &mut Self::State, _: VisitReason, _: &Document, _: SupportedVersion) {
        // This is intentionally empty, as this rule has no state.
    }

    fn comment(&mut self, state: &mut Self::State, comment: &Comment) {

    }
}

