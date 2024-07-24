//! A module for formatting metadata sections (meta and parameter_meta).

use anyhow::Result;
use wdl_ast::v1::LiteralNull;
use wdl_ast::v1::MetadataArray;
use wdl_ast::v1::MetadataObject;
use wdl_ast::v1::MetadataObjectItem;
use wdl_ast::v1::MetadataSection;
use wdl_ast::v1::MetadataValue;
use wdl_ast::v1::ParameterMetadataSection;
use wdl_ast::AstNode;
use wdl_ast::SyntaxElement;
use wdl_ast::SyntaxKind;

use super::comments::format_inline_comment;
use super::comments::format_preceding_comments;
use super::format_state::SPACE;
use super::FormatState;
use super::Formattable;
use super::NEWLINE;

impl Formattable for LiteralNull {
    fn format(&self, buffer: &mut String, _state: &mut FormatState) -> Result<()> {
        buffer.push_str("null");
        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Node(self.syntax().clone())
    }
}

impl Formattable for MetadataObject {
    fn format(&self, buffer: &mut String, state: &mut FormatState) -> Result<()> {
        format_preceding_comments(&self.syntax_element(), buffer, state, false)?;

        let open_brace = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::OpenBrace)
            .expect("Metadata Object should have an open brace");
        format_preceding_comments(&open_brace, buffer, state, true)?;
        // Open braces should ignore the "+1 rule" followed by other interrupted elements.
        if state.interrupted() {
            state.reset_interrupted();
            state.indent(buffer)?;
        }
        buffer.push('{');
        format_inline_comment(&open_brace, buffer, state, false)?;

        state.increment_indent();

        // TODO: Check commas
        for item in self.items() {
            item.format(buffer, state)?;
            buffer.push(',');
            buffer.push_str(NEWLINE);
        }

        state.decrement_indent();

        let close_brace = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::CloseBrace)
            .expect("Metadata Object should have a close brace");
        format_preceding_comments(&close_brace, buffer, state, false)?;
        state.indent(buffer)?;
        buffer.push('}');
        format_inline_comment(&self.syntax_element(), buffer, state, true)?;

        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Node(self.syntax().clone())
    }
}

impl Formattable for MetadataArray {
    fn format(&self, buffer: &mut String, state: &mut FormatState) -> Result<()> {
        format_preceding_comments(&self.syntax_element(), buffer, state, false)?;

        let open_bracket = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::OpenBracket)
            .expect("Metadata Array should have an open bracket");
        format_preceding_comments(&open_bracket, buffer, state, true)?;
        // Open braces should ignore the "+1 rule" followed by other interrupted elements.
        if state.interrupted() {
            state.reset_interrupted();
            state.indent(buffer)?;
        }
        buffer.push('[');
        format_inline_comment(&open_bracket, buffer, state, false)?;

        state.increment_indent();

        // TODO: Check commas
        for item in self.elements() {
            state.indent(buffer)?;
            item.format(buffer, state)?;
            buffer.push(',');
            buffer.push_str(NEWLINE);
        }

        state.decrement_indent();

        let close_bracket = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::CloseBracket)
            .expect("Metadata Array should have a close bracket");
        format_preceding_comments(&close_bracket, buffer, state, false)?;
        state.indent(buffer)?;
        buffer.push(']');
        format_inline_comment(&self.syntax_element(), buffer, state, true)?;

        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Node(self.syntax().clone())
    }
}

impl Formattable for MetadataValue {
    fn format(&self, buffer: &mut String, state: &mut FormatState) -> Result<()> {
        match self {
            MetadataValue::String(s) => s.format(buffer, state),
            MetadataValue::Boolean(b) => b.format(buffer, state),
            MetadataValue::Float(f) => f.format(buffer, state),
            MetadataValue::Integer(i) => i.format(buffer, state),
            MetadataValue::Null(n) => n.format(buffer, state),
            MetadataValue::Object(o) => o.format(buffer, state),
            MetadataValue::Array(a) => a.format(buffer, state),
        }
    }

    fn syntax_element(&self) -> SyntaxElement {
        match self {
            MetadataValue::String(s) => s.syntax_element(),
            MetadataValue::Object(o) => o.syntax_element(),
            MetadataValue::Array(a) => a.syntax_element(),
            MetadataValue::Boolean(b) => b.syntax_element(),
            MetadataValue::Float(f) => f.syntax_element(),
            MetadataValue::Integer(i) => i.syntax_element(),
            MetadataValue::Null(n) => n.syntax_element(),
        }
    }
}

impl Formattable for MetadataObjectItem {
    fn format(&self, buffer: &mut String, state: &mut FormatState) -> Result<()> {
        format_preceding_comments(&self.syntax_element(), buffer, state, false)?;

        let name = self.name();
        state.indent(buffer)?;
        name.format(buffer, state)?;
        format_inline_comment(&name.syntax_element(), buffer, state, true)?;

        let colon = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::Colon)
            .expect("Metadata Object Item should have a colon");
        format_preceding_comments(&colon, buffer, state, true)?;
        if state.interrupted() {
            state.indent(buffer)?;
            state.reset_interrupted();
        }
        buffer.push(':');
        format_inline_comment(&colon, buffer, state, true)?;

        let value = self.value();
        format_preceding_comments(&value.syntax_element(), buffer, state, true)?;
        state.space_or_indent(buffer)?;
        value.format(buffer, state)?;
        format_inline_comment(&self.syntax_element(), buffer, state, true)?;

        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Node(self.syntax().clone())
    }
}

impl Formattable for MetadataSection {
    fn format(&self, buffer: &mut String, state: &mut FormatState) -> Result<()> {
        format_preceding_comments(&self.syntax_element(), buffer, state, false)?;

        let meta_keyword = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::MetaKeyword)
            .expect("Metadata Section should have a meta keyword");
        state.indent(buffer)?;
        buffer.push_str("meta");
        format_inline_comment(&meta_keyword, buffer, state, true)?;

        let open_brace = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::OpenBrace)
            .expect("Metadata Section should have an open brace");
        format_preceding_comments(&open_brace, buffer, state, true)?;
        // Open braces should ignore the "+1 rule" followed by other interrupted elements.
        if state.interrupted() {
            state.reset_interrupted();
            state.indent(buffer)?;
        } else {
            buffer.push_str(SPACE);
        }
        buffer.push('{');
        format_inline_comment(&open_brace, buffer, state, false)?;

        state.increment_indent();

        for item in self.items() {
            item.format(buffer, state)?;
            buffer.push_str(NEWLINE);
        }

        state.decrement_indent();

        let close_brace = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::CloseBrace)
            .expect("Metadata Section should have a close brace");
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

impl Formattable for ParameterMetadataSection {
    fn format(&self, buffer: &mut String, state: &mut FormatState) -> Result<()> {
        format_preceding_comments(&self.syntax_element(), buffer, state, false)?;

        let parameter_meta_keyword = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::ParameterMetaKeyword)
            .expect("Parameter Metadata Section should have a parameter meta keyword");
        state.indent(buffer)?;
        buffer.push_str("parameter_meta");
        format_inline_comment(&parameter_meta_keyword, buffer, state, true)?;

        let open_brace = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::OpenBrace)
            .expect("Parameter Metadata Section should have an open brace");
        format_preceding_comments(&open_brace, buffer, state, true)?;
        // Open braces should ignore the "+1 rule" followed by other interrupted elements.
        if state.interrupted() {
            state.reset_interrupted();
            state.indent(buffer)?;
        } else {
            buffer.push_str(SPACE);
        }
        buffer.push('{');
        format_inline_comment(&open_brace, buffer, state, false)?;

        state.increment_indent();

        for item in self.items() {
            item.format(buffer, state)?;
            buffer.push_str(NEWLINE);
        }

        state.decrement_indent();

        let close_brace = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::CloseBrace)
            .expect("Parameter Metadata Section should have a close brace");
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
