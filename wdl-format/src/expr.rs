//! A module for formatting WDL Expr elements.

use std::fmt::Write;

use wdl_ast::v1::Decl;
use wdl_ast::v1::DefaultOption;
use wdl_ast::v1::DocumentItem;
use wdl_ast::v1::Expr;
use wdl_ast::v1::HintsItem;
use wdl_ast::v1::HintsSection;
use wdl_ast::v1::InputSection;
use wdl_ast::v1::LiteralBoolean;
use wdl_ast::v1::LiteralExpr;
use wdl_ast::v1::LiteralFloat;
use wdl_ast::v1::LiteralInteger;
use wdl_ast::v1::LiteralString;
use wdl_ast::v1::OutputSection;
use wdl_ast::v1::Placeholder;
use wdl_ast::v1::PlaceholderOption;
use wdl_ast::v1::SepOption;
use wdl_ast::v1::StringPart;
use wdl_ast::v1::StringText;
use wdl_ast::v1::StructDefinition;
use wdl_ast::v1::TrueFalseOption;
use wdl_ast::v1::Type;
use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::Ident;
use wdl_ast::SyntaxElement;
use wdl_ast::SyntaxKind;

use super::comments::format_inline_comment;
use super::comments::format_preceding_comments;
use super::first_child_of_kind;
use super::state::SPACE;
use super::Formattable;
use super::State;
use super::NEWLINE;
use super::STRING_TERMINATOR;

impl Formattable for DefaultOption {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, state: &mut State) -> std::fmt::Result {
        let default_word = first_child_of_kind(self.syntax(), SyntaxKind::Ident);
        format_preceding_comments(&default_word, writer, state, true)?;
        write!(writer, "{}", default_word)?;
        format_inline_comment(&default_word, writer, state, true)?;

        let assignment = first_child_of_kind(self.syntax(), SyntaxKind::Assignment);
        format_preceding_comments(&assignment, writer, state, true)?;
        state.space_or_indent(writer)?;
        write!(writer, "{}", assignment)?;
        format_inline_comment(&assignment, writer, state, true)?;

        let value = self.value();
        format_preceding_comments(
            &SyntaxElement::from(value.syntax().clone()),
            writer,
            state,
            true,
        )?;
        state.space_or_indent(writer)?;
        value.format(writer, state)?;
        format_inline_comment(
            &SyntaxElement::from(value.syntax().clone()),
            writer,
            state,
            true,
        )
    }
}

impl Formattable for SepOption {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, state: &mut State) -> std::fmt::Result {
        let sep_word = first_child_of_kind(self.syntax(), SyntaxKind::Ident);
        format_preceding_comments(&sep_word, writer, state, true)?;
        write!(writer, "{}", sep_word)?;
        format_inline_comment(&sep_word, writer, state, true)?;

        let assignment = first_child_of_kind(self.syntax(), SyntaxKind::Assignment);
        format_preceding_comments(&assignment, writer, state, true)?;
        state.space_or_indent(writer)?;
        write!(writer, "{}", assignment)?;
        format_inline_comment(&assignment, writer, state, true)?;

        let separator = self.separator();
        format_preceding_comments(
            &SyntaxElement::from(separator.syntax().clone()),
            writer,
            state,
            true,
        )?;
        state.space_or_indent(writer)?;
        separator.format(writer, state)?;
        format_inline_comment(
            &SyntaxElement::from(separator.syntax().clone()),
            writer,
            state,
            true,
        )
    }
}

impl Formattable for TrueFalseOption {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, state: &mut State) -> std::fmt::Result {
        let mut true_clause = String::new();
        let mut false_clause = String::new();
        let mut which_clause = None;
        for child in self.syntax().children_with_tokens() {
            match child.kind() {
                SyntaxKind::TrueKeyword => {
                    which_clause = Some(true);

                    format_preceding_comments(&child, &mut true_clause, state, true)?;
                    write!(true_clause, "{}", child)?;
                    format_inline_comment(&child, &mut true_clause, state, true)?;
                }
                SyntaxKind::FalseKeyword => {
                    which_clause = Some(false);

                    format_preceding_comments(&child, &mut false_clause, state, true)?;
                    write!(false_clause, "{}", child)?;
                    format_inline_comment(&child, &mut false_clause, state, true)?;
                }
                SyntaxKind::Assignment => {
                    let cur_clause = match which_clause {
                        Some(true) => &mut true_clause,
                        Some(false) => &mut false_clause,
                        _ => unreachable!(
                            "should have found a true or false keyword before an assignment"
                        ),
                    };

                    format_preceding_comments(&child, cur_clause, state, true)?;
                    state.space_or_indent(cur_clause)?;
                    write!(cur_clause, "{}", child)?;
                    format_inline_comment(&child, cur_clause, state, true)?;
                }
                SyntaxKind::LiteralStringNode => {
                    let cur_clause = match which_clause {
                        Some(true) => &mut true_clause,
                        Some(false) => &mut false_clause,
                        _ => unreachable!(
                            "should have found a true or false keyword before a string"
                        ),
                    };

                    format_preceding_comments(&child, cur_clause, state, true)?;
                    state.space_or_indent(cur_clause)?;
                    let literal_string = LiteralString::cast(
                        child
                            .as_node()
                            .expect("LiteralStringNode should be a node")
                            .clone(),
                    )
                    .expect("LiteralStringNode should cast to a LiteralString");
                    literal_string.format(cur_clause, state)?;
                    format_inline_comment(&child, writer, state, true)?;
                }
                SyntaxKind::Whitespace => {
                    // Ignore
                }
                SyntaxKind::Comment => {
                    // Handled by a call to `format_preceding_comments`
                    // or `format_inline_comment` in another match arm.
                }
                _ => {
                    unreachable!("Unexpected syntax kind: {:?}", child.kind());
                }
            }
        }
        write!(writer, "{} {}", true_clause, false_clause)?;

        Ok(())
    }
}

impl Formattable for PlaceholderOption {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, state: &mut State) -> std::fmt::Result {
        match self {
            PlaceholderOption::Default(default) => {
                default.format(writer, state)
            }
            PlaceholderOption::Sep(sep) => {
                sep.format(writer, state)
            }
            PlaceholderOption::TrueFalse(true_false) => {
                true_false.format(writer, state)
            }
        }
    }
}

impl Formattable for Placeholder {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, state: &mut State) -> std::fmt::Result {
        write!(writer, "~{{")?;

        let mut option_present = false;
        if let Some(option) = self.options().next() {
            option.format(writer, state)?;
            option_present = true;
        }

        let expr = self.expr();
        if option_present {
            state.space_or_indent(writer)?;
        }
        expr.format(writer, state)?;

        write!(writer, "}}")
    }
}

impl Formattable for StringText {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, _state: &mut State) -> std::fmt::Result {
        write!(writer, "{}", self.as_str())
    }
}

impl Formattable for LiteralString {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, state: &mut State) -> std::fmt::Result {
        write!(writer, "{}", STRING_TERMINATOR)?;
        for part in self.parts() {
            match part {
                StringPart::Text(text) => {
                    text.format(writer, state)?;
                }
                StringPart::Placeholder(placeholder) => {
                    placeholder.format(writer, state)?;
                }
            }
        }
        write!(writer, "{}", STRING_TERMINATOR)
    }
}

impl Formattable for LiteralBoolean {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, _state: &mut State) -> std::fmt::Result {
        write!(writer, "{}", self.value())
    }
}

impl Formattable for LiteralFloat {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, _state: &mut State) -> std::fmt::Result {
        write!(writer, "{}", self.syntax())
    }
}

impl Formattable for LiteralInteger {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, _state: &mut State) -> std::fmt::Result {
        write!(writer, "{}", self.syntax())
    }
}

impl Formattable for Type {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, _state: &mut State) -> std::fmt::Result {
        write!(writer, "{}", self.syntax())
    }
}

impl Formattable for LiteralExpr {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, state: &mut State) -> std::fmt::Result {
        match self {
            LiteralExpr::Boolean(bool) => {
                bool.format(writer, state)
            }
            LiteralExpr::Float(f) => {
                f.format(writer, state)
            }
            LiteralExpr::Integer(i) => {
                i.format(writer, state)
            }
            LiteralExpr::String(str) => {
                str.format(writer, state)
            }
            _ => {
                write!(writer, "{}", self.syntax())
            }
            // LiteralExpr::Array(literal) => {
            //     literal.format(writer, state)
            // }
            // LiteralExpr::Map(literal) => {
            //     literal.format(writer, state)
            // }
            // LiteralExpr::Object(literal) => {
            //     literal.format(writer, state)
            // }
            // LiteralExpr::Hints(literal) => {
            //     literal.format(writer, state)
            // }
            // LiteralExpr::Struct(literal) => {
            //     literal.format(writer, state)
            // }
            // LiteralExpr::Pair(literal) => {
            //     literal.format(writer, state)
            // }
            // LiteralExpr::None(literal) => {
            //     literal.format(writer, state)
            // }
            // LiteralExpr::Input(literal) => {
            //     literal.format(writer, state)
            // }
            // LiteralExpr::Output(literal) => {
            //     literal.format(writer, state)
            // }
        }
    }
}

impl Formattable for Expr {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, _state: &mut State) -> std::fmt::Result {
        match self {
            Expr::Literal(literal) => {
                literal.format(writer, _state)
            }
            _ => {
                write!(writer, "{}", self.syntax())
            }
        }
    }
}
