//! A module for formatting elements in workflows.

use wdl_ast::v1::CallAfter;
use wdl_ast::v1::CallAlias;
use wdl_ast::v1::CallInputItem;
use wdl_ast::v1::CallStatement;
use wdl_ast::v1::ConditionalStatement;
use wdl_ast::v1::Decl;
use wdl_ast::v1::ScatterStatement;
use wdl_ast::v1::WorkflowDefinition;
use wdl_ast::v1::WorkflowItem;
use wdl_ast::v1::WorkflowStatement;
use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::SyntaxElement;
use wdl_ast::SyntaxKind;

use super::formatter::SPACE;
use super::Formattable;
use super::Formatter;
use super::NEWLINE;

impl Formattable for CallAlias {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        formatter: &mut Formatter,
    ) -> std::fmt::Result {
        // format_preceding_comments(
        //     &SyntaxElement::from(self.syntax().clone()),
        //     writer,
        //     formatter,
        //     true,
        // )?;

        // let as_keyword = first_child_of_kind(self.syntax(), SyntaxKind::AsKeyword);
        // formatter.space_or_indent(writer)?;
        // write!(writer, "{}", as_keyword)?;
        // format_inline_comment(&as_keyword, writer, formatter, true)?;

        // let ident = self.name();
        // format_preceding_comments(
        //     &SyntaxElement::from(ident.syntax().clone()),
        //     writer,
        //     formatter,
        //     true,
        // )?;
        // formatter.space_or_indent(writer)?;
        // ident.format(writer, formatter)?;
        // format_inline_comment(
        //     &SyntaxElement::from(self.syntax().clone()),
        //     writer,
        //     formatter,
        //     true,
        // )
        Ok(())
    }
}

impl Formattable for CallAfter {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        formatter: &mut Formatter,
    ) -> std::fmt::Result {
        // format_preceding_comments(
        //     &SyntaxElement::from(self.syntax().clone()),
        //     writer,
        //     formatter,
        //     true,
        // )?;

        // let after_keyword = first_child_of_kind(self.syntax(),
        // SyntaxKind::AfterKeyword); formatter.space_or_indent(writer)?;
        // write!(writer, "{}", after_keyword)?;
        // format_inline_comment(&after_keyword, writer, formatter, true)?;

        // let ident = self.name();
        // format_preceding_comments(
        //     &SyntaxElement::from(ident.syntax().clone()),
        //     writer,
        //     formatter,
        //     true,
        // )?;
        // formatter.space_or_indent(writer)?;
        // ident.format(writer, formatter)?;
        // format_inline_comment(
        //     &SyntaxElement::from(self.syntax().clone()),
        //     writer,
        //     formatter,
        //     true,
        // )
        Ok(())
    }
}

impl Formattable for CallInputItem {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        formatter: &mut Formatter,
    ) -> std::fmt::Result {
        // let name = self.name();
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
        //     true,
        // )
        Ok(())
    }
}

impl Formattable for CallStatement {
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

        // let call_keyword = first_child_of_kind(self.syntax(),
        // SyntaxKind::CallKeyword); formatter.indent(writer)?;
        // write!(writer, "{}", call_keyword)?;
        // format_inline_comment(&call_keyword, writer, formatter, true)?;

        // let target = self.target();
        // format_preceding_comments(
        //     &SyntaxElement::Node(target.syntax().clone()),
        //     writer,
        //     formatter,
        //     true,
        // )?;
        // formatter.space_or_indent(writer)?;
        // write!(writer, "{}", target.syntax())?;
        // format_inline_comment(
        //     &SyntaxElement::Node(target.syntax().clone()),
        //     writer,
        //     formatter,
        //     true,
        // )?;

        // if let Some(alias) = self.alias() {
        //     alias.format(writer, formatter)?;
        // }

        // for after in self.after() {
        //     after.format(writer, formatter)?;
        // }

        // let inputs: Vec<_> = self.inputs().collect();
        // if !inputs.is_empty() {
        //     let open_brace = first_child_of_kind(self.syntax(),
        // SyntaxKind::OpenBrace);     format_preceding_comments(&open_brace,
        // writer, formatter, true)?;     // Open braces should ignore the "+1
        // rule" followed by other interrupted     // elements.
        //     if formatter.interrupted() {
        //         formatter.reset_interrupted();
        //         formatter.indent(writer)?;
        //     } else {
        //         write!(writer, "{}", SPACE)?;
        //     }
        //     write!(writer, "{}", open_brace)?;
        //     format_inline_comment(&open_brace, writer, formatter, true)?;

        //     // TODO consider detecting if document is >= v1.2 and forcing the
        // optional input     // syntax
        //     if let Some(input_keyword) = self
        //         .syntax()
        //         .children_with_tokens()
        //         .find(|c| c.kind() == SyntaxKind::InputKeyword)
        //     {
        //         format_preceding_comments(&input_keyword, writer, formatter, true)?;
        //         formatter.space_or_indent(writer)?;
        //         write!(writer, "{}", input_keyword)?;
        //         format_inline_comment(&input_keyword, writer, formatter, true)?;

        //         let colon = first_child_of_kind(self.syntax(), SyntaxKind::Colon);
        //         format_preceding_comments(&colon, writer, formatter, true)?;
        //         if formatter.interrupted() {
        //             formatter.indent(writer)?;
        //         }
        //         write!(writer, "{}", colon)?;
        //         format_inline_comment(&colon, writer, formatter, true)?;
        //     } // else v1.2 syntax

        //     if inputs.len() == 1 {
        //         let input = inputs.first().expect("inputs should have a first
        // element");         format_preceding_comments(
        //             &SyntaxElement::from(input.syntax().clone()),
        //             writer,
        //             formatter,
        //             true,
        //         )?;
        //         formatter.space_or_indent(writer)?;
        //         input.format(writer, formatter)?;
        //         // TODO there may be a trailing comma with comments attached to it

        //         let close_brace = first_child_of_kind(self.syntax(),
        // SyntaxKind::CloseBrace);         format_preceding_comments(&
        // close_brace, writer, formatter, true)?;         formatter.
        // space_or_indent(writer)?;         write!(writer, "{}", close_brace)?;
        //     } else {
        //         // multiple inputs
        //         let mut commas = self
        //             .syntax()
        //             .children_with_tokens()
        //             .filter(|c| c.kind() == SyntaxKind::Comma);

        //         formatter.increment_indent();

        //         for input in inputs {
        //             if !formatter.interrupted() {
        //                 write!(writer, "{}", NEWLINE)?;
        //             } else {
        //                 formatter.reset_interrupted();
        //             }
        //             format_preceding_comments(
        //                 &SyntaxElement::from(input.syntax().clone()),
        //                 writer,
        //                 formatter,
        //                 false,
        //             )?;
        //             formatter.indent(writer)?;
        //             input.format(writer, formatter)?;
        //             if let Some(cur_comma) = commas.next() {
        //                 format_preceding_comments(&cur_comma, writer, formatter,
        // true)?;                 write!(writer, ",")?;
        //                 format_inline_comment(&cur_comma, writer, formatter, true)?;
        //             } else {
        //                 write!(writer, ",")?;
        //             }
        //         }
        //         if !formatter.interrupted() {
        //             write!(writer, "{}", NEWLINE)?;
        //         }

        //         formatter.decrement_indent();

        //         let close_brace = first_child_of_kind(self.syntax(),
        // SyntaxKind::CloseBrace);         format_preceding_comments(&
        // close_brace, writer, formatter, false)?;         formatter.
        // indent(writer)?;         write!(writer, "{}", close_brace)?;
        //     }
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

impl Formattable for ConditionalStatement {
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

        // let if_keyword = first_child_of_kind(self.syntax(), SyntaxKind::IfKeyword);
        // formatter.indent(writer)?;
        // write!(writer, "{}", if_keyword)?;
        // format_inline_comment(&if_keyword, writer, formatter, true)?;

        // let open_paren = first_child_of_kind(self.syntax(), SyntaxKind::OpenParen);
        // format_preceding_comments(&open_paren, writer, formatter, true)?;
        // // Open parens should ignore the "+1 rule" followed by other interrupted
        // // elements.
        // if formatter.interrupted() {
        //     formatter.reset_interrupted();
        //     formatter.indent(writer)?;
        // } else {
        //     write!(writer, "{}", SPACE)?;
        // }
        // write!(writer, "{}", open_paren)?;

        // let mut paren_on_same_line = true;
        // let expr = self.expr();
        // // PERF: This calls `to_string()` which is also called later by `format()`
        // // There should be a way to avoid this.
        // let multiline_expr = expr.syntax().to_string().contains(NEWLINE);

        // format_inline_comment(&open_paren, writer, formatter, !multiline_expr)?;
        // if multiline_expr {
        //     formatter.increment_indent();
        //     paren_on_same_line = false;
        // }
        // format_preceding_comments(
        //     &SyntaxElement::from(expr.syntax().clone()),
        //     writer,
        //     formatter,
        //     !multiline_expr,
        // )?;
        // if formatter.interrupted() || multiline_expr {
        //     formatter.indent(writer)?;
        //     paren_on_same_line = false;
        // }
        // expr.format(writer, formatter)?;
        // format_inline_comment(
        //     &SyntaxElement::from(expr.syntax().clone()),
        //     writer,
        //     formatter,
        //     !multiline_expr,
        // )?;
        // if formatter.interrupted() {
        //     paren_on_same_line = false;
        // }

        // let close_paren = first_child_of_kind(self.syntax(), SyntaxKind::CloseParen);
        // format_preceding_comments(&close_paren, writer, formatter, !multiline_expr)?;
        // if formatter.interrupted() || !paren_on_same_line {
        //     formatter.indent(writer)?;
        // }
        // write!(writer, "{}", close_paren)?;

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

        // for stmt in self.statements() {
        //     stmt.format(writer, formatter)?;
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

impl Formattable for ScatterStatement {
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

        // let scatter_keyword = first_child_of_kind(self.syntax(),
        // SyntaxKind::ScatterKeyword); formatter.indent(writer)?;
        // write!(writer, "{}", scatter_keyword)?;
        // format_inline_comment(&scatter_keyword, writer, formatter, true)?;

        // let open_paren = first_child_of_kind(self.syntax(), SyntaxKind::OpenParen);
        // format_preceding_comments(&open_paren, writer, formatter, true)?;
        // // Open parens should ignore the "+1 rule" followed by other interrupted
        // // elements.
        // if formatter.interrupted() {
        //     formatter.reset_interrupted();
        //     formatter.indent(writer)?;
        // } else {
        //     write!(writer, "{}", SPACE)?;
        // }
        // write!(writer, "{}", open_paren)?;
        // format_inline_comment(&open_paren, writer, formatter, true)?;

        // let ident = self.variable();
        // format_preceding_comments(
        //     &SyntaxElement::from(ident.syntax().clone()),
        //     writer,
        //     formatter,
        //     true,
        // )?;
        // if formatter.interrupted() {
        //     formatter.indent(writer)?;
        // }
        // ident.format(writer, formatter)?;
        // format_inline_comment(
        //     &SyntaxElement::from(ident.syntax().clone()),
        //     writer,
        //     formatter,
        //     true,
        // )?;

        // let in_keyword = first_child_of_kind(self.syntax(), SyntaxKind::InKeyword);
        // format_preceding_comments(&in_keyword, writer, formatter, true)?;
        // formatter.space_or_indent(writer)?;
        // write!(writer, "{}", in_keyword)?;
        // format_inline_comment(&in_keyword, writer, formatter, true)?;

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
        //     &SyntaxElement::from(expr.syntax().clone()),
        //     writer,
        //     formatter,
        //     true,
        // )?;

        // let close_paren = first_child_of_kind(self.syntax(), SyntaxKind::CloseParen);
        // format_preceding_comments(&close_paren, writer, formatter, true)?;
        // if formatter.interrupted() {
        //     formatter.indent(writer)?;
        // }
        // write!(writer, "{}", close_paren)?;
        // format_inline_comment(&close_paren, writer, formatter, true)?;

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

        // for stmt in self.statements() {
        //     stmt.format(writer, formatter)?;
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

impl Formattable for WorkflowStatement {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        formatter: &mut Formatter,
    ) -> std::fmt::Result {
        match self {
            WorkflowStatement::Call(c) => c.format(writer, formatter),
            WorkflowStatement::Conditional(c) => c.format(writer, formatter),
            WorkflowStatement::Scatter(s) => s.format(writer, formatter),
            WorkflowStatement::Declaration(d) => Decl::Bound(d.clone()).format(writer, formatter),
        }
    }
}

impl Formattable for WorkflowDefinition {
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

        // let workflow_keyword = first_child_of_kind(self.syntax(),
        // SyntaxKind::WorkflowKeyword); write!(writer, "{}",
        // workflow_keyword)?; format_inline_comment(&workflow_keyword, writer,
        // formatter, true)?;

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
        // let mut body_str = String::new();
        // let mut output_section_str = String::new();
        // let mut hints_section_str = String::new();

        // for item in self.items() {
        //     match item {
        //         WorkflowItem::Metadata(m) => {
        //             m.format(&mut meta_section_str, formatter)?;
        //         }
        //         WorkflowItem::ParameterMetadata(pm) => {
        //             pm.format(&mut parameter_meta_section_str, formatter)?;
        //         }
        //         WorkflowItem::Input(i) => {
        //             i.format(&mut input_section_str, formatter)?;
        //         }
        //         WorkflowItem::Call(c) => {
        //             c.format(&mut body_str, formatter)?;
        //         }
        //         WorkflowItem::Conditional(c) => {
        //             c.format(&mut body_str, formatter)?;
        //         }
        //         WorkflowItem::Scatter(s) => {
        //             s.format(&mut body_str, formatter)?;
        //         }
        //         WorkflowItem::Declaration(d) => {
        //             Decl::Bound(d).format(&mut body_str, formatter)?;
        //         }
        //         WorkflowItem::Output(o) => {
        //             o.format(&mut output_section_str, formatter)?;
        //         }
        //         WorkflowItem::Hints(h) => {
        //             h.format(&mut hints_section_str, formatter)?;
        //         }
        //     }
        // }

        // let mut first_section = true;
        // if !meta_section_str.is_empty() {
        //     first_section = false;
        //     write!(writer, "{}", meta_section_str)?;
        // }
        // if !parameter_meta_section_str.is_empty() {
        //     if first_section {
        //         first_section = false;
        //     } else {
        //         write!(writer, "{}", NEWLINE)?;
        //     }
        //     write!(writer, "{}", parameter_meta_section_str)?;
        // }
        // if !input_section_str.is_empty() {
        //     if first_section {
        //         first_section = false;
        //     } else {
        //         write!(writer, "{}", NEWLINE)?;
        //     }
        //     write!(writer, "{}", input_section_str)?;
        // }
        // if !body_str.is_empty() {
        //     if first_section {
        //         first_section = false;
        //     } else {
        //         write!(writer, "{}", NEWLINE)?;
        //     }
        //     write!(writer, "{}", body_str)?;
        // }
        // if !output_section_str.is_empty() {
        //     if first_section {
        //         first_section = false;
        //     } else {
        //         write!(writer, "{}", NEWLINE)?;
        //     }
        //     write!(writer, "{}", output_section_str)?;
        // }
        // if !hints_section_str.is_empty() {
        //     if !first_section {
        //         write!(writer, "{}", NEWLINE)?;
        //     }
        //     write!(writer, "{}", hints_section_str)?;
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
