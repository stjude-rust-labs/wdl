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
use wdl_ast::v1::BoundDecl;
use crate::Rule;
use crate::Tag;
use crate::TagSet;

const ID: &str = "RedundantNoneAssignment";

fn redundant_none_assignment(span: Span, name: &str) -> Diagnostic {
    Diagnostic::note("redundant None assignment")
        .with_rule(ID)
        .with_highlight(span)
        .with_fix(format!("can be shortened to {}", name))
}

#[derive(Default, Debug, Clone, Copy)]
pub struct RedundantNoneAssignment(Option<SupportedVersion>);

impl Rule for RedundantNoneAssignment {
    fn id(&self) -> &'static str {
        ID
    }

    fn description(&self) -> &'static str {
        "Flags redundant None assignments to optional inputs."
    }

    fn explanation(&self) -> &'static str {
        "Optional inputs with explicit None assignments can be simplified. For example, \
         String? foo = None can be shortened to String? foo."
    }

    fn tags(&self) -> TagSet {
        TagSet::new(&[Tag::Style])
    }

    fn exceptable_nodes(&self) -> Option<&'static [wdl_ast::SyntaxKind]> {
        Some(&[
            wdl_ast::SyntaxKind::VersionStatementNode,
            wdl_ast::SyntaxKind::WorkflowDefinitionNode,
            wdl_ast::SyntaxKind::TaskDefinitionNode, 
            wdl_ast::SyntaxKind::InputSectionNode,
            wdl_ast::SyntaxKind::BoundDeclNode,
        ])
    }
}

impl Visitor for RedundantNoneAssignment {
    type State = Diagnostics;

    fn document(&mut self, _: &mut Self::State, reason: VisitReason, _: &Document, version: SupportedVersion) {
        if reason == VisitReason::Exit {
            return;
        }
        *self = Self(Some(version));
    }

    fn bound_decl(&mut self, state: &mut Self::State, reason: VisitReason, decl: &BoundDecl) {
        if reason == VisitReason::Exit {
            return;
        }

        if !is_in_input_section(decl) {
            return;
        }

        if let type_token = decl.ty() {
            let type_text = type_token.to_string();
            if type_text.ends_with('?') {
                if let expr = decl.expr() {
                    if let Some(name_ref) = expr.as_name_ref() {
                        if name_ref.name().as_str() == "None" {
                            state.exceptable_add(
                                redundant_none_assignment(decl.span(), decl.name().as_str()),
                                SyntaxElement::from(decl.syntax().clone()),
                                &self.exceptable_nodes(),
                            );
                        }
                    }
                }
            }
        }
    }
}

fn is_in_input_section(decl: &BoundDecl) -> bool {
    let mut current = decl.syntax().parent();
    while let Some(parent) = current {
        if parent.kind() == wdl_ast::SyntaxKind::InputSectionNode {
            return true;
        }
        current = parent.parent();
    }
    false
}