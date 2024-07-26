//! A module for formatting elements in workflows.

use anyhow::Result;
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

use super::comments::format_inline_comment;
use super::comments::format_preceding_comments;
use super::state::SPACE;
use super::Formattable;
use super::State;
use super::NEWLINE;

impl Formattable for CallAlias {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, state: &mut State) -> Result<()> {
        format_preceding_comments(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            state,
            true,
        )?;

        let as_keyword = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::AsKeyword)
            .expect("Call Alias should have an as keyword");
        state.space_or_indent(writer)?;
        write!(writer, "{}", as_keyword)?;
        format_inline_comment(&as_keyword, writer, state, true)?;

        let ident = self.name();
        format_preceding_comments(
            &SyntaxElement::from(ident.syntax().clone()),
            writer,
            state,
            true,
        )?;
        state.space_or_indent(writer)?;
        ident.format(writer, state)?;
        format_inline_comment(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            state,
            true,
        )?;
        Ok(())
    }
}

impl Formattable for CallAfter {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, state: &mut State) -> Result<()> {
        format_preceding_comments(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            state,
            true,
        )?;

        let after_keyword = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::AfterKeyword)
            .expect("Call After should have an after keyword");
        state.space_or_indent(writer)?;
        write!(writer, "{}", after_keyword)?;
        format_inline_comment(&after_keyword, writer, state, true)?;

        let ident = self.name();
        format_preceding_comments(
            &SyntaxElement::from(ident.syntax().clone()),
            writer,
            state,
            true,
        )?;
        state.space_or_indent(writer)?;
        ident.format(writer, state)?;
        format_inline_comment(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            state,
            true,
        )?;
        Ok(())
    }
}

impl Formattable for CallInputItem {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, state: &mut State) -> Result<()> {
        let name = self.name();
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
                .expect("Call Input Item should have an equal sign");
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
            true,
        )?;

        Ok(())
    }
}

impl Formattable for CallStatement {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, state: &mut State) -> Result<()> {
        format_preceding_comments(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            state,
            false,
        )?;

        let call_keyword = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::CallKeyword)
            .expect("Call Statement should have a call keyword");
        state.indent(writer)?;
        write!(writer, "{}", call_keyword)?;
        format_inline_comment(&call_keyword, writer, state, true)?;

        let target = self.target();
        format_preceding_comments(
            &SyntaxElement::Node(target.syntax().clone()),
            writer,
            state,
            true,
        )?;
        state.space_or_indent(writer)?;
        write!(writer, "{}", target.syntax())?;
        format_inline_comment(
            &SyntaxElement::Node(target.syntax().clone()),
            writer,
            state,
            true,
        )?;

        if let Some(alias) = self.alias() {
            alias.format(writer, state)?;
        }

        for after in self.after() {
            after.format(writer, state)?;
        }

        let inputs: Vec<_> = self.inputs().collect();
        if !inputs.is_empty() {
            let open_brace = self
                .syntax()
                .children_with_tokens()
                .find(|element| element.kind() == SyntaxKind::OpenBrace)
                .expect("Call Statement with input(s) should have an open brace");
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
            format_inline_comment(&open_brace, writer, state, true)?;

            let input_keyword = self
                .syntax()
                .children_with_tokens()
                .find(|element| element.kind() == SyntaxKind::InputKeyword)
                .expect("Call Statement with input(s) should have an input keyword");
            format_preceding_comments(&input_keyword, writer, state, true)?;
            state.space_or_indent(writer)?;
            write!(writer, "{}", input_keyword)?;
            format_inline_comment(&input_keyword, writer, state, true)?;

            let colon = self
                .syntax()
                .children_with_tokens()
                .find(|element| element.kind() == SyntaxKind::Colon)
                .expect("Call Statement with input(s) should have a colon");
            format_preceding_comments(&colon, writer, state, true)?;
            if state.interrupted() {
                state.indent(writer)?;
            }
            write!(writer, "{}", colon)?;
            format_inline_comment(&colon, writer, state, true)?;

            if inputs.len() == 1 {
                let input = inputs.first().expect("Inputs should have a first element");
                format_preceding_comments(
                    &SyntaxElement::from(input.syntax().clone()),
                    writer,
                    state,
                    true,
                )?;
                state.space_or_indent(writer)?;
                input.format(writer, state)?;

                let close_brace = self
                    .syntax()
                    .children_with_tokens()
                    .find(|element| element.kind() == SyntaxKind::CloseBrace)
                    .expect("Call Statement should have a close brace");
                format_preceding_comments(&close_brace, writer, state, true)?;
                state.space_or_indent(writer)?;
                write!(writer, "{}", close_brace)?;
            } else {
                // multiple inputs
                let mut commas = self
                    .syntax()
                    .children_with_tokens()
                    .filter(|c| c.kind() == SyntaxKind::Comma);

                state.increment_indent();

                for input in inputs {
                    if !state.interrupted() {
                        write!(writer, "{}", NEWLINE)?;
                    } else {
                        state.reset_interrupted();
                    }
                    format_preceding_comments(
                        &SyntaxElement::from(input.syntax().clone()),
                        writer,
                        state,
                        false,
                    )?;
                    state.indent(writer)?;
                    input.format(writer, state)?;
                    if let Some(cur_comma) = commas.next() {
                        format_preceding_comments(&cur_comma, writer, state, true)?;
                        write!(writer, ",")?;
                        format_inline_comment(&cur_comma, writer, state, true)?;
                    } else {
                        write!(writer, ",")?;
                    }
                }
                if !state.interrupted() {
                    write!(writer, "{}", NEWLINE)?;
                }

                state.decrement_indent();

                let close_brace = self
                    .syntax()
                    .children_with_tokens()
                    .find(|element| element.kind() == SyntaxKind::CloseBrace)
                    .expect("Call Statement should have a close brace");
                format_preceding_comments(&close_brace, writer, state, false)?;
                state.indent(writer)?;
                write!(writer, "{}", close_brace)?;
            }
        }

        format_inline_comment(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            state,
            false,
        )?;

        Ok(())
    }
}

impl Formattable for ConditionalStatement {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, state: &mut State) -> Result<()> {
        format_preceding_comments(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            state,
            false,
        )?;

        let if_keyword = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::IfKeyword)
            .expect("Conditional Statement should have an if keyword");
        state.indent(writer)?;
        write!(writer, "{}", if_keyword)?;
        format_inline_comment(&if_keyword, writer, state, true)?;

        let open_paren = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::OpenParen)
            .expect("Conditional Statement should have an open parenthesis");
        format_preceding_comments(&open_paren, writer, state, true)?;
        // Open parens should ignore the "+1 rule" followed by other interrupted
        // elements.
        if state.interrupted() {
            state.reset_interrupted();
            state.indent(writer)?;
        } else {
            write!(writer, "{}", SPACE)?;
        }
        write!(writer, "{}", open_paren)?;

        let mut paren_on_same_line = true;
        let expr = self.expr();
        let multiline_expr = expr.syntax().to_string().contains(NEWLINE);

        format_inline_comment(&open_paren, writer, state, !multiline_expr)?;
        if multiline_expr {
            state.increment_indent();
            paren_on_same_line = false;
        }
        format_preceding_comments(
            &SyntaxElement::from(expr.syntax().clone()),
            writer,
            state,
            !multiline_expr,
        )?;
        if state.interrupted() || multiline_expr {
            state.indent(writer)?;
            paren_on_same_line = false;
        }
        expr.format(writer, state)?;
        format_inline_comment(
            &SyntaxElement::from(expr.syntax().clone()),
            writer,
            state,
            !multiline_expr,
        )?;
        if state.interrupted() {
            paren_on_same_line = false;
        }

        let close_paren = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::CloseParen)
            .expect("Conditional Statement should have a close parenthesis");
        format_preceding_comments(&close_paren, writer, state, !multiline_expr)?;
        if state.interrupted() || !paren_on_same_line {
            state.indent(writer)?;
        }
        write!(writer, "{}", close_paren)?;

        let open_brace = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::OpenBrace)
            .expect("Conditional Statement should have an open brace");
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

        for stmt in self.statements() {
            stmt.format(writer, state)?;
        }

        state.decrement_indent();

        let close_brace = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::CloseBrace)
            .expect("Conditional Statement should have a close brace");
        format_preceding_comments(&close_brace, writer, state, false)?;
        state.indent(writer)?;
        write!(writer, "{}", close_brace)?;
        format_inline_comment(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            state,
            false,
        )?;

        Ok(())
    }
}

impl Formattable for ScatterStatement {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, state: &mut State) -> Result<()> {
        format_preceding_comments(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            state,
            false,
        )?;

        let scatter_keyword = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::ScatterKeyword)
            .expect("Scatter Statement should have a scatter keyword");
        state.indent(writer)?;
        write!(writer, "{}", scatter_keyword)?;
        format_inline_comment(&scatter_keyword, writer, state, true)?;

        let open_paren = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::OpenParen)
            .expect("Scatter Statement should have an open parenthesis");
        format_preceding_comments(&open_paren, writer, state, true)?;
        // Open parens should ignore the "+1 rule" followed by other interrupted
        // elements.
        if state.interrupted() {
            state.reset_interrupted();
            state.indent(writer)?;
        } else {
            write!(writer, "{}", SPACE)?;
        }
        write!(writer, "{}", open_paren)?;
        format_inline_comment(&open_paren, writer, state, true)?;

        let ident = self.variable();
        format_preceding_comments(
            &SyntaxElement::from(ident.syntax().clone()),
            writer,
            state,
            true,
        )?;
        if state.interrupted() {
            state.indent(writer)?;
        }
        ident.format(writer, state)?;
        format_inline_comment(
            &SyntaxElement::from(ident.syntax().clone()),
            writer,
            state,
            true,
        )?;

        let in_keyword = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::InKeyword)
            .expect("Scatter Statement should have an in keyword");
        format_preceding_comments(&in_keyword, writer, state, true)?;
        state.space_or_indent(writer)?;
        write!(writer, "{}", in_keyword)?;
        format_inline_comment(&in_keyword, writer, state, true)?;

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
            &SyntaxElement::from(expr.syntax().clone()),
            writer,
            state,
            true,
        )?;

        let close_paren = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::CloseParen)
            .expect("Scatter Statement should have a close parenthesis");
        format_preceding_comments(&close_paren, writer, state, true)?;
        if state.interrupted() {
            state.indent(writer)?;
        }
        write!(writer, "{}", close_paren)?;
        format_inline_comment(&close_paren, writer, state, true)?;

        let open_brace = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::OpenBrace)
            .expect("Scatter Statement should have an open brace");
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

        for stmt in self.statements() {
            stmt.format(writer, state)?;
        }

        state.decrement_indent();

        let close_brace = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::CloseBrace)
            .expect("Scatter Statement should have a close brace");
        format_preceding_comments(&close_brace, writer, state, false)?;
        state.indent(writer)?;
        write!(writer, "{}", close_brace)?;
        format_inline_comment(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            state,
            false,
        )?;

        Ok(())
    }
}

impl Formattable for WorkflowStatement {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, state: &mut State) -> Result<()> {
        match self {
            WorkflowStatement::Call(c) => c.format(writer, state),
            WorkflowStatement::Conditional(c) => c.format(writer, state),
            WorkflowStatement::Scatter(s) => s.format(writer, state),
            WorkflowStatement::Declaration(d) => Decl::Bound(d.clone()).format(writer, state),
        }
    }
}

impl Formattable for WorkflowDefinition {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, state: &mut State) -> Result<()> {
        format_preceding_comments(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            state,
            false,
        )?;

        let workflow_keyword = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::WorkflowKeyword)
            .expect("Workflow should have a workflow keyword");
        write!(writer, "{}", workflow_keyword)?;
        format_inline_comment(&workflow_keyword, writer, state, true)?;

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
            .expect("Workflow should have an open brace");
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

        let mut meta_section_str = String::new();
        let mut parameter_meta_section_str = String::new();
        let mut input_section_str = String::new();
        let mut body_str = String::new();
        let mut output_section_str = String::new();
        let mut hints_section_str = String::new();

        for item in self.items() {
            match item {
                WorkflowItem::Metadata(m) => {
                    m.format(&mut meta_section_str, state)?;
                }
                WorkflowItem::ParameterMetadata(pm) => {
                    pm.format(&mut parameter_meta_section_str, state)?;
                }
                WorkflowItem::Input(i) => {
                    i.format(&mut input_section_str, state)?;
                }
                WorkflowItem::Call(c) => {
                    c.format(&mut body_str, state)?;
                }
                WorkflowItem::Conditional(c) => {
                    c.format(&mut body_str, state)?;
                }
                WorkflowItem::Scatter(s) => {
                    s.format(&mut body_str, state)?;
                }
                WorkflowItem::Declaration(d) => {
                    Decl::Bound(d).format(&mut body_str, state)?;
                }
                WorkflowItem::Output(o) => {
                    o.format(&mut output_section_str, state)?;
                }
                WorkflowItem::Hints(h) => {
                    h.format(&mut hints_section_str, state)?;
                }
            }
        }

        let mut first_section = true;
        if !meta_section_str.is_empty() {
            first_section = false;
            write!(writer, "{}", meta_section_str)?;
        }
        if !parameter_meta_section_str.is_empty() {
            if first_section {
                first_section = false;
            } else {
                write!(writer, "{}", NEWLINE)?;
            }
            write!(writer, "{}", parameter_meta_section_str)?;
        }
        if !input_section_str.is_empty() {
            if first_section {
                first_section = false;
            } else {
                write!(writer, "{}", NEWLINE)?;
            }
            write!(writer, "{}", input_section_str)?;
        }
        if !body_str.is_empty() {
            if first_section {
                first_section = false;
            } else {
                write!(writer, "{}", NEWLINE)?;
            }
            write!(writer, "{}", body_str)?;
        }
        if !output_section_str.is_empty() {
            if first_section {
                first_section = false;
            } else {
                write!(writer, "{}", NEWLINE)?;
            }
            write!(writer, "{}", output_section_str)?;
        }
        if !hints_section_str.is_empty() {
            if !first_section {
                write!(writer, "{}", NEWLINE)?;
            }
            write!(writer, "{}", hints_section_str)?;
        }

        state.decrement_indent();

        let close_brace = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::CloseBrace)
            .expect("Workflow should have a close brace");
        format_preceding_comments(&close_brace, writer, state, false)?;
        state.indent(writer)?;
        write!(writer, "{}", close_brace)?;
        format_inline_comment(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            state,
            false,
        )?;

        Ok(())
    }
}
