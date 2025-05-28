use wdl_analysis::Diagnostics;
use wdl_analysis::VisitReason;
use wdl_analysis::Visitor;
use wdl_ast::AstNode;
use wdl_ast::Diagnostic;
use wdl_ast::Span;
use wdl_ast::SyntaxElement;
use wdl_ast::SyntaxKind;
use wdl_ast::v1::Expr::Literal;
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

// A rule that discourages object to struct coercion
#[derive(Default, Debug)]
pub struct ObjectToStructRule;

impl Rule for ObjectToStructRule {
    fn id(&self) -> &'static str {
        ID
    }

    fn description(&self) -> &'static str {
        "Discourages object to struct coercion"
    }

    fn explanation(&self) -> &'static str {
        "WDL allows assigning `Object` literals to `Struct`-typed variables, but this hides the \
         intended schema. Prefer defining explicit `Struct` types to improve type safety, clarity, \
         and tool support."
    }

    fn tags(&self) -> TagSet {
        TagSet::new(&[Tag::Clarity, Tag::Deprecated])
    }

    fn exceptable_nodes(&self) -> Option<&'static [wdl_ast::SyntaxKind]> {
        Some(&[SyntaxKind::BoundDeclNode])
    }

    fn related_rules(&self) -> &[&'static str] {
        &["DeprecatedObject"] // since deprecated object rule exists
    }
}

impl Visitor for ObjectToStructRule {
    fn reset(&mut self) {
        *self = Self::default();
    }

    fn bound_decl(
        &mut self,
        diagnostics: &mut Diagnostics,
        reason: VisitReason,
        decl: &wdl_ast::v1::BoundDecl,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        match decl.ty() {
            // if it is a reference to a variable
            // likely ...
            // struct myStruct;
            // myStruct = Object {}
            Type::Ref(_) => {
                // we check if it is bounded to a object literal declaration
                if let Literal(wdl_ast::v1::LiteralExpr::Object(object_type_token)) = decl.expr() {
                    diagnostics.exceptable_add(
                        prefer_struct_over_object(object_type_token.span()),
                        SyntaxElement::from(decl.inner().clone()),
                        &self.exceptable_nodes(),
                    )
                }
            }
            _ => {}
        }
    }
}
