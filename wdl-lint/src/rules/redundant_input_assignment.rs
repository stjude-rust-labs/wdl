//! A lint rule for flagging redundant input assignments

use std::fmt::Debug;

use wdl_ast::Ast::V1;
use wdl_ast::Diagnostic;
use wdl_ast::Diagnostics;
use wdl_ast::Document;
use wdl_ast::Node::NameRef;
use wdl_ast::Span;
use wdl_ast::SupportedVersion;
use wdl_ast::VisitReason;
use wdl_ast::Visitor;
use wdl_ast::v1::InputSection;

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

    fn document(
        &mut self,
        _: &mut Self::State,
        reason: VisitReason,
        _: &Document,
        version: SupportedVersion,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        // Reset the visitor upon document entry
        *self = Self(Some(version));
    }

    fn input_section(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        section: &InputSection,
    ) {
        /// if the version statement is earlier than 1.1 return
        if let SupportedVersion::V1(minor_version) = self.0.expect("version should exist here") {
            if minor_version < V1::One {
                return;
            }

            section
                .declarations()
                .for_each(|dcl| {
                    if let Some(expr) = dcl.expr() {
                        if let expr_name = NameRef(expr.clone()) {
                            if (expr_name == dcl.name()) {
                                state.push(redundant_input_assignment(expr_name.to_span()));
                            }
                        }
                    }
                });
        }
    }
}
