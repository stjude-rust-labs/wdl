//! Formatting of WDL v1.x expression elements.

use wdl_ast::SyntaxKind;

use crate::PreToken;
use crate::TokenStream;
use crate::Writable as _;
use crate::element::FormatElement;

/// Formats a [`LiteralString`](wdl_ast::v1::LiteralString).
pub fn format_literal_string(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    for child in element.children().expect("literal string children") {
        match child.element().kind() {
            SyntaxKind::SingleQuote => {
                stream.push_literal_in_place_of_token(
                    child.element().as_token().expect("token"),
                    "\"".to_owned(),
                );
            }
            _ => {
                (&child).write(stream);
            }
        }
    }
}

/// Formats a [`LiteralBoolean`](wdl_ast::v1::LiteralBoolean).
pub fn format_literal_boolean(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children().expect("literal boolean children");
    let bool = children.next().expect("literal boolean token");
    (&bool).write(stream);
    assert!(children.next().is_none());
}

/// Formats a [`NegationExpr`](wdl_ast::v1::NegationExpr).
pub fn format_negation_expr(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children().expect("negation expr children");
    let minus = children.next().expect("negation expr minus");
    assert!(minus.element().kind() == SyntaxKind::Minus);
    (&minus).write(stream);

    let expr = children.next().expect("negation expr expr");
    (&expr).write(stream);
    assert!(children.next().is_none());
}

/// Formats a [`LiteralInteger`](wdl_ast::v1::LiteralInteger).
pub fn format_literal_integer(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    for child in element.children().expect("literal integer children") {
        (&child).write(stream);
    }
}

/// Formats a [`LiteralFloat`](wdl_ast::v1::LiteralFloat).
pub fn format_literal_float(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    for child in element.children().expect("literal float children") {
        (&child).write(stream);
    }
}

/// Formats a [`NameReference`](wdl_ast::v1::NameRef).
pub fn format_name_ref(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children().expect("name ref children");
    let name = children.next().expect("name ref name");
    (&name).write(stream);
    assert!(children.next().is_none());
}

/// Formats a [`LiteralArray`](wdl_ast::v1::LiteralArray).
pub fn format_literal_array(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children().expect("literal array children");

    let open_bracket = children.next().expect("literal array open bracket");
    assert!(open_bracket.element().kind() == SyntaxKind::OpenBracket);
    (&open_bracket).write(stream);
    stream.increment_indent();

    let mut close_bracket = None;
    let mut commas = Vec::new();
    let items = children
        .filter(|child| {
            if child.element().kind() == SyntaxKind::CloseBracket {
                close_bracket = Some(child.to_owned());
                false
            } else if child.element().kind() == SyntaxKind::Comma {
                commas.push(child.to_owned());
                false
            } else {
                true
            }
        })
        .collect::<Vec<_>>();

    let mut commas = commas.iter();
    for item in items {
        (&item).write(stream);
        if let Some(comma) = commas.next() {
            (comma).write(stream);
            stream.end_line();
        } else {
            stream.push_literal(",".to_string(), SyntaxKind::Comma);
            stream.end_line();
        }
    }

    stream.decrement_indent();
    (&close_bracket.expect("literal array close bracket")).write(stream);
}

/// Formats a [`AccessExpr`](wdl_ast::v1::AccessExpr).
pub fn format_access_expr(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    for child in element.children().expect("access expr children") {
        (&child).write(stream);
    }
}

/// Formats a [`CallExpr`](wdl_ast::v1::CallExpr).
pub fn format_call_expr(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    for child in element.children().expect("call expr children") {
        (&child).write(stream);
        if child.element().kind() == SyntaxKind::Comma {
            stream.end_word();
        }
    }
}

/// Formats an [`IndexExpr`](wdl_ast::v1::IndexExpr).
pub fn format_index_expr(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    for child in element.children().expect("index expr children") {
        (&child).write(stream);
    }
}

/// Formats an [`AdditionExpr`](wdl_ast::v1::AdditionExpr).
pub fn format_addition_expr(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    for child in element.children().expect("addition expr children") {
        let kind = child.element().kind();
        if kind == SyntaxKind::Plus {
            stream.end_word();
        }
        (&child).write(stream);
        if kind == SyntaxKind::Plus {
            stream.end_word();
        }
    }
}

/// Formats a [`MultiplicationExpr`](wdl_ast::v1::MultiplicationExpr).
pub fn format_multiplication_expr(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    for child in element.children().expect("multiplication expr children") {
        let kind = child.element().kind();
        if kind == SyntaxKind::Asterisk {
            stream.end_word();
        }
        (&child).write(stream);
        if kind == SyntaxKind::Asterisk {
            stream.end_word();
        }
    }
}

/// Formats a [`LogicalAndExpr`](wdl_ast::v1::LogicalAndExpr).
pub fn format_logical_and_expr(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    for child in element.children().expect("logical and expr children") {
        let kind = child.element().kind();
        if kind == SyntaxKind::LogicalAnd {
            stream.end_word();
        }
        (&child).write(stream);
        if kind == SyntaxKind::LogicalAnd {
            stream.end_word();
        }
    }
}

/// Formats a [`LogicalNotExpr`](wdl_ast::v1::LogicalNotExpr).
pub fn format_logical_not_expr(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children().expect("logical not expr children");
    let not = children.next().expect("logical not expr not");
    assert!(not.element().kind() == SyntaxKind::Exclamation);
    (&not).write(stream);

    let expr = children.next().expect("logical not expr expr");
    (&expr).write(stream);
    assert!(children.next().is_none());
}

/// Formats a [`LogicalOrExpr`](wdl_ast::v1::LogicalOrExpr).
pub fn format_logical_or_expr(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    for child in element.children().expect("logical or expr children") {
        let should_end_word = child.element().kind() == SyntaxKind::LogicalOr;
        if should_end_word {
            stream.end_word();
        }
        (&child).write(stream);
        if should_end_word {
            stream.end_word();
        }
    }
}

/// Formats an [`EqualityExpr`](wdl_ast::v1::EqualityExpr).
pub fn format_equality_expr(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    for child in element.children().expect("equality expr children") {
        let should_end_word = child.element().kind() == SyntaxKind::Equal;
        if should_end_word {
            stream.end_word();
        }
        (&child).write(stream);
        if should_end_word {
            stream.end_word();
        }
    }
}

/// Formats a [`InequalityExpr`](wdl_ast::v1::InequalityExpr).
pub fn format_inequality_expr(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    for child in element.children().expect("inequality expr children") {
        let should_end_word = child.element().kind() == SyntaxKind::NotEqual;
        if should_end_word {
            stream.end_word();
        }
        (&child).write(stream);
        if should_end_word {
            stream.end_word();
        }
    }
}

/// Formats a [`LessExpr`](wdl_ast::v1::LessExpr).
pub fn format_less_expr(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    for child in element.children().expect("less expr children") {
        let should_end_word = child.element().kind() == SyntaxKind::Less;
        if should_end_word {
            stream.end_word();
        }
        (&child).write(stream);
        if should_end_word {
            stream.end_word();
        }
    }
}

/// Formats a [`LessEqualExpr`](wdl_ast::v1::LessEqualExpr).
pub fn format_less_equal_expr(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    for child in element.children().expect("less equal expr children") {
        let should_end_word = child.element().kind() == SyntaxKind::LessEqual;
        if should_end_word {
            stream.end_word();
        }
        (&child).write(stream);
        if should_end_word {
            stream.end_word();
        }
    }
}

/// Formats a [`GreaterExpr`](wdl_ast::v1::GreaterExpr).
pub fn format_greater_expr(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    for child in element.children().expect("greater expr children") {
        let should_end_word = child.element().kind() == SyntaxKind::Greater;
        if should_end_word {
            stream.end_word();
        }
        (&child).write(stream);
        if should_end_word {
            stream.end_word();
        }
    }
}

/// Formats a [`GreaterEqualExpr`](wdl_ast::v1::GreaterEqualExpr).
pub fn format_greater_equal_expr(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    for child in element.children().expect("greater equal expr children") {
        let should_end_word = child.element().kind() == SyntaxKind::GreaterEqual;
        if should_end_word {
            stream.end_word();
        }
        (&child).write(stream);
        if should_end_word {
            stream.end_word();
        }
    }
}

/// Formats a [`ParenthesizedExpr`](wdl_ast::v1::ParenthesizedExpr).
pub fn format_parenthesized_expr(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    for child in element.children().expect("parenthesized expr children") {
        (&child).write(stream);
    }
}

/// Formats an [`IfExpr`](wdl_ast::v1::IfExpr).
pub fn format_if_expr(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children().expect("if expr children");

    let if_keyword = children.next().expect("if keyword");
    assert!(if_keyword.element().kind() == SyntaxKind::IfKeyword);
    (&if_keyword).write(stream);
    stream.end_word();

    for child in children {
        let kind = child.element().kind();
        let should_end_word = kind == SyntaxKind::ThenKeyword || kind == SyntaxKind::ElseKeyword;
        if should_end_word {
            stream.end_word();
        }
        (&child).write(stream);
        if should_end_word {
            stream.end_word();
        }
    }
}
