use wdl_analysis::Diagnostics;
use wdl_analysis::VisitReason;
use wdl_analysis::Visitor;
use wdl_ast::AstNode;
use wdl_ast::Diagnostic;
use wdl_ast::Span;
use wdl_ast::SyntaxElement;
use wdl_ast::SyntaxKind;
use wdl_ast::v1::BoundDecl;
use wdl_ast::v1::Type;

use crate::Rule;
use crate::Tag;
use crate::TagSet;

const ID: &str = "ObjectToStruct";

fn prefer_struct_over_object(span: Span) -> Diagnostic {
    Diagnostic::warning("Use of `Object` type where a `Struct` could be defined.")
        .with_rule(ID)
        .with_highlight(span)
        .with_fix("Define a specific `Struct` type and use it instead of `Object`.")
}

#[derive(Default, Debug, Clone, Copy)]
pub struct ObjectToStructRule;

impl Rule for ObjectToStructRule {
    fn id(&self) -> &'static str {
        ID
    }

    fn description(&self) -> &'static str {
        "Discourages direct use of `Object` type, recommending `Struct` definitions for better \
         type safety and clarity."
    }

    fn explanation(&self) -> &'static str {
        "WDL supports object-to-struct coercion, allowing `Object` literals to be assigned to \
         `Struct`-typed variables. However, declarations should use specific `Struct` types \
         directly whenever the data structure is fixed and known. Using explicit `Struct`s \
         enhances type safety, improves code readability, and provides more precise information \
         for downstream analysis tools. Declaring variables as `Object` when a `Struct` could be \
         defined can obscure the intended data schema."
    }

    fn tags(&self) -> TagSet {
        TagSet::new(&[Tag::Clarity])
    }

    fn exceptable_nodes(&self) -> Option<&'static [wdl_ast::SyntaxKind]> {
        Some(&[
            SyntaxKind::VersionStatementNode,
            SyntaxKind::TaskDefinitionNode,
            SyntaxKind::WorkflowDefinitionNode,
            SyntaxKind::BoundDeclNode,
            SyntaxKind::UnboundDeclNode,
        ])
    }

    fn related_rules(&self) -> &[&'static str] {
        &["DeprecatedObject"] // since deprecated object rule exists
    }
}

impl Visitor for ObjectToStructRule {
    fn reset(&mut self) {
        *self = Self::default();
    }

    fn bound_decl(&mut self, diagnostics: &mut Diagnostics, reason: VisitReason, decl: &BoundDecl) {
        if reason == VisitReason::Exit {
            return;
        }

        // we check if the expression that is being initialised to the declaration is an
        // object type or not
        match decl.expr() {
            wdl_ast::v1::Expr::Literal(wdl_ast::v1::LiteralExpr::Object(token)) => {
                diagnostics.exceptable_add(
                    prefer_struct_over_object(token.span()),
                    SyntaxElement::from(decl.inner().clone()),
                    &self.exceptable_nodes(),
                );
            }
            _ => {}
        }
    }

    // this is redundant because of deprecated object rule !!!
    fn unbound_decl(
        &mut self,
        diagnostics: &mut Diagnostics,
        reason: VisitReason,
        decl: &wdl_ast::v1::UnboundDecl,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        if let Type::Object(object_type_token) = decl.ty() {
            diagnostics.exceptable_add(
                prefer_struct_over_object(object_type_token.span()),
                SyntaxElement::from(decl.inner().clone()),
                &self.exceptable_nodes(),
            )
        }
    }
}
