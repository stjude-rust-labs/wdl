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
use wdl_ast::v1::StructKeyword;
use wdl_ast::v1::TrueFalseOption;
use wdl_ast::v1::Type;
use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::SyntaxElement;
use wdl_ast::SyntaxKind;
use wdl_grammar::SyntaxExt;

use super::formatter::SPACE;
use super::Formattable;
use super::Formatter;
use super::NEWLINE;
use super::STRING_TERMINATOR;

impl Formattable for DefaultOption {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        formatter: &mut Formatter,
    ) -> std::fmt::Result {
        // let default_word = first_child_of_kind(self.syntax(), SyntaxKind::Ident);
        // format_preceding_comments(&default_word, writer, formatter, true)?;
        // write!(writer, "{}", default_word)?;
        // format_inline_comment(&default_word, writer, formatter, true)?;

        // let assignment = first_child_of_kind(self.syntax(), SyntaxKind::Assignment);
        // format_preceding_comments(&assignment, writer, formatter, true)?;
        // formatter.space_or_indent(writer)?;
        // write!(writer, "{}", assignment)?;
        // format_inline_comment(&assignment, writer, formatter, true)?;

        // let value = self.value();
        // format_preceding_comments(
        //     &SyntaxElement::from(value.syntax().clone()),
        //     writer,
        //     formatter,
        //     true,
        // )?;
        // formatter.space_or_indent(writer)?;
        // value.format(writer, formatter)?;
        // format_inline_comment(
        //     &SyntaxElement::from(value.syntax().clone()),
        //     writer,
        //     formatter,
        //     true,
        // )
        Ok(())
    }
}

impl Formattable for SepOption {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        formatter: &mut Formatter,
    ) -> std::fmt::Result {
        //     let sep_word = first_child_of_kind(self.syntax(), SyntaxKind::Ident);
        //     format_preceding_comments(&sep_word, writer, formatter, true)?;
        //     write!(writer, "{}", sep_word)?;
        //     format_inline_comment(&sep_word, writer, formatter, true)?;

        //     let assignment = first_child_of_kind(self.syntax(),
        // SyntaxKind::Assignment);     format_preceding_comments(&assignment,
        // writer, formatter, true)?;     formatter.space_or_indent(writer)?;
        //     write!(writer, "{}", assignment)?;
        //     format_inline_comment(&assignment, writer, formatter, true)?;

        //     let separator = self.separator();
        //     format_preceding_comments(
        //         &SyntaxElement::from(separator.syntax().clone()),
        //         writer,
        //         formatter,
        //         true,
        //     )?;
        //     formatter.space_or_indent(writer)?;
        //     separator.format(writer, formatter)?;
        //     format_inline_comment(
        //         &SyntaxElement::from(separator.syntax().clone()),
        //         writer,
        //         formatter,
        //         true,
        //     )
        Ok(())
    }
}

impl Formattable for TrueFalseOption {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        formatter: &mut Formatter,
    ) -> std::fmt::Result {
        // let mut true_clause = String::new();
        // let mut false_clause = String::new();
        // let mut which_clause = None;
        // for child in self.syntax().children_with_tokens() {
        //     match child.kind() {
        //         SyntaxKind::TrueKeyword => {
        //             which_clause = Some(true);

        //             format_preceding_comments(&child, &mut true_clause, formatter,
        // true)?;             write!(true_clause, "{}", child)?;
        //             format_inline_comment(&child, &mut true_clause, formatter,
        // true)?;         }
        //         SyntaxKind::FalseKeyword => {
        //             which_clause = Some(false);

        //             format_preceding_comments(&child, &mut false_clause, formatter,
        // true)?;             write!(false_clause, "{}", child)?;
        //             format_inline_comment(&child, &mut false_clause, formatter,
        // true)?;         }
        //         SyntaxKind::Assignment => {
        //             let cur_clause = match which_clause {
        //                 Some(true) => &mut true_clause,
        //                 Some(false) => &mut false_clause,
        //                 _ => unreachable!(
        //                     "should have found a true or false keyword before an
        // assignment"                 ),
        //             };

        //             format_preceding_comments(&child, cur_clause, formatter, true)?;
        //             formatter.space_or_indent(cur_clause)?;
        //             write!(cur_clause, "{}", child)?;
        //             format_inline_comment(&child, cur_clause, formatter, true)?;
        //         }
        //         SyntaxKind::LiteralStringNode => {
        //             let cur_clause = match which_clause {
        //                 Some(true) => &mut true_clause,
        //                 Some(false) => &mut false_clause,
        //                 _ => unreachable!(
        //                     "should have found a true or false keyword before a
        // string"                 ),
        //             };

        //             format_preceding_comments(&child, cur_clause, formatter, true)?;
        //             formatter.space_or_indent(cur_clause)?;
        //             let literal_string = LiteralString::cast(
        //                 child
        //                     .as_node()
        //                     .expect("LiteralStringNode should be a node")
        //                     .clone(),
        //             )
        //             .expect("LiteralStringNode should cast to a LiteralString");
        //             literal_string.format(cur_clause, formatter)?;
        //             format_inline_comment(&child, writer, formatter, true)?;
        //         }
        //         SyntaxKind::Whitespace => {
        //             // Ignore
        //         }
        //         SyntaxKind::Comment => {
        //             // Handled by a call to `format_preceding_comments`
        //             // or `format_inline_comment` in another match arm.
        //         }
        //         _ => {
        //             unreachable!("Unexpected syntax kind: {:?}", child.kind());
        //         }
        //     }
        // }
        // write!(writer, "{} {}", true_clause, false_clause)?;

        Ok(())
    }
}

impl Formattable for PlaceholderOption {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        formatter: &mut Formatter,
    ) -> std::fmt::Result {
        match self {
            PlaceholderOption::Default(default) => default.format(writer, formatter),
            PlaceholderOption::Sep(sep) => sep.format(writer, formatter),
            PlaceholderOption::TrueFalse(true_false) => true_false.format(writer, formatter),
        }
    }
}

impl Formattable for Placeholder {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        formatter: &mut Formatter,
    ) -> std::fmt::Result {
        // coerce all placeholders into '~{}' placeholders
        // (as opposed to '${}' placeholders)
        write!(writer, "~{{")?;

        let mut option_present = false;
        if let Some(option) = self.options().next() {
            option.format(writer, formatter)?;
            option_present = true;
        }

        let expr = self.expr();
        if option_present {
            formatter.space_or_indent(writer)?;
        }
        expr.format(writer, formatter)?;

        write!(writer, "}}")
    }
}

impl Formattable for StringText {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        _state: &mut Formatter,
    ) -> std::fmt::Result {
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
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        formatter: &mut Formatter,
    ) -> std::fmt::Result {
        write!(writer, "{}", STRING_TERMINATOR)?;
        for part in self.parts() {
            match part {
                StringPart::Text(text) => {
                    text.format(writer, formatter)?;
                }
                StringPart::Placeholder(placeholder) => {
                    placeholder.format(writer, formatter)?;
                }
            }
        }
        write!(writer, "{}", STRING_TERMINATOR)
    }
}

impl Formattable for LiteralBoolean {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        _state: &mut Formatter,
    ) -> std::fmt::Result {
        write!(writer, "{}", self.value()) // TODO
    }
}

impl Formattable for LiteralFloat {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        _state: &mut Formatter,
    ) -> std::fmt::Result {
        write!(writer, "{}", self.syntax()) // TODO
    }
}

impl Formattable for LiteralInteger {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        _state: &mut Formatter,
    ) -> std::fmt::Result {
        write!(writer, "{}", self.syntax()) // TODO
    }
}

impl Formattable for Type {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        _state: &mut Formatter,
    ) -> std::fmt::Result {
        write!(writer, "{}", self.syntax()) // TODO
    }
}

impl Formattable for Expr {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        _state: &mut Formatter,
    ) -> std::fmt::Result {
        write!(writer, "{}", self.syntax()) // TODO
    }
}

impl Formattable for Decl {
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

        // let ty = self.ty();
        // formatter.indent(writer)?;
        // ty.format(writer, formatter)?;
        // format_inline_comment(
        //     &SyntaxElement::from(ty.syntax().clone()),
        //     writer,
        //     formatter,
        //     true,
        // )?;

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

        // if let Some(expr) = self.expr() {
        //     let assignment = first_child_of_kind(self.syntax(),
        // SyntaxKind::Assignment);     format_preceding_comments(&assignment,
        // writer, formatter, true)?;     formatter.space_or_indent(writer)?;
        //     write!(writer, "{}", assignment)?;
        //     format_inline_comment(&assignment, writer, formatter, true)?;

        //     format_preceding_comments(
        //         &SyntaxElement::from(expr.syntax().clone()),
        //         writer,
        //         formatter,
        //         true,
        //     )?;
        //     formatter.space_or_indent(writer)?;
        //     expr.format(writer, formatter)?;
        // }
        // format_inline_comment(
        //     &SyntaxElement::from(self.syntax().clone()),
        //     writer,
        //     formatter,
        //     false,
        // )
        Ok(())
    }
}

impl Formattable for InputSection {
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

        // let input_keyword = first_child_of_kind(self.syntax(),
        // SyntaxKind::InputKeyword); formatter.indent(writer)?;
        // write!(writer, "{}", input_keyword)?;
        // format_inline_comment(&input_keyword, writer, formatter, true)?;

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

        // for decl in self.declarations() {
        //     decl.format(writer, formatter)?;
        // }

        // formatter.decrement_indent();

        // let close_brace = first_child_of_kind(self.syntax(), SyntaxKind::CloseBrace);
        // format_preceding_comments(&close_brace, writer, formatter, false)?;
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

impl Formattable for OutputSection {
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

        // let output_keyword = first_child_of_kind(self.syntax(),
        // SyntaxKind::OutputKeyword); formatter.indent(writer)?;
        // write!(writer, "{}", output_keyword)?;
        // format_inline_comment(&output_keyword, writer, formatter, true)?;

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

        // for decl in self.declarations() {
        //     Decl::Bound(decl).format(writer, formatter)?;
        // }

        // formatter.decrement_indent();

        // let close_brace = first_child_of_kind(self.syntax(), SyntaxKind::CloseBrace);
        // format_preceding_comments(&close_brace, writer, formatter, false)?;
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

impl Formattable for HintsItem {
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

impl Formattable for HintsSection {
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

        // let hints_keyword = first_child_of_kind(self.syntax(),
        // SyntaxKind::HintsKeyword); formatter.indent(writer)?;
        // write!(writer, "{}", hints_keyword)?;
        // format_inline_comment(&hints_keyword, writer, formatter, true)?;

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
        // format_preceding_comments(&close_brace, writer, formatter, false)?;
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

impl Formattable for StructKeyword {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        _formatter: &mut Formatter,
    ) -> std::fmt::Result {
        write!(writer, "{}", self.as_str())
    }
}

impl Formattable for StructDefinition {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        formatter: &mut Formatter,
    ) -> std::fmt::Result {
        formatter.format_preceding_trivia(writer, self.syntax().preceding_trivia(), false, true)?;

        let struct_keyword = self.keyword();
        struct_keyword.format(writer, formatter)?;
        formatter.format_inline_comment(writer, struct_keyword.syntax().inline_comment(), true)?;

        let name = self.name();
        formatter.format_preceding_trivia(writer, self.syntax().preceding_trivia(), true, false)?;
        formatter.space_or_indent(writer)?;
        name.format(writer, formatter)?;
        formatter.format_inline_comment(writer, name.syntax().inline_comment(), true)?;
        // formatter.space_or_indent(writer)?;
        // name.format(writer, formatter)?;
        // format_inline_comment(
        //     &SyntaxElement::from(name.syntax().clone()),
        //     writer,
        //     formatter,
        //     true,
        // )?;

        // let open_brace = first_child_of_kind(self.syntax(), SyntaxKind::OpenBrace);
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

        // if let Some(m) = self.metadata().next() {
        //     m.format(writer, formatter)?;
        //     write!(writer, "{}", NEWLINE)?;
        // }

        // if let Some(pm) = self.parameter_metadata().next() {
        //     pm.format(writer, formatter)?;
        //     write!(writer, "{}", NEWLINE)?;
        // }

        // for decl in self.members() {
        //     Decl::Unbound(decl).format(writer, formatter)?;
        // }

        // formatter.decrement_indent();

        // let close_brace = self
        //     .syntax()
        //     .children_with_tokens()
        //     .find(|element| element.kind() == SyntaxKind::CloseBrace)
        //     .expect("StructDefinition should have a close brace");
        // format_preceding_comments(&close_brace, writer, formatter, false)?;
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

impl Formattable for DocumentItem {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        formatter: &mut Formatter,
    ) -> std::fmt::Result {
        match self {
            DocumentItem::Import(_) => {
                unreachable!("Import statements should not be formatted as a DocumentItem")
            }
            DocumentItem::Workflow(workflow) => workflow.format(writer, formatter),
            DocumentItem::Task(task) => task.format(writer, formatter),
            DocumentItem::Struct(structure) => structure.format(writer, formatter),
        }
    }
}
