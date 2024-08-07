//! A module for formatting WDL v1 elements.

use wdl_ast::v1::Decl;
use wdl_ast::v1::DocumentItem;
use wdl_ast::v1::HintsItem;
use wdl_ast::v1::HintsSection;
use wdl_ast::v1::InputSection;
use wdl_ast::v1::OutputSection;
use wdl_ast::v1::StructDefinition;
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
            let assignment = first_child_of_kind(self.syntax(), SyntaxKind::Assignment);
            format_preceding_comments(&assignment, writer, state, true)?;
            state.space_or_indent(writer)?;
            write!(writer, "{}", assignment)?;
            format_inline_comment(&assignment, writer, state, true)?;

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
