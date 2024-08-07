//! A module for formatting metadata sections (meta and parameter_meta).

use wdl_ast::v1::LiteralNull;
use wdl_ast::v1::MetadataArray;
use wdl_ast::v1::MetadataObject;
use wdl_ast::v1::MetadataObjectItem;
use wdl_ast::v1::MetadataSection;
use wdl_ast::v1::MetadataValue;
use wdl_ast::v1::ParameterMetadataSection;
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

impl Formattable for LiteralNull {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, _state: &mut State) -> std::fmt::Result {
        write!(writer, "{}", self.syntax())
    }
}

impl Formattable for MetadataObject {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, state: &mut State) -> std::fmt::Result {
        format_preceding_comments(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            state,
            false,
        )?;

        let open_brace = first_child_of_kind(self.syntax(), SyntaxKind::OpenBrace);
        format_preceding_comments(&open_brace, writer, state, true)?;
        // Open braces should ignore the "+1 rule" followed by other interrupted
        // elements.
        if state.interrupted() {
            state.reset_interrupted();
            state.indent(writer)?;
        }
        write!(writer, "{}", open_brace)?;
        format_inline_comment(&open_brace, writer, state, false)?;

        state.increment_indent();

        let mut commas = self
            .syntax()
            .children_with_tokens()
            .filter(|c| c.kind() == SyntaxKind::Comma);

        for item in self.items() {
            item.format(writer, state)?;
            if let Some(cur_comma) = commas.next() {
                format_preceding_comments(&cur_comma, writer, state, true)?;
                write!(writer, ",")?;
                format_inline_comment(&cur_comma, writer, state, false)?;
            } else {
                // No trailing comma was in the input
                write!(writer, ",")?;
                write!(writer, "{}", NEWLINE)?;
            }
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
            true,
        )
    }
}

impl Formattable for MetadataArray {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, state: &mut State) -> std::fmt::Result {
        format_preceding_comments(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            state,
            false,
        )?;

        let open_bracket = first_child_of_kind(self.syntax(), SyntaxKind::OpenBracket);
        format_preceding_comments(&open_bracket, writer, state, true)?;
        // Open braces should ignore the "+1 rule" followed by other interrupted
        // elements.
        if state.interrupted() {
            state.reset_interrupted();
            state.indent(writer)?;
        }
        write!(writer, "{}", open_bracket)?;
        format_inline_comment(&open_bracket, writer, state, false)?;

        state.increment_indent();

        let mut commas = self
            .syntax()
            .children_with_tokens()
            .filter(|c| c.kind() == SyntaxKind::Comma);

        for item in self.elements() {
            state.indent(writer)?;
            item.format(writer, state)?;
            if let Some(cur_comma) = commas.next() {
                format_preceding_comments(&cur_comma, writer, state, true)?;
                write!(writer, ",")?;
                format_inline_comment(&cur_comma, writer, state, false)?;
            } else {
                // No trailing comma was in the input
                write!(writer, ",")?;
                write!(writer, "{}", NEWLINE)?;
            }
        }

        state.decrement_indent();

        let close_bracket = first_child_of_kind(self.syntax(), SyntaxKind::CloseBracket);
        format_preceding_comments(&close_bracket, writer, state, false)?;
        state.indent(writer)?;
        write!(writer, "{}", close_bracket)?;
        format_inline_comment(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            state,
            true,
        )
    }
}

impl Formattable for MetadataValue {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, state: &mut State) -> std::fmt::Result {
        match self {
            MetadataValue::String(s) => s.format(writer, state),
            MetadataValue::Boolean(b) => b.format(writer, state),
            MetadataValue::Float(f) => f.format(writer, state),
            MetadataValue::Integer(i) => i.format(writer, state),
            MetadataValue::Null(n) => n.format(writer, state),
            MetadataValue::Object(o) => o.format(writer, state),
            MetadataValue::Array(a) => a.format(writer, state),
        }
    }
}

impl Formattable for MetadataObjectItem {
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
            state.reset_interrupted();
        }
        write!(writer, "{}", colon)?;
        format_inline_comment(&colon, writer, state, true)?;

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
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            state,
            true,
        )
    }
}

impl Formattable for MetadataSection {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, state: &mut State) -> std::fmt::Result {
        format_preceding_comments(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            state,
            false,
        )?;

        let meta_keyword = first_child_of_kind(self.syntax(), SyntaxKind::MetaKeyword);
        state.indent(writer)?;
        write!(writer, "{}", meta_keyword)?;
        format_inline_comment(&meta_keyword, writer, state, true)?;

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
            write!(writer, "{}", NEWLINE)?;
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

impl Formattable for ParameterMetadataSection {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, state: &mut State) -> std::fmt::Result {
        format_preceding_comments(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            state,
            false,
        )?;

        let parameter_meta_keyword =
            first_child_of_kind(self.syntax(), SyntaxKind::ParameterMetaKeyword);
        state.indent(writer)?;
        write!(writer, "{}", parameter_meta_keyword)?;
        format_inline_comment(&parameter_meta_keyword, writer, state, true)?;

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
            write!(writer, "{}", NEWLINE)?;
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
