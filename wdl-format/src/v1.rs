//! A module for formatting WDL v1 elements.

use wdl_ast::v1::Decl;
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
use wdl_ast::v1::StringPart;
use wdl_ast::v1::StructDefinition;
use wdl_ast::v1::Type;
use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::SyntaxElement;
use wdl_ast::SyntaxKind;

use super::comments::format_inline_comment;
use super::comments::format_preceding_comments;
use super::state::SPACE;
use super::Formattable;
use super::State;
use super::NEWLINE;
use super::STRING_TERMINATOR;

impl Formattable for LiteralString {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, _state: &mut State) -> std::fmt::Result {
        write!(writer, "{}", STRING_TERMINATOR)?;
        for part in self.parts() {
            match part {
                StringPart::Text(text) => {
                    write!(writer, "{}", text.as_str())?;
                }
                StringPart::Placeholder(placeholder) => {
                    write!(writer, "{}", placeholder.syntax())?;
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
            let equal_sign = self
                .syntax()
                .children_with_tokens()
                .find(|element| element.kind() == SyntaxKind::Assignment)
                .expect("Bound declaration should have an equals sign");
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

        let input_keyword = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::InputKeyword)
            .expect("Input Section should have an input keyword");
        state.indent(writer)?;
        write!(writer, "{}", input_keyword)?;
        format_inline_comment(&input_keyword, writer, state, true)?;

        let open_brace = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::OpenBrace)
            .expect("Input Section should have an open brace");
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

        let close_brace = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::CloseBrace)
            .expect("Input Section should have a close brace");
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

        let output_keyword = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::OutputKeyword)
            .expect("Output Section should have an output keyword");
        state.indent(writer)?;
        write!(writer, "{}", output_keyword)?;
        format_inline_comment(&output_keyword, writer, state, true)?;

        let open_brace = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::OpenBrace)
            .expect("Output Section should have an open brace");
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

        let close_brace = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::CloseBrace)
            .expect("Output Section should have a close brace");
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

        let colon = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::Colon)
            .expect("Hints Item should have a colon");
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

        let hints_keyword = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::HintsKeyword)
            .expect("Hints Section should have a hints keyword");
        state.indent(writer)?;
        write!(writer, "{}", hints_keyword)?;
        format_inline_comment(&hints_keyword, writer, state, true)?;

        let open_brace = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::OpenBrace)
            .expect("Hints Section should have an open brace");
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

        let close_brace = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::CloseBrace)
            .expect("Hints Section should have a close brace");
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

        let struct_keyword = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::StructKeyword)
            .expect("Struct Definition should have a struct keyword");
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

        let open_brace = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::OpenBrace)
            .expect("Struct Definition should have an open brace");
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
