//! A lint rule for flagging redundant input assignments

use std::fmt::Debug;
use wdl_ast::{AstToken, VersionStatement};
use wdl_ast::Diagnostic;
use wdl_ast::Diagnostics;
use wdl_ast::Document;
use wdl_ast::Span;
use wdl_ast::SupportedVersion;
use wdl_ast::v1::{Decl, InputSection};
use wdl_ast::VisitReason;
use wdl_ast::Visitor;

use crate::Rule;
use crate::Tag;
use crate::TagSet;

/// The identifier for the Redundant Input Assignment rule.
const ID: &str = "RedundantInputAssignment";

/// Create a "Redundant Input Assignment" diagnostic.
fn redundant_input_assignment(span: Span) -> Diagnostic {
    Diagnostic::note("redundant input assignments can be shortened")
        .with_rule(ID)
        .with_label("redundant input assignments can be shortened", span)
}

/// Detects a malformed lint directive.
#[derive(Default, Debug, Clone, Copy)]
pub struct RedundantInputAssignment {
    included_version: bool,
}

impl Rule for RedundantInputAssignment {
    fn id(&self) -> &'static str {
        ID
    }

    fn description(&self) -> &'static str {
        "Flags redundant input assignments."
    }

    fn explanation(&self) -> &'static str {
        "Input assignments that are redundant can be shortened. For example, `{ input: a = a }` \
        can be shortened to `{ input: a }`."
    }

    fn tags(&self) -> TagSet {
        TagSet::new(&[Tag::Clarity, Tag::Correctness])
    }

    fn exceptable_nodes(&self) -> Option<&'static [wdl_ast::SyntaxKind]> {
        None
    }
}

impl Visitor for RedundantInputAssignment {
    type State = Diagnostics;

    fn document(&mut self, _: &mut Self::State, _: VisitReason, _: &Document, _: SupportedVersion) {
        // This is intentionally empty, as this rule has no state.
    }

    fn version_statement(&mut self, state: &mut Self::State, reason: VisitReason, stmt: &VersionStatement) {
        // this current takes into account for any possible patch versions, which might be unnecessary
        let binding = stmt.version();
        let split_versions = binding.as_str().split(".").collect::<Vec<_>>();
        if split_versions[0] == "1" && split_versions[1] == "0" {
            self.included_version = false;
        } else {
            self.included_version = true;
        }
    }

    fn input_section(&mut self, state: &mut Self::State, reason: VisitReason, section: &InputSection) {
        /// if the version statement is earlier than 1.1 return
        if !self.included_version {
            return
        }

        /// for each declaration in the input section,
        section.declarations().for_each(|dcl| {
            if let Some(expr) = dcl.expr() {
                // if dcl.name().as_str() == expr {
                //     state.add(redundant_input_assignment(Span::new(0, 12)));
                // }
            }
        });
    }
}