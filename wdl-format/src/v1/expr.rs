//! Formatting of WDL v1.x expression elements.

use wdl_ast::SyntaxKind;

use crate::PreToken;
use crate::TokenStream;
use crate::Writable as _;
use crate::element::FormatElement;

/// Formats a [`SepOption`](wdl_ast::v1::SepOption).
pub fn format_sep_option(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children().expect("sep option children");

    let sep_keyword = children.next().expect("sep keyword");
    assert!(sep_keyword.element().kind() == SyntaxKind::Ident);
    (&sep_keyword).write(stream);

    let equals = children.next().expect("sep equals");
    assert!(equals.element().kind() == SyntaxKind::Assignment);
    (&equals).write(stream);

    let sep_value = children.next().expect("sep value");
    assert!(sep_value.element().kind() == SyntaxKind::LiteralStringNode);
    (&sep_value).write(stream);
    stream.end_word();

    assert!(children.next().is_none());
}

/// Formats a [`DefaultOption`](wdl_ast::v1::DefaultOption).
pub fn format_default_option(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children().expect("default option children");

    let default_keyword = children.next().expect("default keyword");
    assert!(default_keyword.element().kind() == SyntaxKind::Ident);
    (&default_keyword).write(stream);

    let equals = children.next().expect("default equals");
    assert!(equals.element().kind() == SyntaxKind::Assignment);
    (&equals).write(stream);

    let default_value = children.next().expect("default value");
    (&default_value).write(stream);
    stream.end_word();

    assert!(children.next().is_none());
}

/// Formats a [`TrueFalseOption`](wdl_ast::v1::TrueFalseOption).
pub fn format_true_false_option(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children().expect("true false option children");

    let first_keyword = children.next().expect("true false option first keyword");
    let first_keyword_kind = first_keyword.element().kind();
    assert!(
        first_keyword_kind == SyntaxKind::TrueKeyword
            || first_keyword_kind == SyntaxKind::FalseKeyword
    );

    let first_equals = children.next().expect("true false option first equals");
    assert!(first_equals.element().kind() == SyntaxKind::Assignment);

    let first_value = children.next().expect("true false option first value");

    let second_keyword = children.next().expect("true false option second keyword");
    let second_keyword_kind = second_keyword.element().kind();
    assert!(
        second_keyword_kind == SyntaxKind::TrueKeyword
            || second_keyword_kind == SyntaxKind::FalseKeyword
    );

    let second_equals = children.next().expect("true false option second equals");
    assert!(second_equals.element().kind() == SyntaxKind::Assignment);

    let second_value = children.next().expect("true false option second value");

    if first_keyword_kind == SyntaxKind::TrueKeyword {
        assert!(second_keyword_kind == SyntaxKind::FalseKeyword);
        (&first_keyword).write(stream);
        (&first_equals).write(stream);
        (&first_value).write(stream);
        stream.end_word();
        (&second_keyword).write(stream);
        (&second_equals).write(stream);
        (&second_value).write(stream);
    } else {
        assert!(second_keyword_kind == SyntaxKind::TrueKeyword);
        (&second_keyword).write(stream);
        (&second_equals).write(stream);
        (&second_value).write(stream);
        stream.end_word();
        (&first_keyword).write(stream);
        (&first_equals).write(stream);
        (&first_value).write(stream);
    }
    stream.end_word();

    assert!(children.next().is_none());
}

/// Formats a [`Placeholder`](wdl_ast::v1::Placeholder).
pub fn format_placeholder(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children().expect("placeholder children");

    let open = children.next().expect("placeholder open");
    assert!(open.element().kind() == SyntaxKind::PlaceholderOpen);
    let syntax = open.element().syntax();
    let text = syntax.as_token().expect("token").text();
    match text {
        "${" => {
            stream.push_literal_in_place_of_token(
                open.element().as_token().expect("token"),
                "~{".to_owned(),
            );
        }
        "~{" => {
            (&open).write(stream);
        }
        _ => {
            unreachable!("unexpected placeholder open: {:?}", text);
        }
    }

    for child in children {
        (&child).write(stream);
    }
}

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
            SyntaxKind::DoubleQuote => {
                (&child).write(stream);
            }
            SyntaxKind::LiteralStringText => {
                let mut replacement = String::new();
                let syntax = child.element().syntax();
                let mut chars = syntax.as_token().expect("token").text().chars().peekable();
                let mut prev_c = None;
                while let Some(c) = chars.next() {
                    match c {
                        '\\' => {
                            if let Some(next_c) = chars.peek() {
                                if *next_c == '\'' {
                                    // Do not write this backslash
                                    prev_c = Some(c);
                                    continue;
                                }
                            }
                            replacement.push(c);
                        }
                        '"' => {
                            if let Some(pc) = prev_c {
                                if pc != '\\' {
                                    replacement.push('\\');
                                }
                            }
                            replacement.push(c);
                        }
                        _ => {
                            replacement.push(c);
                        }
                    }
                    prev_c = Some(c);
                }

                stream.push_literal_in_place_of_token(
                    child.element().as_token().expect("token"),
                    replacement,
                );
            }
            SyntaxKind::PlaceholderNode => {
                (&child).write(stream);
            }
            _ => {
                unreachable!(
                    "unexpected child in literal string: {:?}",
                    child.element().kind()
                );
            }
        }
    }
}

/// Formats a [`LiteralNone`](wdl_ast::v1::LiteralNone).
pub fn format_literal_none(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children().expect("literal none children");
    let none = children.next().expect("literal none token");
    assert!(none.element().kind() == SyntaxKind::NoneKeyword);
    (&none).write(stream);
    assert!(children.next().is_none());
}

/// Formats a [`LiteralPair`](wdl_ast::v1::LiteralPair).
pub fn format_literal_pair(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children().expect("literal pair children");

    let open_paren = children.next().expect("literal pair open paren");
    assert!(open_paren.element().kind() == SyntaxKind::OpenParen);
    (&open_paren).write(stream);

    let left = children.next().expect("literal pair left");
    (&left).write(stream);

    let comma = children.next().expect("literal pair comma");
    assert!(comma.element().kind() == SyntaxKind::Comma);
    (&comma).write(stream);
    stream.end_word();

    let right = children.next().expect("literal pair right");
    (&right).write(stream);

    let close_paren = children.next().expect("literal pair close paren");
    assert!(close_paren.element().kind() == SyntaxKind::CloseParen);
    (&close_paren).write(stream);
    assert!(children.next().is_none());
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

/// Formats a [`NameRef`](wdl_ast::v1::NameRef).
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

    let empty = items.is_empty();
    if !empty {
        stream.increment_indent();
    }
    let mut commas = commas.iter();
    for item in items {
        (&item).write(stream);
        if let Some(comma) = commas.next() {
            (comma).write(stream);
        } else {
            stream.push_literal(",".to_string(), SyntaxKind::Comma);
        }
        stream.end_line();
    }

    if !empty {
        stream.decrement_indent();
    }
    (&close_bracket.expect("literal array close bracket")).write(stream);
}

/// Formats a [`LiteralMapItem`](wdl_ast::v1::LiteralMapItem).
pub fn format_literal_map_item(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children().expect("literal map item children");

    let key = children.next().expect("literal map item key");
    (&key).write(stream);

    let colon = children.next().expect("literal map item colon");
    assert!(colon.element().kind() == SyntaxKind::Colon);
    (&colon).write(stream);
    stream.end_word();

    let value = children.next().expect("literal map item value");
    (&value).write(stream);
    assert!(children.next().is_none());
}

/// Formats a [`LiteralMap`](wdl_ast::v1::LiteralMap).
pub fn format_literal_map(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children().expect("literal map children");

    let open_brace = children.next().expect("literal map open brace");
    assert!(open_brace.element().kind() == SyntaxKind::OpenBrace);
    (&open_brace).write(stream);
    stream.increment_indent();

    let mut close_brace = None;
    let mut commas = Vec::new();
    let items = children
        .filter(|child| {
            if child.element().kind() == SyntaxKind::CloseBrace {
                close_brace = Some(child.to_owned());
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
        } else {
            stream.push_literal(",".to_string(), SyntaxKind::Comma);
        }
        stream.end_line();
    }

    stream.decrement_indent();
    (&close_brace.expect("literal map close brace")).write(stream);
}

/// Formats a [`LiteralObjectItem`](wdl_ast::v1::LiteralObjectItem).
pub fn format_literal_object_item(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children().expect("literal object item children");

    let key = children.next().expect("literal object item key");
    assert!(key.element().kind() == SyntaxKind::Ident);
    (&key).write(stream);

    let colon = children.next().expect("literal object item colon");
    assert!(colon.element().kind() == SyntaxKind::Colon);
    (&colon).write(stream);
    stream.end_word();

    let value = children.next().expect("literal object item value");
    (&value).write(stream);
    assert!(children.next().is_none());
}

/// Formats a [`LiteralObject`](wdl_ast::v1::LiteralObject).
pub fn format_literal_object(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children().expect("literal object children");

    let open_brace = children.next().expect("literal object open brace");
    assert!(open_brace.element().kind() == SyntaxKind::OpenBrace);
    (&open_brace).write(stream);
    stream.increment_indent();

    let mut close_brace = None;
    let mut commas = Vec::new();
    let members = children
        .filter(|child| {
            if child.element().kind() == SyntaxKind::CloseBrace {
                close_brace = Some(child.to_owned());
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
    for member in members {
        (&member).write(stream);
        if let Some(comma) = commas.next() {
            (comma).write(stream);
        } else {
            stream.push_literal(",".to_string(), SyntaxKind::Comma);
        }
        stream.end_line();
    }

    stream.decrement_indent();
    (&close_brace.expect("literal object close brace")).write(stream);
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
        let should_end_word = child.element().kind() == SyntaxKind::Plus;
        if should_end_word {
            stream.end_word();
        }
        (&child).write(stream);
        if should_end_word {
            stream.end_word();
        }
    }
}

/// Formats a [`SubtractionExpr`](wdl_ast::v1::SubtractionExpr).
pub fn format_subtraction_expr(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    for child in element.children().expect("subtraction expr children") {
        let should_end_word = child.element().kind() == SyntaxKind::Minus;
        if should_end_word {
            stream.end_word();
        }
        (&child).write(stream);
        if should_end_word {
            stream.end_word();
        }
    }
}

/// Formats a [`MultiplicationExpr`](wdl_ast::v1::MultiplicationExpr).
pub fn format_multiplication_expr(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    for child in element.children().expect("multiplication expr children") {
        let should_end_word = child.element().kind() == SyntaxKind::Asterisk;
        if should_end_word {
            stream.end_word();
        }
        (&child).write(stream);
        if should_end_word {
            stream.end_word();
        }
    }
}

/// Formats a [`DivisionExpr`](wdl_ast::v1::DivisionExpr).
pub fn format_division_expr(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    for child in element.children().expect("division expr children") {
        let should_end_word = child.element().kind() == SyntaxKind::Slash;
        if should_end_word {
            stream.end_word();
        }
        (&child).write(stream);
        if should_end_word {
            stream.end_word();
        }
    }
}

/// Formats a [`ModuloExpr`](wdl_ast::v1::ModuloExpr).
pub fn format_modulo_expr(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    for child in element.children().expect("modulo expr children") {
        let should_end_word = child.element().kind() == SyntaxKind::Percent;
        if should_end_word {
            stream.end_word();
        }
        (&child).write(stream);
        if should_end_word {
            stream.end_word();
        }
    }
}

/// Formats an [`ExponentiationExpr`](wdl_ast::v1::ExponentiationExpr).
pub fn format_exponentiation_expr(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    for child in element.children().expect("exponentiation expr children") {
        let should_end_word = child.element().kind() == SyntaxKind::Exponentiation;
        if should_end_word {
            stream.end_word();
        }
        (&child).write(stream);
        if should_end_word {
            stream.end_word();
        }
    }
}

/// Formats a [`LogicalAndExpr`](wdl_ast::v1::LogicalAndExpr).
pub fn format_logical_and_expr(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    for child in element.children().expect("logical and expr children") {
        let should_end_word = child.element().kind() == SyntaxKind::LogicalAnd;
        if should_end_word {
            stream.end_word();
        }
        (&child).write(stream);
        if should_end_word {
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

    let nested_else_if = match stream.last_literal_kind() {
        Some(SyntaxKind::ElseKeyword) => true,
        _ => false,
    };

    let if_keyword = children.next().expect("if keyword");
    assert!(if_keyword.element().kind() == SyntaxKind::IfKeyword);
    (&if_keyword).write(stream);
    stream.end_word();

    for child in children {
        let kind = child.element().kind();
        if kind == SyntaxKind::ThenKeyword {
            if !nested_else_if {
                stream.increment_indent();
            } else {
                stream.end_word();
            }
        } else if kind == SyntaxKind::ElseKeyword {
            stream.end_line();
        }
        (&child).write(stream);
        if matches!(kind, SyntaxKind::ElseKeyword | SyntaxKind::ThenKeyword) {
            stream.end_word();
        }
    }
    if !nested_else_if {
        stream.decrement_indent();
    }
}
