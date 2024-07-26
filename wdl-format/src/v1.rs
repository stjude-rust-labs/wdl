//! A module for formatting WDL v1 elements.

use std::fmt::Write;

use anyhow::Result;
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
    fn format(&self, buffer: &mut String, _state: &mut State) -> Result<()> {
        buffer.push(STRING_TERMINATOR);
        for part in self.parts() {
            match part {
                StringPart::Text(text) => {
                    write!(buffer, "{}", text.as_str())?;
                }
                StringPart::Placeholder(placeholder) => {
                    write!(buffer, "{}", placeholder.syntax())?;
                }
            }
        }
        buffer.push(STRING_TERMINATOR);
        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Node(self.syntax().clone())
    }
}

impl Formattable for LiteralBoolean {
    fn format(&self, buffer: &mut String, _state: &mut State) -> Result<()> {
        buffer.push_str(&self.value().to_string());
        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Node(self.syntax().clone())
    }
}

impl Formattable for LiteralFloat {
    fn format(&self, buffer: &mut String, _state: &mut State) -> Result<()> {
        write!(buffer, "{}", self.syntax())?;
        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Node(self.syntax().clone())
    }
}

impl Formattable for LiteralInteger {
    fn format(&self, buffer: &mut String, _state: &mut State) -> Result<()> {
        write!(buffer, "{}", self.syntax())?;
        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Node(self.syntax().clone())
    }
}

impl Formattable for Type {
    fn format(&self, buffer: &mut String, _state: &mut State) -> Result<()> {
        write!(buffer, "{}", self.syntax())?;
        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Node(self.syntax().clone())
    }
}

impl Formattable for Expr {
    fn format(&self, buffer: &mut String, _state: &mut State) -> Result<()> {
        write!(buffer, "{}", self.syntax())?;
        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Node(self.syntax().clone())
    }
}

impl Formattable for Decl {
    fn format(&self, buffer: &mut String, state: &mut State) -> Result<()> {
        format_preceding_comments(&self.syntax_element(), buffer, state, false)?;

        let ty = self.ty();
        state.indent(buffer)?;
        ty.format(buffer, state)?;
        format_inline_comment(&ty.syntax_element(), buffer, state, true)?;

        let name = self.name();
        format_preceding_comments(&name.syntax_element(), buffer, state, true)?;
        state.space_or_indent(buffer)?;
        name.format(buffer, state)?;
        format_inline_comment(&name.syntax_element(), buffer, state, true)?;

        if let Some(expr) = self.expr() {
            let eq = self
                .syntax()
                .children_with_tokens()
                .find(|element| element.kind() == SyntaxKind::Assignment)
                .expect("Bound declaration should have an equals sign");
            format_preceding_comments(&eq, buffer, state, true)?;
            state.space_or_indent(buffer)?;
            buffer.push_str(&eq.to_string());
            format_inline_comment(&eq, buffer, state, true)?;

            format_preceding_comments(&expr.syntax_element(), buffer, state, true)?;
            state.space_or_indent(buffer)?;
            expr.format(buffer, state)?;
        }
        format_inline_comment(&self.syntax_element(), buffer, state, false)?;

        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Node(self.syntax().clone())
    }
}

impl Formattable for InputSection {
    fn format(&self, buffer: &mut String, state: &mut State) -> Result<()> {
        format_preceding_comments(&self.syntax_element(), buffer, state, false)?;

        let input_keyword = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::InputKeyword)
            .expect("Input Section should have an input keyword");
        state.indent(buffer)?;
        buffer.push_str(&input_keyword.to_string());
        format_inline_comment(&input_keyword, buffer, state, true)?;

        let open_brace = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::OpenBrace)
            .expect("Input Section should have an open brace");
        format_preceding_comments(&open_brace, buffer, state, true)?;
        // Open braces should ignore the "+1 rule" followed by other interrupted
        // elements.
        if state.interrupted() {
            state.reset_interrupted();
            state.indent(buffer)?;
        } else {
            buffer.push_str(SPACE);
        }
        buffer.push_str(&open_brace.to_string());
        format_inline_comment(&open_brace, buffer, state, false)?;

        state.increment_indent();

        for decl in self.declarations() {
            decl.format(buffer, state)?;
        }

        state.decrement_indent();

        let close_brace = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::CloseBrace)
            .expect("Input Section should have a close brace");
        format_preceding_comments(&close_brace, buffer, state, false)?;
        state.indent(buffer)?;
        buffer.push_str(&close_brace.to_string());
        format_inline_comment(&self.syntax_element(), buffer, state, false)?;

        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Node(self.syntax().clone())
    }
}

impl Formattable for OutputSection {
    fn format(&self, buffer: &mut String, state: &mut State) -> Result<()> {
        format_preceding_comments(&self.syntax_element(), buffer, state, false)?;

        let output_keyword = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::OutputKeyword)
            .expect("Output Section should have an output keyword");
        state.indent(buffer)?;
        buffer.push_str("output");
        format_inline_comment(&output_keyword, buffer, state, true)?;

        let open_brace = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::OpenBrace)
            .expect("Output Section should have an open brace");
        format_preceding_comments(&open_brace, buffer, state, true)?;
        // Open braces should ignore the "+1 rule" followed by other interrupted
        // elements.
        if state.interrupted() {
            state.reset_interrupted();
            state.indent(buffer)?;
        } else {
            buffer.push_str(SPACE);
        }
        buffer.push_str(&open_brace.to_string());
        format_inline_comment(&open_brace, buffer, state, false)?;

        state.increment_indent();

        for decl in self.declarations() {
            Decl::Bound(decl).format(buffer, state)?;
        }

        state.decrement_indent();

        let close_brace = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::CloseBrace)
            .expect("Output Section should have a close brace");
        format_preceding_comments(&close_brace, buffer, state, false)?;
        state.indent(buffer)?;
        buffer.push_str(&close_brace.to_string());
        format_inline_comment(&self.syntax_element(), buffer, state, false)?;

        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Node(self.syntax().clone())
    }
}

impl Formattable for HintsItem {
    fn format(&self, buffer: &mut String, state: &mut State) -> Result<()> {
        format_preceding_comments(&self.syntax_element(), buffer, state, false)?;

        let name = self.name();
        state.indent(buffer)?;
        name.format(buffer, state)?;
        format_inline_comment(&name.syntax_element(), buffer, state, true)?;

        let colon = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::Colon)
            .expect("Hints Item should have a colon");
        format_preceding_comments(&colon, buffer, state, true)?;
        if state.interrupted() {
            state.indent(buffer)?;
        }
        buffer.push_str(&colon.to_string());
        format_inline_comment(&colon, buffer, state, true)?;

        let expr = self.expr();
        format_preceding_comments(&expr.syntax_element(), buffer, state, true)?;
        state.space_or_indent(buffer)?;
        expr.format(buffer, state)?;
        format_inline_comment(&self.syntax_element(), buffer, state, false)?;

        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Node(self.syntax().clone())
    }
}

impl Formattable for HintsSection {
    fn format(&self, buffer: &mut String, state: &mut State) -> Result<()> {
        format_preceding_comments(&self.syntax_element(), buffer, state, false)?;

        let hints_keyword = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::HintsKeyword)
            .expect("Hints Section should have a hints keyword");
        state.indent(buffer)?;
        buffer.push_str(&hints_keyword.to_string());
        format_inline_comment(&hints_keyword, buffer, state, true)?;

        let open_brace = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::OpenBrace)
            .expect("Hints Section should have an open brace");
        format_preceding_comments(&open_brace, buffer, state, true)?;
        // Open braces should ignore the "+1 rule" followed by other interrupted
        // elements.
        if state.interrupted() {
            state.reset_interrupted();
            state.indent(buffer)?;
        } else {
            buffer.push_str(SPACE);
        }
        buffer.push_str(&open_brace.to_string());
        format_inline_comment(&open_brace, buffer, state, false)?;

        state.increment_indent();

        for item in self.items() {
            item.format(buffer, state)?;
        }

        state.decrement_indent();

        let close_brace = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::CloseBrace)
            .expect("Hints Section should have a close brace");
        format_preceding_comments(&close_brace, buffer, state, false)?;
        state.indent(buffer)?;
        buffer.push_str(&close_brace.to_string());
        format_inline_comment(&self.syntax_element(), buffer, state, false)?;

        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Node(self.syntax().clone())
    }
}

impl Formattable for StructDefinition {
    fn format(&self, buffer: &mut String, state: &mut State) -> Result<()> {
        format_preceding_comments(&self.syntax_element(), buffer, state, false)?;

        let struct_keyword = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::StructKeyword)
            .expect("Struct Definition should have a struct keyword");
        buffer.push_str(&struct_keyword.to_string());
        format_inline_comment(&struct_keyword, buffer, state, true)?;

        let name = self.name();
        format_preceding_comments(&name.syntax_element(), buffer, state, true)?;
        state.space_or_indent(buffer)?;
        name.format(buffer, state)?;
        format_inline_comment(&name.syntax_element(), buffer, state, true)?;

        let open_brace = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::OpenBrace)
            .expect("Struct Definition should have an open brace");
        // Open braces should ignore the "+1 rule" followed by other interrupted
        // elements.
        if state.interrupted() {
            state.reset_interrupted();
            state.indent(buffer)?;
        } else {
            buffer.push_str(SPACE);
        }
        buffer.push_str(&open_brace.to_string());
        format_inline_comment(&open_brace, buffer, state, false)?;

        state.increment_indent();

        if let Some(m) = self.metadata().next() {
            m.format(buffer, state)?;
            buffer.push_str(NEWLINE);
        }

        if let Some(pm) = self.parameter_metadata().next() {
            pm.format(buffer, state)?;
            buffer.push_str(NEWLINE);
        }

        for decl in self.members() {
            Decl::Unbound(decl).format(buffer, state)?;
        }

        state.decrement_indent();

        let close_brace = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::CloseBrace)
            .expect("Struct Definition should have a close brace");
        format_preceding_comments(&close_brace, buffer, state, false)?;
        state.indent(buffer)?;
        buffer.push_str(&close_brace.to_string());
        format_inline_comment(&self.syntax_element(), buffer, state, false)?;

        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Node(self.syntax().clone())
    }
}

impl Formattable for DocumentItem {
    fn format(&self, buffer: &mut String, state: &mut State) -> Result<()> {
        match self {
            DocumentItem::Import(_) => {
                unreachable!("Import statements should not be formatted as a DocumentItem")
            }
            DocumentItem::Workflow(workflow) => workflow.format(buffer, state),
            DocumentItem::Task(task) => task.format(buffer, state),
            DocumentItem::Struct(structure) => structure.format(buffer, state),
        }
    }

    fn syntax_element(&self) -> SyntaxElement {
        match self {
            DocumentItem::Import(_) => {
                unreachable!("Import statements should not be formatted as a DocumentItem")
            }
            DocumentItem::Workflow(workflow) => workflow.syntax_element(),
            DocumentItem::Task(task) => task.syntax_element(),
            DocumentItem::Struct(structure) => structure.syntax_element(),
        }
    }
}
