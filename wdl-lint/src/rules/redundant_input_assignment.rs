//! A lint rule for flagging redundant input assignments

use std::fmt::Debug;

use rowan::ast::AstNode;
use wdl_ast::AstNodeExt;
use wdl_ast::AstToken;
use wdl_ast::Diagnostic;
use wdl_ast::Diagnostics;
use wdl_ast::Document;
use wdl_ast::Span;
use wdl_ast::SupportedVersion;
use wdl_ast::SyntaxElement;
use wdl_ast::VisitReason;
use wdl_ast::Visitor;
use wdl_ast::v1::CallStatement;

use crate::Rule;
use crate::Tag;
use crate::TagSet;

/// The identifier for the Redundant Input Assignment rule.
const ID: &str = "RedundantInputAssignment";

/// Create a "Redundant Input Assignment" diagnostic.
fn redundant_input_assignment(span: Span, name: &str) -> Diagnostic {
    Diagnostic::note("redundant input assignment")
        .with_rule(ID)
        .with_highlight(span)
        .with_fix(format!("can be shortened to `{}`", name))
}

/// Detects a redundant input assignment.
#[derive(Default, Debug, Clone, Copy)]
pub struct RedundantInputAssignment(Option<SupportedVersion>);

impl Rule for RedundantInputAssignment {
    fn id(&self) -> &'static str {
        ID
    }

    fn description(&self) -> &'static str {
        "Flags redundant input assignments."
    }

    fn explanation(&self) -> &'static str {
        "Redundant input assignments can be shortened in WDL versions >=v1.1 with an implicit \
         binding. For example, `{ input: a = a }` can be shortened to `{ input: a }`."
    }

    fn tags(&self) -> TagSet {
        TagSet::new(&[Tag::Style])
    }

    fn exceptable_nodes(&self) -> Option<&'static [wdl_ast::SyntaxKind]> {
        Some(&[
            wdl_ast::SyntaxKind::VersionStatementNode,
            wdl_ast::SyntaxKind::WorkflowDefinitionNode,
            wdl_ast::SyntaxKind::CallStatementNode,
            wdl_ast::SyntaxKind::CallInputItemNode,
        ])
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

    fn call_statement(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        stmt: &CallStatement,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        if let SupportedVersion::V1(minor_version) = self.0.expect("version should exist here") {
            if minor_version < wdl_ast::version::V1::One {
                return;
            }
            stmt.inputs().for_each(|input| {
                if let Some(expr) = input.expr() {
                    if let Some(expr_name) = expr.as_name_ref() {
                        if expr_name.name().as_str() == input.name().as_str() {
                            state.exceptable_add(
                                redundant_input_assignment(input.span(), input.name().as_str()),
                                SyntaxElement::from(input.syntax().clone()),
                                &self.exceptable_nodes(),
                            );
                        }
                    }
                }
            });
        }
    }
}
