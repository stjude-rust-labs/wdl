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
use super::format_element_with_comments;
use super::formatter::SPACE;
use super::Formattable;
use super::Formatter;
use super::LinePosition;
use super::NEWLINE;

impl Formattable for LiteralNull {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        _state: &mut Formatter,
    ) -> std::fmt::Result {
        write!(writer, "{}", self.syntax())
    }
}

impl Formattable for MetadataObject {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        formatter: &mut Formatter,
    ) -> std::fmt::Result {
        format_preceding_comments(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            formatter,
            false,
        )?;

        let open_brace = first_child_of_kind(self.syntax(), SyntaxKind::OpenBrace);
        format_element_with_comments(
            &open_brace,
            writer,
            formatter,
            LinePosition::End,
            |writer, formatter| {
                if formatter.interrupted() {
                    formatter.reset_interrupted();
                    formatter.indent(writer)?;
                }
                Ok(())
            },
        )?;

        formatter.increment_indent();

        let mut commas = self
            .syntax()
            .children_with_tokens()
            .filter(|c| c.kind() == SyntaxKind::Comma);

        for item in self.items() {
            item.format(writer, formatter)?;
            if let Some(cur_comma) = commas.next() {
                format_element_with_comments(
                    &cur_comma,
                    writer,
                    formatter,
                    LinePosition::End,
                    |_, _| Ok(()),
                )?;
            } else {
                // No trailing comma was in the input
                write!(writer, ",")?;
                write!(writer, "{}", NEWLINE)?;
            }
        }

        formatter.decrement_indent();

        let close_brace = first_child_of_kind(self.syntax(), SyntaxKind::CloseBrace);
        format_preceding_comments(&close_brace, writer, formatter, false)?;
        formatter.indent(writer)?;
        write!(writer, "{}", close_brace)?;
        format_inline_comment(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            formatter,
            true,
        )
    }
}

impl Formattable for MetadataArray {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        formatter: &mut Formatter,
    ) -> std::fmt::Result {
        format_preceding_comments(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            formatter,
            false,
        )?;

        let open_bracket = first_child_of_kind(self.syntax(), SyntaxKind::OpenBracket);
        format_element_with_comments(
            &open_bracket,
            writer,
            formatter,
            LinePosition::End,
            |writer, formatter| {
                if formatter.interrupted() {
                    formatter.reset_interrupted();
                    formatter.indent(writer)?;
                }
                Ok(())
            },
        )?;

        formatter.increment_indent();

        let mut commas = self
            .syntax()
            .children_with_tokens()
            .filter(|c| c.kind() == SyntaxKind::Comma);

        for item in self.elements() {
            formatter.indent(writer)?;
            item.format(writer, formatter)?;
            if let Some(cur_comma) = commas.next() {
                format_element_with_comments(
                    &cur_comma,
                    writer,
                    formatter,
                    LinePosition::End,
                    |_, _| Ok(()),
                )?;
            } else {
                // No trailing comma was in the input
                write!(writer, ",")?;
                write!(writer, "{}", NEWLINE)?;
            }
        }

        formatter.decrement_indent();

        let close_bracket = first_child_of_kind(self.syntax(), SyntaxKind::CloseBracket);
        format_preceding_comments(&close_bracket, writer, formatter, false)?;
        formatter.indent(writer)?;
        write!(writer, "{}", close_bracket)?;
        format_inline_comment(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            formatter,
            true,
        )
    }
}

impl Formattable for MetadataValue {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        formatter: &mut Formatter,
    ) -> std::fmt::Result {
        match self {
            MetadataValue::String(s) => s.format(writer, formatter),
            MetadataValue::Boolean(b) => b.format(writer, formatter),
            MetadataValue::Float(f) => f.format(writer, formatter),
            MetadataValue::Integer(i) => i.format(writer, formatter),
            MetadataValue::Null(n) => n.format(writer, formatter),
            MetadataValue::Object(o) => o.format(writer, formatter),
            MetadataValue::Array(a) => a.format(writer, formatter),
        }
    }
}

impl Formattable for MetadataObjectItem {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        formatter: &mut Formatter,
    ) -> std::fmt::Result {
        format_preceding_comments(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            formatter,
            false,
        )?;

        let name = self.name();
        formatter.indent(writer)?;
        name.format(writer, formatter)?;
        format_inline_comment(
            &SyntaxElement::from(name.syntax().clone()),
            writer,
            formatter,
            true,
        )?;

        let colon = first_child_of_kind(self.syntax(), SyntaxKind::Colon);
        format_element_with_comments(
            &colon,
            writer,
            formatter,
            LinePosition::Middle,
            |writer, formatter| {
                if formatter.interrupted() {
                    formatter.indent(writer)?;
                    formatter.reset_interrupted();
                }
                Ok(())
            },
        )?;

        let value = self.value();
        format_preceding_comments(
            &SyntaxElement::from(value.syntax().clone()),
            writer,
            formatter,
            true,
        )?;
        formatter.space_or_indent(writer)?;
        value.format(writer, formatter)?;
        format_inline_comment(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            formatter,
            true,
        )
    }
}

impl Formattable for MetadataSection {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        formatter: &mut Formatter,
    ) -> std::fmt::Result {
        format_preceding_comments(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            formatter,
            false,
        )?;

        let meta_keyword = first_child_of_kind(self.syntax(), SyntaxKind::MetaKeyword);
        formatter.indent(writer)?;
        write!(writer, "{}", meta_keyword)?;
        format_inline_comment(&meta_keyword, writer, formatter, true)?;

        let open_brace = first_child_of_kind(self.syntax(), SyntaxKind::OpenBrace);
        format_element_with_comments(
            &open_brace,
            writer,
            formatter,
            LinePosition::End,
            |writer, formatter| {
                if formatter.interrupted() {
                    formatter.reset_interrupted();
                    formatter.indent(writer)?;
                } else {
                    write!(writer, "{}", SPACE)?;
                }
                Ok(())
            },
        )?;

        formatter.increment_indent();

        for item in self.items() {
            item.format(writer, formatter)?;
            if formatter.interrupted() {
                formatter.reset_interrupted();
            } else {
                write!(writer, "{}", NEWLINE)?;
            }
        }

        formatter.decrement_indent();

        let close_brace = first_child_of_kind(self.syntax(), SyntaxKind::CloseBrace);
        format_preceding_comments(&close_brace, writer, formatter, false)?;
        formatter.indent(writer)?;
        write!(writer, "{}", close_brace)?;
        format_inline_comment(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            formatter,
            false,
        )
    }
}

impl Formattable for ParameterMetadataSection {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        formatter: &mut Formatter,
    ) -> std::fmt::Result {
        format_preceding_comments(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            formatter,
            false,
        )?;

        let parameter_meta_keyword =
            first_child_of_kind(self.syntax(), SyntaxKind::ParameterMetaKeyword);
        formatter.indent(writer)?;
        write!(writer, "{}", parameter_meta_keyword)?;
        format_inline_comment(&parameter_meta_keyword, writer, formatter, true)?;

        let open_brace = first_child_of_kind(self.syntax(), SyntaxKind::OpenBrace);
        format_element_with_comments(
            &open_brace,
            writer,
            formatter,
            LinePosition::End,
            |writer, formatter| {
                if formatter.interrupted() {
                    formatter.reset_interrupted();
                    formatter.indent(writer)?;
                } else {
                    write!(writer, "{}", SPACE)?;
                }
                Ok(())
            },
        )?;

        formatter.increment_indent();

        for item in self.items() {
            item.format(writer, formatter)?;
            if formatter.interrupted() {
                formatter.reset_interrupted();
            } else {
                write!(writer, "{}", NEWLINE)?;
            }
        }

        formatter.decrement_indent();

        let close_brace = first_child_of_kind(self.syntax(), SyntaxKind::CloseBrace);
        format_preceding_comments(&close_brace, writer, formatter, false)?;
        formatter.indent(writer)?;
        write!(writer, "{}", close_brace)?;
        format_inline_comment(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            formatter,
            false,
        )
    }
}
