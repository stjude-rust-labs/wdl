//! A lint rule for spacing of expressions.

use rowan::NodeOrToken;
use wdl_ast::v1::Expr;
use wdl_ast::AstNode;
use wdl_ast::Diagnostic;
use wdl_ast::Diagnostics;
use wdl_ast::Document;
use wdl_ast::Span;
use wdl_ast::SupportedVersion;
use wdl_ast::SyntaxKind;
use wdl_ast::SyntaxNode;
use wdl_ast::SyntaxToken;
use wdl_ast::ToSpan;
use wdl_ast::VisitReason;
use wdl_ast::Visitor;

use crate::Rule;
use crate::Tag;
use crate::TagSet;

/// The identifier for the expression spacing rule.
const ID: &str = "ExpressionSpacing";

/// Reports disallowed whitespace after prefix operators.
fn prefix_whitespace(span: Span) -> Diagnostic {
    Diagnostic::note("prefix operators may not contain whitespace")
        .with_rule(ID)
        .with_highlight(span)
        .with_fix("remove the internal whitespace")
}

/// Reports missing following whitespace around operators
fn missing_surrounding_whitespace(span: Span) -> Diagnostic {
    Diagnostic::note("operators must be surrounded by whitespace")
        .with_rule(ID)
        .with_highlight(span)
        .with_fix("add a space before and after this operator")
}

/// Reports missing preceding whitespace around operators
fn missing_preceding_whitespace(span: Span) -> Diagnostic {
    Diagnostic::note("operators must be preceded by whitespace")
        .with_rule(ID)
        .with_highlight(span)
        .with_fix("add a space before this operator")
}

/// Reports missing following whitespace around operators
fn missing_following_whitespace(span: Span) -> Diagnostic {
    Diagnostic::note("operators must be followed by whitespace")
        .with_rule(ID)
        .with_highlight(span)
        .with_fix("add a space after this operator")
}

/// Report disallowed space
fn disallowed_space(span: Span) -> Diagnostic {
    Diagnostic::note("this space is not allowed")
        .with_rule(ID)
        .with_highlight(span)
        .with_fix("remove the space")
}

/// Reports missing preceding whitespace around assignments
fn assignment_missing_preceding_whitespace(span: Span) -> Diagnostic {
    Diagnostic::note("assignments must be preceded by whitespace")
        .with_rule(ID)
        .with_highlight(span)
        .with_fix("add a space before this assignment")
}

/// Reports missing following whitespace around assignments
fn assignment_missing_following_whitespace(span: Span) -> Diagnostic {
    Diagnostic::note("assignments must be followed by whitespace")
        .with_rule(ID)
        .with_highlight(span)
        .with_fix("add a space after this assignment")
}

/// Reports missing following whitespace around assignments
fn assignment_missing_surrounding_whitespace(span: Span) -> Diagnostic {
    Diagnostic::note("assignments must be surrounded by whitespace")
        .with_rule(ID)
        .with_highlight(span)
        .with_fix("add a space before and after this assignment")
}

/// Detects improperly spaced expressions.
#[derive(Default, Debug, Clone, Copy)]
pub struct ExpressionSpacingRule;

impl Rule for ExpressionSpacingRule {
    fn id(&self) -> &'static str {
        ID
    }

    fn description(&self) -> &'static str {
        "Ensures that WDL expressions are properly spaced."
    }

    fn explanation(&self) -> &'static str {
        "Proper spacing is important for readability and consistency. This rule ensures that \
         expressions are spaced properly. The following tokens should be surrounded by whitespace \
         when used as an infix: `=`, `==`, `!=`, `&&`, `||`, `<`, `<=`, `>`, `>=`, `+`, `-`, `*`, \
         `/`, and `%`. The following tokens should not be followed by whitespace when used as a \
         prefix: `+`, `-`, and `!`. Opening brackets (`(`, `[`) should not be followed by a space, \
         but may be followed by a newline. Closing brackets (`)`, `]`) should not be preceded by a \
         space, but may be preceded by a newline. Sometimes a long expression will exceed the \
         maximum line width. In these cases, one or more linebreaks must be introduced. Line \
         continuations should be indented one more level than the beginning of the expression. \
         There should never be more than one level of indentation change per-line. If bracketed \
         content (things between `()` or `[]`) must be split onto multiple lines, a newline should \
         follow the opening bracket, the contents should be indented an additional level, then the \
         closing bracket should be de-indented to match the indentation of the opening bracket. If \
         you are line splitting an expression on an infix operator, the operator and at least the \
         beginning of the RHS operand should be on the continued line. (i.e. an operator should \
         not be on a line by itself.) If you are using the `if...then...else...` construct as part \
         of your expression and it needs to be line split, the entire construct should be wrapped \
         in parentheses (`()`). The opening parenthesis should be immediately followed by a \
         newline. `if`, `then`, and `else` should all start a line one more level of indentation \
         than the wrapping paratheses. The closing parenthesis should be on the same level of \
         indentation as the opening parenthesis. If you are using the `if...then...else...` \
         construct on one line, it does not need to be wrapped in parentheses. However, if any of \
         the 3 clauses are more complex than a single identifier, they should be wrapped in \
         parentheses. Sometimes a developer will choose to line split an expression despite it \
         being able to all fit on one line that is <=90 characters wide. That is perfectly \
         acceptable, though you may notice in the below examples the single line form can be more \
         readable. There is 'wiggle' room allowed by the above rules. This is intentional, and \
         allows developers to choose a more compact or a more spaced out expression."
    }

    fn tags(&self) -> TagSet {
        TagSet::new(&[Tag::Spacing])
    }
}

impl Visitor for ExpressionSpacingRule {
    type State = Diagnostics;

    fn document(
        &mut self,
        _: &mut Self::State,
        reason: VisitReason,
        _: &Document,
        _: SupportedVersion,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        // Reset the visitor upon document entry
        *self = Default::default();
    }

    fn expr(&mut self, state: &mut Self::State, reason: VisitReason, expr: &Expr) {
        if reason == VisitReason::Exit {
            return;
        }

        match expr {
            Expr::LogicalNot(_) | Expr::Negation(_) => {
                // No following spacing allowed
                if expr
                    .syntax()
                    .children_with_tokens()
                    .filter(|t| t.kind() == SyntaxKind::Whitespace)
                    .count()
                    > 0
                {
                    state.add(prefix_whitespace(expr.syntax().text_range().to_span()));
                }
            }
            Expr::Parenthesized(_) => {
                // Find the actual open and close parentheses
                let open = expr
                    .syntax()
                    .children_with_tokens()
                    .filter(|t| t.kind() == SyntaxKind::OpenParen)
                    .next()
                    .expect("parenthesized expression should have an opening parenthesis");
                let close = expr
                    .syntax()
                    .children_with_tokens()
                    .filter(|t| t.kind() == SyntaxKind::CloseParen)
                    .next()
                    .expect("parenthesized expression should have an closing parenthesis");

                // The opening parenthesis can be preceded by whitespace, another open
                // parenthesis, or a negation (!). The parenthesized expression
                // can be the first thing at its level if it is wrapped in a
                // EqualityExpressionNode.
                if let Some(prev) = expr.syntax().prev_sibling_or_token() {
                    match prev.kind() {
                        SyntaxKind::Whitespace
                        | SyntaxKind::OpenParen
                        | SyntaxKind::NegationExprNode
                        | SyntaxKind::Exclamation
                        | SyntaxKind::Plus // This and all below will report on those tokens.
                        | SyntaxKind::Minus
                        | SyntaxKind::Asterisk
                        | SyntaxKind::Exponentiation
                        | SyntaxKind::Slash
                        | SyntaxKind::Less
                        | SyntaxKind::LessEqual
                        | SyntaxKind::Greater
                        | SyntaxKind::GreaterEqual
                        | SyntaxKind::Percent
                        | SyntaxKind::LogicalAnd
                        | SyntaxKind::LogicalOr => {}
                        _ => {
                            // opening parens should be preceded by whitespace
                            state.add(missing_preceding_whitespace(open.text_range().to_span()));
                        }
                    }
                } else {
                    // No prior elements, so we need to go up a level.
                    if let Some(parent) = expr.syntax().parent() {
                        if let Some(parent_prev) = parent.prev_sibling_or_token() {
                            if parent_prev.kind() != SyntaxKind::Whitespace {
                                // opening parens should be preceded by whitespace
                                state.add(missing_preceding_whitespace(
                                    parent.text_range().to_span(),
                                ));
                            }
                        }
                    } else {
                        unreachable!(
                            "parenthesized expression should have a prior sibling or a parent"
                        );
                    }
                }

                // Opening parenthesis cannot be followed by a space, but can be followed by a
                // newline.
                if let Some(open_next) = open.next_sibling_or_token() {
                    if open_next.kind() == SyntaxKind::Whitespace {
                        let token = open_next.as_token().expect("should be a token");
                        if token.text().starts_with(" ") {
                            // opening parens should not be followed by non-newline whitespace
                            state.add(disallowed_space(token.text_range().to_span()));
                        }
                    }
                }

                // Closing parenthesis should not be preceded by a space, but can be preceded by
                // a newline.
                if let Some(close_prev) = close.prev_sibling_or_token() {
                    if close_prev.kind() == SyntaxKind::Whitespace
                        && !close_prev
                            .as_token()
                            .expect("should be a token")
                            .text()
                            .contains("\n")
                    {
                        // closing parenthesis should not be preceded by whitespace without a
                        // newline
                        state.add(disallowed_space(close_prev.text_range().to_span()));
                    }
                }
            }
            Expr::LogicalAnd(_) | Expr::LogicalOr(_) => {
                // find the operator
                let op = expr
                    .syntax()
                    .children_with_tokens()
                    .filter(|t| match t.kind() {
                        SyntaxKind::LogicalAnd | SyntaxKind::LogicalOr => true,
                        _ => false,
                    })
                    .next()
                    .expect("expression node should have an operator");

                check_required_surrounding_ws(state, &op);
            }
            Expr::Equality(_) | Expr::Inequality(_) => {
                // find the operator
                let op = expr
                    .syntax()
                    .children_with_tokens()
                    .filter(|t| match t.kind() {
                        SyntaxKind::Equal | SyntaxKind::NotEqual => true,
                        _ => false,
                    })
                    .next()
                    .expect("expression node should have an operator");

                check_required_surrounding_ws(state, &op);
            }
            Expr::Addition(_)
            | Expr::Subtraction(_)
            | Expr::Multiplication(_)
            | Expr::Division(_)
            | Expr::Modulo(_)
            | Expr::Exponentiation(_) => {
                // find the operator
                let op = expr
                    .syntax()
                    .children_with_tokens()
                    .filter(|t| match t.kind() {
                        SyntaxKind::Plus
                        | SyntaxKind::Minus
                        | SyntaxKind::Asterisk
                        | SyntaxKind::Slash
                        | SyntaxKind::Percent
                        | SyntaxKind::Exponentiation => true,
                        _ => false,
                    })
                    .next()
                    .expect("expression node should have an operator");

                // Infix operators must be surrounded by whitespace
                if op.prev_sibling_or_token().map(|t| t.kind()) != Some(SyntaxKind::Whitespace) {
                    // assignments must be preceded by whitespace
                    state.add(missing_preceding_whitespace(op.text_range().to_span()));
                }
                if op.next_sibling_or_token().map(|t| t.kind()) != Some(SyntaxKind::Whitespace) {
                    // assignments must be followed by whitespace
                    state.add(missing_following_whitespace(op.text_range().to_span()));
                }
            }
            Expr::Less(_) | Expr::LessEqual(_) | Expr::Greater(_) | Expr::GreaterEqual(_) => {
                // find the operator
                let op = expr
                    .syntax()
                    .children_with_tokens()
                    .filter(|t| match t.kind() {
                        SyntaxKind::Less
                        | SyntaxKind::LessEqual
                        | SyntaxKind::Greater
                        | SyntaxKind::GreaterEqual => true,
                        _ => false,
                    })
                    .next()
                    .expect("expression node should have an operator");

                check_required_surrounding_ws(state, &op);
            }
            Expr::If(_) => {
                // find the if keyword
                let if_keyword = expr
                    .syntax()
                    .children_with_tokens()
                    .filter(|t| t.kind() == SyntaxKind::IfKeyword)
                    .next()
                    .expect("if expression node should have an if keyword");
                let then_keyword = expr
                    .syntax()
                    .children_with_tokens()
                    .filter(|t| t.kind() == SyntaxKind::ThenKeyword)
                    .next()
                    .expect("if expression node should have a then keyword");
                let else_keyword = expr
                    .syntax()
                    .children_with_tokens()
                    .filter(|t| t.kind() == SyntaxKind::ElseKeyword)
                    .next();
            }
            Expr::Index(_) => {
                let open_bracket = expr
                    .syntax()
                    .children_with_tokens()
                    .find(|t| t.kind() == SyntaxKind::OpenBracket)
                    .expect("index expression node should have an opening bracket");
                let close_bracket = expr
                    .syntax()
                    .children_with_tokens()
                    .find(|t| t.kind() == SyntaxKind::CloseBracket)
                    .expect("index expression node should have a closing bracket");

                let checks = vec![
                    open_bracket.prev_sibling_or_token().filter(|t| t.kind() == SyntaxKind::Whitespace),
                    open_bracket.next_sibling_or_token().filter(|t| t.kind() == SyntaxKind::Whitespace),
                    close_bracket.prev_sibling_or_token().filter(|t| t.kind() == SyntaxKind::Whitespace),
                    close_bracket.next_sibling_or_token().filter(|t| t.kind() == SyntaxKind::Whitespace),
                ];

                checks
                    .iter()
                    .for_each(|f|{
                    if let Some(ws) = f {
                        state.add(disallowed_space(ws.text_range().to_span()));
                    }
                });
            }
            Expr::Access(acc) => {
                let op = acc.syntax().children_with_tokens().find(|t| t.kind() == SyntaxKind::Dot)
                    .expect("access expression node should have a dot operator");
                let before_ws =
                    op.prev_sibling_or_token().filter(|t| t.kind() == SyntaxKind::Whitespace);
                let after_ws =
                    op.next_sibling_or_token().filter(|t| t.kind() == SyntaxKind::Whitespace);

                if let Some(ws) = before_ws {
                    state.add(disallowed_space(ws.text_range().to_span()));
                }
                if let Some(ws) = after_ws {
                    state.add(disallowed_space(ws.text_range().to_span()));
                }
            }
            // Expr::Literal, Expr::Name, Expr::Call
            _ => {}
        }
    }

    fn bound_decl(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        decl: &wdl_ast::v1::BoundDecl,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        if let Some(assign) = decl
            .syntax()
            .descendants_with_tokens()
            .find(|t| t.kind() == SyntaxKind::Assignment)
        {
            let before_ws =
                assign.prev_sibling_or_token().map(|t| t.kind()) == Some(SyntaxKind::Whitespace);
            let after_ws =
                assign.next_sibling_or_token().map(|t| t.kind()) == Some(SyntaxKind::Whitespace);

            if !before_ws && !after_ws {
                // assignments must be surrounded by whitespace
                state.add(assignment_missing_surrounding_whitespace(
                    assign.text_range().to_span(),
                ));
            } else if !before_ws {
                // assignments must be preceded by whitespace
                state.add(assignment_missing_preceding_whitespace(
                    assign.text_range().to_span(),
                ));
            } else if !after_ws {
                // assignments must be followed by whitespace
                state.add(assignment_missing_following_whitespace(
                    assign.text_range().to_span(),
                ));
            }
        }
    }
}

fn check_required_surrounding_ws(
    state: &mut Diagnostics,
    op: &NodeOrToken<SyntaxNode, SyntaxToken>,
) {
    let before_ws = op.prev_sibling_or_token().map(|t| t.kind()) == Some(SyntaxKind::Whitespace);
    let after_ws = op.next_sibling_or_token().map(|t| t.kind()) == Some(SyntaxKind::Whitespace);

    if !before_ws && !after_ws {
        // must be surrounded by whitespace
        state.add(missing_surrounding_whitespace(op.text_range().to_span()));
    } else if !before_ws {
        // must be preceded by whitespace
        state.add(missing_preceding_whitespace(op.text_range().to_span()));
    } else if !after_ws {
        // must be followed by whitespace
        state.add(missing_following_whitespace(op.text_range().to_span()));
    }
}
