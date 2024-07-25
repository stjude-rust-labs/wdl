//! A module for formatting elements in workflows.

use std::fmt::Write;

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
use wdl_ast::SyntaxElement;
use wdl_ast::SyntaxKind;

use super::comments::format_inline_comment;
use super::comments::format_preceding_comments;
use super::format_state::SPACE;
use super::FormatState;
use super::Formattable;
use super::NEWLINE;

impl Formattable for CallAlias {
    fn format(&self, buffer: &mut String, state: &mut FormatState) -> Result<()> {
        format_preceding_comments(&self.syntax_element(), buffer, state, true)?;

        let as_keyword = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::AsKeyword)
            .expect("Call Alias should have an as keyword");
        state.space_or_indent(buffer)?;
        buffer.push_str("as");
        format_inline_comment(&as_keyword, buffer, state, true)?;

        let ident = self.name();
        format_preceding_comments(&ident.syntax_element(), buffer, state, true)?;
        state.space_or_indent(buffer)?;
        ident.format(buffer, state)?;
        format_inline_comment(&self.syntax_element(), buffer, state, true)?;
        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Node(self.syntax().clone())
    }
}

impl Formattable for CallAfter {
    fn format(&self, buffer: &mut String, state: &mut FormatState) -> Result<()> {
        format_preceding_comments(&self.syntax_element(), buffer, state, true)?;

        let after_keyword = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::AfterKeyword)
            .expect("Call After should have an after keyword");
        state.space_or_indent(buffer)?;
        buffer.push_str("after");
        format_inline_comment(&after_keyword, buffer, state, true)?;

        let ident = self.name();
        format_preceding_comments(&ident.syntax_element(), buffer, state, true)?;
        state.space_or_indent(buffer)?;
        ident.format(buffer, state)?;
        format_inline_comment(&self.syntax_element(), buffer, state, true)?;
        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Node(self.syntax().clone())
    }
}

impl Formattable for CallInputItem {
    fn format(&self, buffer: &mut String, state: &mut FormatState) -> Result<()> {
        let name = self.name();
        name.format(buffer, state)?;
        format_inline_comment(&name.syntax_element(), buffer, state, true)?;

        if let Some(expr) = self.expr() {
            let equal_sign = self
                .syntax()
                .children_with_tokens()
                .find(|element| element.kind() == SyntaxKind::Assignment)
                .expect("Call Input Item should have an equal sign");
            format_preceding_comments(&equal_sign, buffer, state, true)?;
            state.space_or_indent(buffer)?;
            buffer.push('=');
            format_inline_comment(&equal_sign, buffer, state, true)?;

            format_preceding_comments(&expr.syntax_element(), buffer, state, true)?;
            state.space_or_indent(buffer)?;
            expr.format(buffer, state)?;
        }

        format_inline_comment(&self.syntax_element(), buffer, state, true)?;

        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Node(self.syntax().clone())
    }
}

impl Formattable for CallStatement {
    fn format(&self, buffer: &mut String, state: &mut FormatState) -> Result<()> {
        format_preceding_comments(&self.syntax_element(), buffer, state, false)?;

        let call_keyword = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::CallKeyword)
            .expect("Call Statement should have a call keyword");
        state.indent(buffer)?;
        buffer.push_str("call");
        format_inline_comment(&call_keyword, buffer, state, true)?;

        let target = self.target();
        format_preceding_comments(
            &SyntaxElement::Node(target.syntax().clone()),
            buffer,
            state,
            true,
        )?;
        state.space_or_indent(buffer)?;
        write!(buffer, "{}", target.syntax())?;
        format_inline_comment(
            &SyntaxElement::Node(target.syntax().clone()),
            buffer,
            state,
            true,
        )?;

        if let Some(alias) = self.alias() {
            alias.format(buffer, state)?;
        }

        for after in self.after() {
            after.format(buffer, state)?;
        }

        let inputs: Vec<_> = self.inputs().collect();
        if !inputs.is_empty() {
            let open_brace = self
                .syntax()
                .children_with_tokens()
                .find(|element| element.kind() == SyntaxKind::OpenBrace)
                .expect("Call Statement should have an open brace");
            format_preceding_comments(&open_brace, buffer, state, true)?;
            // Open braces should ignore the "+1 rule" followed by other interrupted
            // elements.
            if state.interrupted() {
                state.reset_interrupted();
                state.indent(buffer)?;
            } else {
                buffer.push_str(SPACE);
            }
            buffer.push('{');
            format_inline_comment(&open_brace, buffer, state, true)?;

            let input_keyword = self
                .syntax()
                .children_with_tokens()
                .find(|element| element.kind() == SyntaxKind::InputKeyword)
                .expect("Call Statement should have an input keyword");
            format_preceding_comments(&input_keyword, buffer, state, true)?;
            state.space_or_indent(buffer)?;
            buffer.push_str("input");
            format_inline_comment(&input_keyword, buffer, state, true)?;

            let colon = self
                .syntax()
                .children_with_tokens()
                .find(|element| element.kind() == SyntaxKind::Colon)
                .expect("Call Statement should have a colon");
            format_preceding_comments(&colon, buffer, state, true)?;
            if state.interrupted() {
                state.indent(buffer)?;
            }
            buffer.push(':');
            format_inline_comment(&colon, buffer, state, true)?;

            if inputs.len() == 1 {
                let input = inputs.first().expect("Inputs should have a first element");
                format_preceding_comments(&input.syntax_element(), buffer, state, true)?;
                state.space_or_indent(buffer)?;
                input.format(buffer, state)?;

                let close_brace = self
                    .syntax()
                    .children_with_tokens()
                    .find(|element| element.kind() == SyntaxKind::CloseBrace)
                    .expect("Call Statement should have a close brace");
                format_preceding_comments(&close_brace, buffer, state, true)?;
                state.space_or_indent(buffer)?;
                buffer.push('}');
            } else {
                // multiple inputs
                let mut commas = self
                    .syntax()
                    .children_with_tokens()
                    .filter(|c| c.kind() == SyntaxKind::Comma);

                state.increment_indent();

                for input in inputs {
                    if !state.interrupted() {
                        buffer.push_str(NEWLINE);
                    } else {
                        state.reset_interrupted();
                    }
                    format_preceding_comments(&input.syntax_element(), buffer, state, false)?;
                    state.indent(buffer)?;
                    input.format(buffer, state)?;
                    if let Some(cur_comma) = commas.next() {
                        format_preceding_comments(&cur_comma, buffer, state, true)?;
                        buffer.push(',');
                        format_inline_comment(&cur_comma, buffer, state, true)?;
                    } else {
                        buffer.push(',');
                    }
                }
                if !state.interrupted() {
                    buffer.push_str(NEWLINE);
                }

                state.decrement_indent();

                let close_brace = self
                    .syntax()
                    .children_with_tokens()
                    .find(|element| element.kind() == SyntaxKind::CloseBrace)
                    .expect("Call Statement should have a close brace");
                format_preceding_comments(&close_brace, buffer, state, false)?;
                state.indent(buffer)?;
                buffer.push('}');
            }
        }

        format_inline_comment(&self.syntax_element(), buffer, state, false)?;

        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Node(self.syntax().clone())
    }
}

impl Formattable for ConditionalStatement {
    fn format(&self, buffer: &mut String, state: &mut FormatState) -> Result<()> {
        format_preceding_comments(&self.syntax_element(), buffer, state, false)?;

        let if_keyword = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::IfKeyword)
            .expect("Conditional Statement should have an if keyword");
        state.indent(buffer)?;
        buffer.push_str("if");
        format_inline_comment(&if_keyword, buffer, state, true)?;

        let open_paren = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::OpenParen)
            .expect("Conditional Statement should have an open parenthesis");
        format_preceding_comments(&open_paren, buffer, state, true)?;
        // Open parens should ignore the "+1 rule" followed by other interrupted
        // elements.
        if state.interrupted() {
            state.reset_interrupted();
            state.indent(buffer)?;
        } else {
            buffer.push_str(SPACE);
        }
        buffer.push('(');

        let mut paren_on_same_line = true;
        let expr = self.expr();
        let multiline_expr = expr.syntax().to_string().contains(NEWLINE);

        format_inline_comment(&open_paren, buffer, state, !multiline_expr)?;
        if multiline_expr {
            state.increment_indent();
            paren_on_same_line = false;
        }
        format_preceding_comments(&expr.syntax_element(), buffer, state, !multiline_expr)?;
        if state.interrupted() || multiline_expr {
            state.indent(buffer)?;
            paren_on_same_line = false;
        }
        expr.format(buffer, state)?;
        format_inline_comment(&expr.syntax_element(), buffer, state, !multiline_expr)?;
        if state.interrupted() {
            paren_on_same_line = false;
        }

        let close_paren = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::CloseParen)
            .expect("Conditional Statement should have a close parenthesis");
        format_preceding_comments(&close_paren, buffer, state, !multiline_expr)?;
        if state.interrupted() || !paren_on_same_line {
            state.indent(buffer)?;
        }
        buffer.push(')');

        let open_brace = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::OpenBrace)
            .expect("Conditional Statement should have an open brace");
        format_preceding_comments(&open_brace, buffer, state, true)?;
        // Open braces should ignore the "+1 rule" followed by other interrupted
        // elements.
        if state.interrupted() {
            state.reset_interrupted();
            state.indent(buffer)?;
        } else {
            buffer.push_str(SPACE);
        }
        buffer.push('{');
        format_inline_comment(&open_brace, buffer, state, false)?;

        state.increment_indent();

        for stmt in self.statements() {
            stmt.format(buffer, state)?;
        }

        state.decrement_indent();

        let close_brace = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::CloseBrace)
            .expect("Conditional Statement should have a close brace");
        format_preceding_comments(&close_brace, buffer, state, false)?;
        state.indent(buffer)?;
        buffer.push('}');
        format_inline_comment(&self.syntax_element(), buffer, state, false)?;

        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Node(self.syntax().clone())
    }
}

impl Formattable for ScatterStatement {
    fn format(&self, buffer: &mut String, state: &mut FormatState) -> Result<()> {
        format_preceding_comments(&self.syntax_element(), buffer, state, false)?;

        let scatter_keyword = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::ScatterKeyword)
            .expect("Scatter Statement should have a scatter keyword");
        state.indent(buffer)?;
        buffer.push_str("scatter");
        format_inline_comment(&scatter_keyword, buffer, state, true)?;

        let open_paren = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::OpenParen)
            .expect("Scatter Statement should have an open parenthesis");
        format_preceding_comments(&open_paren, buffer, state, true)?;
        // Open parens should ignore the "+1 rule" followed by other interrupted
        // elements.
        if state.interrupted() {
            state.reset_interrupted();
            state.indent(buffer)?;
        } else {
            buffer.push_str(SPACE);
        }
        buffer.push('(');
        format_inline_comment(&open_paren, buffer, state, true)?;

        let ident = self.variable();
        format_preceding_comments(&ident.syntax_element(), buffer, state, true)?;
        if state.interrupted() {
            state.indent(buffer)?;
        }
        ident.format(buffer, state)?;
        format_inline_comment(&ident.syntax_element(), buffer, state, true)?;

        let in_keyword = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::InKeyword)
            .expect("Scatter Statement should have an in keyword");
        format_preceding_comments(&in_keyword, buffer, state, true)?;
        state.space_or_indent(buffer)?;
        buffer.push_str("in");
        format_inline_comment(&in_keyword, buffer, state, true)?;

        let expr = self.expr();
        format_preceding_comments(&expr.syntax_element(), buffer, state, true)?;
        state.space_or_indent(buffer)?;
        expr.format(buffer, state)?;
        format_inline_comment(&expr.syntax_element(), buffer, state, true)?;

        let close_paren = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::CloseParen)
            .expect("Scatter Statement should have a close parenthesis");
        format_preceding_comments(&close_paren, buffer, state, true)?;
        if state.interrupted() {
            state.indent(buffer)?;
        }
        buffer.push(')');
        format_inline_comment(&close_paren, buffer, state, true)?;

        let open_brace = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::OpenBrace)
            .expect("Scatter Statement should have an open brace");
        format_preceding_comments(&open_brace, buffer, state, true)?;
        // Open braces should ignore the "+1 rule" followed by other interrupted
        // elements.
        if state.interrupted() {
            state.reset_interrupted();
            state.indent(buffer)?;
        } else {
            buffer.push_str(SPACE);
        }
        buffer.push('{');
        format_inline_comment(&open_brace, buffer, state, false)?;

        state.increment_indent();

        for stmt in self.statements() {
            stmt.format(buffer, state)?;
        }

        state.decrement_indent();

        let close_brace = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::CloseBrace)
            .expect("Scatter Statement should have a close brace");
        format_preceding_comments(&close_brace, buffer, state, false)?;
        state.indent(buffer)?;
        buffer.push('}');
        format_inline_comment(&self.syntax_element(), buffer, state, false)?;

        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Node(self.syntax().clone())
    }
}

impl Formattable for WorkflowStatement {
    fn format(&self, buffer: &mut String, state: &mut FormatState) -> Result<()> {
        match self {
            WorkflowStatement::Call(c) => c.format(buffer, state),
            WorkflowStatement::Conditional(c) => c.format(buffer, state),
            WorkflowStatement::Scatter(s) => s.format(buffer, state),
            WorkflowStatement::Declaration(d) => {
                Decl::Bound(d.clone()).format(buffer, state)
            }
        }
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Node(self.syntax().clone())
    }
}

impl Formattable for WorkflowDefinition {
    fn format(&self, buffer: &mut String, state: &mut FormatState) -> Result<()> {
        format_preceding_comments(&self.syntax_element(), buffer, state, false)?;

        let workflow_keyword = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::WorkflowKeyword)
            .expect("Workflow should have a workflow keyword");
        buffer.push_str("workflow");
        format_inline_comment(&workflow_keyword, buffer, state, true)?;

        let name = self.name();
        format_preceding_comments(&name.syntax_element(), buffer, state, true)?;
        state.space_or_indent(buffer)?;
        name.format(buffer, state)?;
        format_inline_comment(&name.syntax_element(), buffer, state, true)?;

        let open_brace = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::OpenBrace)
            .expect("Workflow should have an open brace");
        format_preceding_comments(&open_brace, buffer, state, true)?;
        // Open braces should ignore the "+1 rule" followed by other interrupted
        // elements.
        if state.interrupted() {
            state.reset_interrupted();
            state.indent(buffer)?;
        } else {
            buffer.push_str(SPACE);
        }
        buffer.push('{');
        format_inline_comment(&open_brace, buffer, state, false)?;

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
            buffer.push_str(&meta_section_str);
        }
        if !parameter_meta_section_str.is_empty() {
            if first_section {
                first_section = false;
            } else {
                buffer.push_str(NEWLINE);
            }
            buffer.push_str(&parameter_meta_section_str);
        }
        if !input_section_str.is_empty() {
            if first_section {
                first_section = false;
            } else {
                buffer.push_str(NEWLINE);
            }
            buffer.push_str(&input_section_str);
        }
        if !body_str.is_empty() {
            if first_section {
                first_section = false;
            } else {
                buffer.push_str(NEWLINE);
            }
            buffer.push_str(&body_str);
        }
        if !output_section_str.is_empty() {
            if first_section {
                first_section = false;
            } else {
                buffer.push_str(NEWLINE);
            }
            buffer.push_str(&output_section_str);
        }
        if !hints_section_str.is_empty() {
            if !first_section {
                buffer.push_str(NEWLINE);
            }
            buffer.push_str(&hints_section_str);
        }

        state.decrement_indent();

        let close_brace = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::CloseBrace)
            .expect("Workflow should have a close brace");
        format_preceding_comments(&close_brace, buffer, state, false)?;
        state.indent(buffer)?;
        buffer.push('}');
        format_inline_comment(&self.syntax_element(), buffer, state, false)?;

        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Node(self.syntax().clone())
    }
}
