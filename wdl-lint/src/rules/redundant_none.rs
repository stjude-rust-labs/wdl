use crate::Rule;
use crate::Tag;
use crate::TagSet;
use std::fmt::Debug;
use wdl_ast::AstToken;
use wdl_ast::Diagnostic;
use wdl_ast::Diagnostics;
use wdl_ast::Document;
use wdl_ast::Span;
use wdl_ast::SupportedVersion;
use wdl_ast::SyntaxElement;
use wdl_ast::VisitReason;
use wdl_ast::Visitor;
use wdl_ast::v1::InputSection;

const ID: &str = "RedundantNoneAssignment";

fn redundant_none_assignment(span: Span, name: &str) -> Diagnostic {
    Diagnostic::note("redundant None assignment")
        .with_rule(ID)
        .with_highlight(span)
        .with_fix(format!("can be shortened to {}", name))
}

#[derive(Default, Debug, Clone, Copy)]
pub struct RedundantNoneAssignment;

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
        TagSet::new(&[Tag::Clarity])
    }

    fn exceptable_nodes(&self) -> Option<&'static [wdl_ast::SyntaxKind]> {
        Some(&[
            wdl_ast::SyntaxKind::VersionStatementNode,
            wdl_ast::SyntaxKind::InputSectionNode,
            wdl_ast::SyntaxKind::BoundDeclNode,
        ])
    }
}

impl Visitor for RedundantNoneAssignment {
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
        *self = Default::default();
    }

    fn input_section(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        section: &InputSection,
    ) {
        if reason == VisitReason::Exit {
            return;
        }
        section.declarations().for_each(|decl| {
            if let token = decl.ty() {
                if token.is_optional() {
                    if let Some(expr) = decl.expr() {
                        if let Some(name_ref) = expr.as_literal().unwrap().as_none() {
                            let text_range = decl.syntax().text_range();
                            let span =
                                Span::from(text_range.start().into()..text_range.end().into());
                            state.exceptable_add(
                                redundant_none_assignment(span, decl.name().as_str()),
                                SyntaxElement::from(decl.syntax().clone()),
                                &self.exceptable_nodes(),
                            );
                        }
                    }
                }
            }
        });
    }
}
