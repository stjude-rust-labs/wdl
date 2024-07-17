//! A module for formatting elements in tasks.

use wdl_ast::v1::CommandPart;
use wdl_ast::v1::CommandSection;
use wdl_ast::v1::CommandText;
use wdl_ast::v1::Decl;
use wdl_ast::v1::RequirementsItem;
use wdl_ast::v1::RequirementsSection;
use wdl_ast::v1::RuntimeItem;
use wdl_ast::v1::RuntimeSection;
use wdl_ast::v1::TaskDefinition;
use wdl_ast::v1::TaskItem;
use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::SyntaxElement;
use wdl_ast::SyntaxKind;

use super::formatter::SPACE;
use super::Formattable;
use super::Formatter;
use super::NEWLINE;

impl Formattable for CommandText {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        _formatter: &mut Formatter,
    ) -> std::fmt::Result {
        write!(writer, "{}", self.as_str())
    }
}

impl Formattable for CommandSection {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        formatter: &mut Formatter,
    ) -> std::fmt::Result {
        // format_preceding_comments(
        //     &SyntaxElement::from(self.syntax().clone()),
        //     writer,
        //     formatter,
        //     false,
        // )?;

        // let command_keyword = first_child_of_kind(self.syntax(),
        // SyntaxKind::CommandKeyword); formatter.indent(writer)?;
        // write!(writer, "{}", command_keyword)?;
        // format_inline_comment(&command_keyword, writer, formatter, true)?;

        // // coerce all command sections to use heredoc ('<<<>>>>') syntax
        // // (as opposed to bracket ('{}') syntax)
        // let open_section = if self.is_heredoc() {
        //     first_child_of_kind(self.syntax(), SyntaxKind::OpenHeredoc)
        // } else {
        //     first_child_of_kind(self.syntax(), SyntaxKind::OpenBrace)
        // };
        // format_preceding_comments(&open_section, writer, formatter, true)?;

        // // Open braces should ignore the "+1 rule" followed by other interrupted
        // // elements.
        // if formatter.interrupted() {
        //     formatter.reset_interrupted();
        //     formatter.indent(writer)?;
        // } else {
        //     write!(writer, "{}", SPACE)?;
        // }
        // write!(writer, "<<<")?;

        // for part in self.parts() {
        //     match part {
        //         CommandPart::Text(t) => {
        //             t.format(writer, formatter)?;
        //         }
        //         CommandPart::Placeholder(p) => {
        //             p.format(writer, formatter)?;
        //         }
        //     }
        // }

        // write!(writer, ">>>")?;
        // format_inline_comment(
        //     &SyntaxElement::from(self.syntax().clone()),
        //     writer,
        //     formatter,
        //     false,
        // )
        Ok(())
    }
}

impl Formattable for RuntimeItem {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        formatter: &mut Formatter,
    ) -> std::fmt::Result {
        // format_preceding_comments(
        //     &SyntaxElement::from(self.syntax().clone()),
        //     writer,
        //     formatter,
        //     false,
        // )?;

        // let name = self.name();
        // formatter.indent(writer)?;
        // name.format(writer, formatter)?;
        // format_inline_comment(
        //     &SyntaxElement::from(name.syntax().clone()),
        //     writer,
        //     formatter,
        //     true,
        // )?;

        // let colon = first_child_of_kind(self.syntax(), SyntaxKind::Colon);
        // format_preceding_comments(&colon, writer, formatter, true)?;
        // if formatter.interrupted() {
        //     formatter.reset_interrupted();
        //     formatter.indent(writer)?;
        // }
        // write!(writer, "{}", colon)?;
        // format_inline_comment(&colon, writer, formatter, true)?;

        // let expr = self.expr();
        // format_preceding_comments(
        //     &SyntaxElement::from(expr.syntax().clone()),
        //     writer,
        //     formatter,
        //     true,
        // )?;
        // formatter.space_or_indent(writer)?;
        // expr.format(writer, formatter)?;
        // format_inline_comment(
        //     &SyntaxElement::from(self.syntax().clone()),
        //     writer,
        //     formatter,
        //     false,
        // )
        Ok(())
    }
}

impl Formattable for RuntimeSection {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        formatter: &mut Formatter,
    ) -> std::fmt::Result {
        // format_preceding_comments(
        //     &SyntaxElement::from(self.syntax().clone()),
        //     writer,
        //     formatter,
        //     false,
        // )?;

        // let runtime_keyword = first_child_of_kind(self.syntax(),
        // SyntaxKind::RuntimeKeyword); formatter.indent(writer)?;
        // write!(writer, "{}", runtime_keyword)?;
        // format_inline_comment(&runtime_keyword, writer, formatter, true)?;

        // let open_brace = first_child_of_kind(self.syntax(), SyntaxKind::OpenBrace);
        // format_preceding_comments(&open_brace, writer, formatter, true)?;
        // // Open braces should ignore the "+1 rule" followed by other interrupted
        // // elements.
        // if formatter.interrupted() {
        //     formatter.reset_interrupted();
        //     formatter.indent(writer)?;
        // } else {
        //     write!(writer, "{}", SPACE)?;
        // }
        // write!(writer, "{}", open_brace)?;
        // format_inline_comment(&open_brace, writer, formatter, false)?;

        // formatter.increment_indent();

        // for item in self.items() {
        //     item.format(writer, formatter)?;
        // }

        // formatter.decrement_indent();

        // let close_brace = first_child_of_kind(self.syntax(), SyntaxKind::CloseBrace);
        // format_preceding_comments(&close_brace, writer, formatter, true)?;
        // formatter.indent(writer)?;
        // write!(writer, "{}", close_brace)?;
        // format_inline_comment(
        //     &SyntaxElement::from(self.syntax().clone()),
        //     writer,
        //     formatter,
        //     false,
        // )
        Ok(())
    }
}

impl Formattable for RequirementsItem {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        formatter: &mut Formatter,
    ) -> std::fmt::Result {
        // format_preceding_comments(
        //     &SyntaxElement::from(self.syntax().clone()),
        //     writer,
        //     formatter,
        //     false,
        // )?;

        // let name = self.name();
        // formatter.indent(writer)?;
        // name.format(writer, formatter)?;
        // format_inline_comment(
        //     &SyntaxElement::from(name.syntax().clone()),
        //     writer,
        //     formatter,
        //     true,
        // )?;

        // let colon = first_child_of_kind(self.syntax(), SyntaxKind::Colon);
        // format_preceding_comments(&colon, writer, formatter, true)?;
        // if formatter.interrupted() {
        //     formatter.reset_interrupted();
        //     formatter.indent(writer)?;
        // }
        // write!(writer, "{}", colon)?;
        // format_inline_comment(&colon, writer, formatter, true)?;

        // let expr = self.expr();
        // format_preceding_comments(
        //     &SyntaxElement::from(expr.syntax().clone()),
        //     writer,
        //     formatter,
        //     true,
        // )?;
        // formatter.space_or_indent(writer)?;
        // expr.format(writer, formatter)?;
        // format_inline_comment(
        //     &SyntaxElement::from(self.syntax().clone()),
        //     writer,
        //     formatter,
        //     false,
        // )
        Ok(())
    }
}

impl Formattable for RequirementsSection {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        formatter: &mut Formatter,
    ) -> std::fmt::Result {
        // format_preceding_comments(
        //     &SyntaxElement::from(self.syntax().clone()),
        //     writer,
        //     formatter,
        //     false,
        // )?;

        // let requirements_keyword =
        //     first_child_of_kind(self.syntax(), SyntaxKind::RequirementsKeyword);
        // formatter.indent(writer)?;
        // write!(writer, "{}", requirements_keyword)?;
        // format_inline_comment(&requirements_keyword, writer, formatter, true)?;

        // let open_brace = first_child_of_kind(self.syntax(), SyntaxKind::OpenBrace);
        // format_preceding_comments(&open_brace, writer, formatter, true)?;
        // // Open braces should ignore the "+1 rule" followed by other interrupted
        // // elements.
        // if formatter.interrupted() {
        //     formatter.reset_interrupted();
        //     formatter.indent(writer)?;
        // } else {
        //     write!(writer, "{}", SPACE)?;
        // }
        // write!(writer, "{}", open_brace)?;
        // format_inline_comment(&open_brace, writer, formatter, false)?;

        // formatter.increment_indent();

        // for item in self.items() {
        //     item.format(writer, formatter)?;
        // }

        // formatter.decrement_indent();

        // let close_brace = first_child_of_kind(self.syntax(), SyntaxKind::CloseBrace);
        // format_preceding_comments(&close_brace, writer, formatter, true)?;
        // formatter.indent(writer)?;
        // write!(writer, "{}", close_brace)?;
        // format_inline_comment(
        //     &SyntaxElement::from(self.syntax().clone()),
        //     writer,
        //     formatter,
        //     false,
        // )
        Ok(())
    }
}

impl Formattable for TaskDefinition {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        formatter: &mut Formatter,
    ) -> std::fmt::Result {
        // format_preceding_comments(
        //     &SyntaxElement::from(self.syntax().clone()),
        //     writer,
        //     formatter,
        //     false,
        // )?;

        // let task_keyword = first_child_of_kind(self.syntax(),
        // SyntaxKind::TaskKeyword); formatter.indent(writer)?;
        // write!(writer, "{}", task_keyword)?;
        // format_inline_comment(&task_keyword, writer, formatter, true)?;

        // let name = self.name();
        // format_preceding_comments(
        //     &SyntaxElement::from(name.syntax().clone()),
        //     writer,
        //     formatter,
        //     true,
        // )?;
        // formatter.space_or_indent(writer)?;
        // name.format(writer, formatter)?;
        // format_inline_comment(
        //     &SyntaxElement::from(name.syntax().clone()),
        //     writer,
        //     formatter,
        //     true,
        // )?;

        // let open_brace = first_child_of_kind(self.syntax(), SyntaxKind::OpenBrace);
        // format_preceding_comments(&open_brace, writer, formatter, true)?;
        // // Open braces should ignore the "+1 rule" followed by other interrupted
        // // elements.
        // if formatter.interrupted() {
        //     formatter.reset_interrupted();
        //     formatter.indent(writer)?;
        // } else {
        //     write!(writer, "{}", SPACE)?;
        // }
        // write!(writer, "{}", open_brace)?;
        // format_inline_comment(&open_brace, writer, formatter, false)?;

        // formatter.increment_indent();

        // let mut meta_section_str = String::new();
        // let mut parameter_meta_section_str = String::new();
        // let mut input_section_str = String::new();
        // let mut declaration_section_str = String::new();
        // let mut command_section_str = String::new();
        // let mut output_section_str = String::new();
        // let mut runtime_section_str = String::new();
        // let mut hints_section_str = String::new();
        // let mut requirements_section_str = String::new();

        // for item in self.items() {
        //     match item {
        //         TaskItem::Metadata(m) => {
        //             m.format(&mut meta_section_str, formatter)?;
        //         }
        //         TaskItem::ParameterMetadata(pm) => {
        //             pm.format(&mut parameter_meta_section_str, formatter)?;
        //         }
        //         TaskItem::Input(i) => {
        //             i.format(&mut input_section_str, formatter)?;
        //         }
        //         TaskItem::Declaration(d) => {
        //             Decl::Bound(d).format(&mut declaration_section_str, formatter)?;
        //         }
        //         TaskItem::Command(c) => {
        //             c.format(&mut command_section_str, formatter)?;
        //         }
        //         TaskItem::Output(o) => {
        //             o.format(&mut output_section_str, formatter)?;
        //         }
        //         TaskItem::Runtime(r) => {
        //             r.format(&mut runtime_section_str, formatter)?;
        //         }
        //         TaskItem::Hints(h) => {
        //             h.format(&mut hints_section_str, formatter)?;
        //         }
        //         TaskItem::Requirements(r) => {
        //             r.format(&mut requirements_section_str, formatter)?;
        //         }
        //     }
        // }

        // let mut first_section = true;

        // if !meta_section_str.is_empty() {
        //     first_section = false;
        //     write!(writer, "{}", meta_section_str)?;
        // }
        // if !parameter_meta_section_str.is_empty() {
        //     if !first_section {
        //         write!(writer, "{}", NEWLINE)?;
        //     }
        //     first_section = false;
        //     write!(writer, "{}", parameter_meta_section_str)?;
        // }
        // if !input_section_str.is_empty() {
        //     if !first_section {
        //         write!(writer, "{}", NEWLINE)?;
        //     }
        //     first_section = false;
        //     write!(writer, "{}", input_section_str)?;
        // }
        // if !declaration_section_str.is_empty() {
        //     if !first_section {
        //         write!(writer, "{}", NEWLINE)?;
        //     }
        //     first_section = false;
        //     write!(writer, "{}", declaration_section_str)?;
        // }
        // // Command section is required
        // if !first_section {
        //     write!(writer, "{}", NEWLINE)?;
        // }
        // write!(writer, "{}", command_section_str)?;
        // if !output_section_str.is_empty() {
        //     write!(writer, "{}", NEWLINE)?;
        //     write!(writer, "{}", output_section_str)?;
        // }
        // if !runtime_section_str.is_empty() {
        //     write!(writer, "{}", NEWLINE)?;
        //     write!(writer, "{}", runtime_section_str)?;
        // }
        // if !hints_section_str.is_empty() {
        //     write!(writer, "{}", NEWLINE)?;
        //     write!(writer, "{}", hints_section_str)?;
        // }
        // if !requirements_section_str.is_empty() {
        //     write!(writer, "{}", NEWLINE)?;
        //     write!(writer, "{}", requirements_section_str)?;
        // }

        // formatter.decrement_indent();

        // let close_brace = first_child_of_kind(self.syntax(), SyntaxKind::CloseBrace);
        // format_preceding_comments(&close_brace, writer, formatter, true)?;
        // formatter.indent(writer)?;
        // write!(writer, "{}", close_brace)?;
        // format_inline_comment(
        //     &SyntaxElement::from(self.syntax().clone()),
        //     writer,
        //     formatter,
        //     false,
        // )
        Ok(())
    }
}
