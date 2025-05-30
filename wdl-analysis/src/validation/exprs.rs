//! Validation of scoped expressions.

use std::fmt;

use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::Diagnostic;
use wdl_ast::Span;
use wdl_ast::SupportedVersion;
use wdl_ast::v1;
use wdl_ast::v1::HintsKeyword;
use wdl_ast::v1::InputKeyword;
use wdl_ast::v1::OutputKeyword;
use wdl_ast::version::V1;

use crate::Diagnostics;
use crate::VisitReason;
use crate::Visitor;
use crate::document::Document;

/// Creates a "hints scope required" diagnostic.
fn hints_scope_required(literal: &Literal) -> Diagnostic {
    Diagnostic::error(format!(
        "`{literal}` literals can only be used within a hints section"
    ))
    .with_highlight(literal.span())
}

/// Creates a "literal cannot nest" diagnostic.
fn literal_cannot_nest(nested: &Literal, outer: &Literal) -> Diagnostic {
    Diagnostic::error(format!(
        "`{nested}` literals cannot be nested within `{outer}` literals"
    ))
    .with_label(
        format!("this `{nested}` literal cannot be nested"),
        nested.span(),
    )
    .with_label(format!("the outer `{outer}` literal is here"), outer.span())
}

/// Keeps track of the spans of a `hints`, `input`, or `output` literal.
#[derive(Debug, Clone, Copy)]
enum Literal {
    /// The literal is a `hints`.
    Hints(Span),
    /// The literal is an `input`.
    Input(Span),
    /// The literal is an `output`.
    Output(Span),
}

impl Literal {
    /// Gets the span of literal.
    fn span(&self) -> Span {
        match self {
            Self::Hints(s) | Self::Input(s) | Self::Output(s) => *s,
        }
    }
}

impl fmt::Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Hints(_) => write!(f, "hints"),
            Self::Input(_) => write!(f, "input"),
            Self::Output(_) => write!(f, "output"),
        }
    }
}

/// An AST visitor that ensures that certain expressions only appear in
/// acceptable scopes.
#[derive(Debug, Default)]
pub struct ScopedExprVisitor {
    /// The version of the document we're currently visiting.
    version: Option<SupportedVersion>,
    /// Whether or not we're currently in a `hints` section.
    in_hints_section: bool,
    /// The stack of literals encountered.
    literals: Vec<Literal>,
}

impl Visitor for ScopedExprVisitor {
    fn reset(&mut self) {
        self.version = None;
        self.in_hints_section = false;
        self.literals.clear();
    }

    fn document(
        &mut self,
        _: &mut Diagnostics,
        _: VisitReason,
        _: &Document,
        version: SupportedVersion,
    ) {
        self.version = Some(version);
    }

    fn task_hints_section(
        &mut self,
        _: &mut Diagnostics,
        reason: VisitReason,
        _: &v1::TaskHintsSection,
    ) {
        self.in_hints_section = reason == VisitReason::Enter;
    }

    fn expr(&mut self, diagnostics: &mut Diagnostics, reason: VisitReason, expr: &v1::Expr) {
        // Only visit expressions for WDL >=1.2
        if self.version.expect("should have a version") < SupportedVersion::V1(V1::Two) {
            return;
        }

        if reason == VisitReason::Exit {
            match expr {
                v1::Expr::Literal(v1::LiteralExpr::Hints(_))
                | v1::Expr::Literal(v1::LiteralExpr::Input(_))
                | v1::Expr::Literal(v1::LiteralExpr::Output(_)) => {
                    self.literals.pop();
                }
                _ => {}
            }
            return;
        }

        let literal = match expr {
            v1::Expr::Literal(v1::LiteralExpr::Hints(l)) => Literal::Hints(
                l.token::<HintsKeyword<_>>()
                    .expect("should have keyword")
                    .span(),
            ),
            v1::Expr::Literal(v1::LiteralExpr::Input(l)) => Literal::Input(
                l.token::<InputKeyword<_>>()
                    .expect("should have keyword")
                    .span(),
            ),
            v1::Expr::Literal(v1::LiteralExpr::Output(l)) => Literal::Output(
                l.token::<OutputKeyword<_>>()
                    .expect("should have keyword")
                    .span(),
            ),
            _ => return,
        };

        if self.in_hints_section {
            // Check for prohibited nesting
            let prohibited = match literal {
                Literal::Hints(_) => {
                    self.literals.len() > 1
                        || (self.literals.len() == 1
                            && matches!(self.literals[0], Literal::Hints(_)))
                }
                Literal::Input(_) | Literal::Output(_) => !self.literals.is_empty(),
            };

            if prohibited {
                let outer = self.literals.last().expect("should have an outer literal");
                diagnostics.add(literal_cannot_nest(&literal, outer));
            }
        } else {
            // Any use of these literals outside of a `hints` section is prohibited
            diagnostics.add(hints_scope_required(&literal));
        }

        self.literals.push(literal);
    }
}
