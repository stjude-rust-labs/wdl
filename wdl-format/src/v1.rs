//! A module for formatting WDL v1 elements.

use std::fmt::Write;

use wdl_ast::v1::Decl;
use wdl_ast::v1::DefaultOption;
use wdl_ast::v1::DocumentItem;
use wdl_ast::v1::Expr;
use wdl_ast::v1::HintsItem;
use wdl_ast::v1::HintsSection;
use wdl_ast::v1::InputSection;
use wdl_ast::v1::LiteralBoolean;
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
            PlaceholderOption::Default(default) => default.format(writer, state),
            PlaceholderOption::Sep(sep) => sep.format(writer, state),
            PlaceholderOption::TrueFalse(true_false) => true_false.format(writer, state),
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
        let mut iter = self.as_str().chars().peekable();
        let mut prev_c = None;
        while let Some(c) = iter.next() {
            match c {
                '\\' => {
                    if let Some(next_c) = iter.peek() {
                        if *next_c == '\'' {
                            // Do not write this backslash
                            prev_c = Some(c);
                            continue;
                        }
                    }
                    writer.write_char(c)?;
                }
                '"' => {
                    if let Some(pc) = prev_c {
                        if pc != '\\' {
                            writer.write_char('\\')?;
                        }
                    }
                    writer.write_char(c)?;
                }
                _ => {
                    writer.write_char(c)?;
                }
            }
            prev_c = Some(c);
        }

        Ok(())
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

impl Formattable for Expr {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, _state: &mut State) -> std::fmt::Result {
        write!(writer, "{}", self.syntax())
    }
}

impl Formattable for Decl {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, state: &mut State) -> std::fmt::Result {
        format_preceding_comments(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            state,
            false,
        )?;

        let ty = self.ty();
        state.indent(writer)?;
        ty.format(writer, state)?;
        format_inline_comment(
            &SyntaxElement::from(ty.syntax().clone()),
            writer,
            state,
            true,
        )?;

        let name = self.name();
        format_preceding_comments(
            &SyntaxElement::from(name.syntax().clone()),
            writer,
            state,
            true,
        )?;
        state.space_or_indent(writer)?;
        name.format(writer, state)?;
        format_inline_comment(
            &SyntaxElement::from(name.syntax().clone()),
            writer,
            state,
            true,
        )?;

        if let Some(expr) = self.expr() {
            let equal_sign = first_child_of_kind(self.syntax(), SyntaxKind::Assignment);
            format_preceding_comments(&equal_sign, writer, state, true)?;
            state.space_or_indent(writer)?;
            write!(writer, "{}", equal_sign)?;
            format_inline_comment(&equal_sign, writer, state, true)?;

            format_preceding_comments(
                &SyntaxElement::from(expr.syntax().clone()),
                writer,
                state,
                true,
            )?;
            state.space_or_indent(writer)?;
            expr.format(writer, state)?;
        }
        format_inline_comment(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            state,
            false,
        )
    }
}

impl Formattable for InputSection {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, state: &mut State) -> std::fmt::Result {
        format_preceding_comments(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            state,
            false,
        )?;

        let input_keyword = first_child_of_kind(self.syntax(), SyntaxKind::InputKeyword);
        state.indent(writer)?;
        write!(writer, "{}", input_keyword)?;
        format_inline_comment(&input_keyword, writer, state, true)?;

        let open_brace = first_child_of_kind(self.syntax(), SyntaxKind::OpenBrace);
        format_preceding_comments(&open_brace, writer, state, true)?;
        // Open braces should ignore the "+1 rule" followed by other interrupted
        // elements.
        if state.interrupted() {
            state.reset_interrupted();
            state.indent(writer)?;
        } else {
            write!(writer, "{}", SPACE)?;
        }
        write!(writer, "{}", open_brace)?;
        format_inline_comment(&open_brace, writer, state, false)?;

        state.increment_indent();

        for decl in self.declarations() {
            decl.format(writer, state)?;
        }

        state.decrement_indent();

        let close_brace = first_child_of_kind(self.syntax(), SyntaxKind::CloseBrace);
        format_preceding_comments(&close_brace, writer, state, false)?;
        state.indent(writer)?;
        write!(writer, "{}", close_brace)?;
        format_inline_comment(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            state,
            false,
        )
    }
}

impl Formattable for OutputSection {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, state: &mut State) -> std::fmt::Result {
        format_preceding_comments(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            state,
            false,
        )?;

        let output_keyword = first_child_of_kind(self.syntax(), SyntaxKind::OutputKeyword);
        state.indent(writer)?;
        write!(writer, "{}", output_keyword)?;
        format_inline_comment(&output_keyword, writer, state, true)?;

        let open_brace = first_child_of_kind(self.syntax(), SyntaxKind::OpenBrace);
        format_preceding_comments(&open_brace, writer, state, true)?;
        // Open braces should ignore the "+1 rule" followed by other interrupted
        // elements.
        if state.interrupted() {
            state.reset_interrupted();
            state.indent(writer)?;
        } else {
            write!(writer, "{}", SPACE)?;
        }
        write!(writer, "{}", open_brace)?;
        format_inline_comment(&open_brace, writer, state, false)?;

        state.increment_indent();

        for decl in self.declarations() {
            Decl::Bound(decl).format(writer, state)?;
        }

        state.decrement_indent();

        let close_brace = first_child_of_kind(self.syntax(), SyntaxKind::CloseBrace);
        format_preceding_comments(&close_brace, writer, state, false)?;
        state.indent(writer)?;
        write!(writer, "{}", close_brace)?;
        format_inline_comment(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            state,
            false,
        )
    }
}

impl Formattable for HintsItem {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, state: &mut State) -> std::fmt::Result {
        format_preceding_comments(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            state,
            false,
        )?;

        let name = self.name();
        state.indent(writer)?;
        name.format(writer, state)?;
        format_inline_comment(
            &SyntaxElement::from(name.syntax().clone()),
            writer,
            state,
            true,
        )?;

        let colon = first_child_of_kind(self.syntax(), SyntaxKind::Colon);
        format_preceding_comments(&colon, writer, state, true)?;
        if state.interrupted() {
            state.indent(writer)?;
        }
        write!(writer, "{}", colon)?;
        format_inline_comment(&colon, writer, state, true)?;

        let expr = self.expr();
        format_preceding_comments(
            &SyntaxElement::from(expr.syntax().clone()),
            writer,
            state,
            true,
        )?;
        state.space_or_indent(writer)?;
        expr.format(writer, state)?;
        format_inline_comment(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            state,
            false,
        )
    }
}

impl Formattable for HintsSection {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, state: &mut State) -> std::fmt::Result {
        format_preceding_comments(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            state,
            false,
        )?;

        let hints_keyword = first_child_of_kind(self.syntax(), SyntaxKind::HintsKeyword);
        state.indent(writer)?;
        write!(writer, "{}", hints_keyword)?;
        format_inline_comment(&hints_keyword, writer, state, true)?;

        let open_brace = first_child_of_kind(self.syntax(), SyntaxKind::OpenBrace);
        format_preceding_comments(&open_brace, writer, state, true)?;
        // Open braces should ignore the "+1 rule" followed by other interrupted
        // elements.
        if state.interrupted() {
            state.reset_interrupted();
            state.indent(writer)?;
        } else {
            write!(writer, "{}", SPACE)?;
        }
        write!(writer, "{}", open_brace)?;
        format_inline_comment(&open_brace, writer, state, false)?;

        state.increment_indent();

        for item in self.items() {
            item.format(writer, state)?;
        }

        state.decrement_indent();

        let close_brace = first_child_of_kind(self.syntax(), SyntaxKind::CloseBrace);
        format_preceding_comments(&close_brace, writer, state, false)?;
        state.indent(writer)?;
        write!(writer, "{}", close_brace)?;
        format_inline_comment(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            state,
            false,
        )
    }
}

impl Formattable for StructDefinition {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, state: &mut State) -> std::fmt::Result {
        format_preceding_comments(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            state,
            false,
        )?;

        let struct_keyword = first_child_of_kind(self.syntax(), SyntaxKind::StructKeyword);
        write!(writer, "{}", struct_keyword)?;
        format_inline_comment(&struct_keyword, writer, state, true)?;

        let name = self.name();
        format_preceding_comments(
            &SyntaxElement::from(name.syntax().clone()),
            writer,
            state,
            true,
        )?;
        state.space_or_indent(writer)?;
        name.format(writer, state)?;
        format_inline_comment(
            &SyntaxElement::from(name.syntax().clone()),
            writer,
            state,
            true,
        )?;

        let open_brace = first_child_of_kind(self.syntax(), SyntaxKind::OpenBrace);
        // Open braces should ignore the "+1 rule" followed by other interrupted
        // elements.
        if state.interrupted() {
            state.reset_interrupted();
            state.indent(writer)?;
        } else {
            write!(writer, "{}", SPACE)?;
        }
        write!(writer, "{}", open_brace)?;
        format_inline_comment(&open_brace, writer, state, false)?;

        state.increment_indent();

        if let Some(m) = self.metadata().next() {
            m.format(writer, state)?;
            write!(writer, "{}", NEWLINE)?;
        }

        if let Some(pm) = self.parameter_metadata().next() {
            pm.format(writer, state)?;
            write!(writer, "{}", NEWLINE)?;
        }

        for decl in self.members() {
            Decl::Unbound(decl).format(writer, state)?;
        }

        state.decrement_indent();

        let close_brace = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::CloseBrace)
            .expect("Struct Definition should have a close brace");
        format_preceding_comments(&close_brace, writer, state, false)?;
        state.indent(writer)?;
        write!(writer, "{}", close_brace)?;
        format_inline_comment(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            state,
            false,
        )
    }
}

impl Formattable for DocumentItem {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, state: &mut State) -> std::fmt::Result {
        match self {
            DocumentItem::Import(_) => {
                unreachable!("Import statements should not be formatted as a DocumentItem")
            }
            DocumentItem::Workflow(workflow) => workflow.format(writer, state),
            DocumentItem::Task(task) => task.format(writer, state),
            DocumentItem::Struct(structure) => structure.format(writer, state),
        }
    }
}
