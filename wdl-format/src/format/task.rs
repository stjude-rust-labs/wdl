//! A module for formatting elements in tasks.

use anyhow::Result;
use wdl_ast::v1::CommandPart;
use wdl_ast::v1::CommandSection;
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

use super::comments::format_inline_comment;
use super::comments::format_preceding_comments;
use super::format_state::SPACE;
use super::FormatState;
use super::Formattable;
use super::NEWLINE;

impl Formattable for CommandSection {
    fn format(&self, buffer: &mut String, state: &mut FormatState) -> Result<()> {
        format_preceding_comments(&self.syntax_element(), buffer, state, false)?;

        let command_keyword = self
            .syntax()
            .children_with_tokens()
            .find(|c| c.kind() == SyntaxKind::CommandKeyword)
            .expect("Command section should have a command keyword");
        state.indent(buffer)?;
        buffer.push_str("command");
        format_inline_comment(&command_keyword, buffer, state, true)?;

        if self.is_heredoc() {
            let open_heredoc = self
                .syntax()
                .children_with_tokens()
                .find(|c| c.kind() == SyntaxKind::OpenHeredoc)
                .expect("Command section should have an open heredoc");
            format_preceding_comments(&open_heredoc, buffer, state, true)?;
            // Open braces should ignore the "+1 rule" followed by other interrupted
            // elements.
            if state.interrupted() {
                state.reset_interrupted();
                state.indent(buffer)?;
            } else {
                buffer.push_str(SPACE);
            }
            buffer.push_str("<<<");
        } else {
            let open_brace = self
                .syntax()
                .children_with_tokens()
                .find(|c| c.kind() == SyntaxKind::OpenBrace)
                .expect("Command section should have an open brace");
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
        }

        for part in self.parts() {
            match part {
                CommandPart::Text(t) => {
                    buffer.push_str(t.as_str());
                }
                CommandPart::Placeholder(p) => {
                    buffer.push_str(&p.syntax().to_string()); // TODO format placeholders
                }
            }
        }

        if self.is_heredoc() {
            buffer.push_str(">>>");
        } else {
            buffer.push('}');
        }
        format_inline_comment(&self.syntax_element(), buffer, state, false)?;

        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Node(self.syntax().clone())
    }
}

impl Formattable for RuntimeItem {
    fn format(&self, buffer: &mut String, state: &mut FormatState) -> Result<()> {
        format_preceding_comments(&self.syntax_element(), buffer, state, false)?;

        let name = self.name();
        state.indent(buffer)?;
        name.format(buffer, state)?;
        format_inline_comment(&name.syntax_element(), buffer, state, true)?;

        let colon = self
            .syntax()
            .children_with_tokens()
            .find(|c| c.kind() == SyntaxKind::Colon)
            .expect("Runtime item should have a colon");
        format_preceding_comments(&colon, buffer, state, true)?;
        if state.interrupted() {
            // TODO: does this need a reset_interrupted?
            state.indent(buffer)?;
        }
        buffer.push(':');
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

impl Formattable for RuntimeSection {
    fn format(&self, buffer: &mut String, state: &mut FormatState) -> Result<()> {
        format_preceding_comments(&self.syntax_element(), buffer, state, false)?;

        let runtime_keyword = self
            .syntax()
            .children_with_tokens()
            .find(|c| c.kind() == SyntaxKind::RuntimeKeyword)
            .expect("Runtime section should have a runtime keyword");
        state.indent(buffer)?;
        buffer.push_str("runtime");
        format_inline_comment(&runtime_keyword, buffer, state, true)?;

        let open_brace = self
            .syntax()
            .children_with_tokens()
            .find(|c| c.kind() == SyntaxKind::OpenBrace)
            .expect("Runtime section should have an open brace");
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

        for item in self.items() {
            item.format(buffer, state)?;
        }

        state.decrement_indent();

        let close_brace = self
            .syntax()
            .children_with_tokens()
            .find(|c| c.kind() == SyntaxKind::CloseBrace)
            .expect("Runtime section should have a close brace");
        format_preceding_comments(&close_brace, buffer, state, true)?;
        state.indent(buffer)?;
        buffer.push('}');
        format_inline_comment(&self.syntax_element(), buffer, state, false)?;

        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Node(self.syntax().clone())
    }
}

impl Formattable for RequirementsItem {
    fn format(&self, buffer: &mut String, state: &mut FormatState) -> Result<()> {
        format_preceding_comments(&self.syntax_element(), buffer, state, false)?;

        let name = self.name();
        state.indent(buffer)?;
        name.format(buffer, state)?;
        format_inline_comment(&name.syntax_element(), buffer, state, true)?;

        let colon = self
            .syntax()
            .children_with_tokens()
            .find(|c| c.kind() == SyntaxKind::Colon)
            .expect("Requirements item should have a colon");
        format_preceding_comments(&colon, buffer, state, true)?;
        if state.interrupted() {
            // TODO does this need a reset_interrupted?
            state.indent(buffer)?;
        }
        buffer.push(':');
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

impl Formattable for RequirementsSection {
    fn format(&self, buffer: &mut String, state: &mut FormatState) -> Result<()> {
        format_preceding_comments(&self.syntax_element(), buffer, state, false)?;

        let requirements_keyword = self
            .syntax()
            .children_with_tokens()
            .find(|c| c.kind() == SyntaxKind::RequirementsKeyword)
            .expect("Requirements section should have a requirements keyword");
        state.indent(buffer)?;
        buffer.push_str("requirements");
        format_inline_comment(&requirements_keyword, buffer, state, true)?;

        let open_brace = self
            .syntax()
            .children_with_tokens()
            .find(|c| c.kind() == SyntaxKind::OpenBrace)
            .expect("Requirements section should have an open brace");
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

        for item in self.items() {
            item.format(buffer, state)?;
        }

        state.decrement_indent();

        let close_brace = self
            .syntax()
            .children_with_tokens()
            .find(|c| c.kind() == SyntaxKind::CloseBrace)
            .expect("Requirements section should have a close brace");
        format_preceding_comments(&close_brace, buffer, state, true)?;
        state.indent(buffer)?;
        buffer.push('}');
        format_inline_comment(&self.syntax_element(), buffer, state, false)?;

        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Node(self.syntax().clone())
    }
}

impl Formattable for TaskDefinition {
    fn format(&self, buffer: &mut String, state: &mut FormatState) -> Result<()> {
        format_preceding_comments(&self.syntax_element(), buffer, state, false)?;

        let task_keyword = self
            .syntax()
            .children_with_tokens()
            .find(|c| c.kind() == SyntaxKind::TaskKeyword)
            .expect("Task should have a task keyword");
        state.indent(buffer)?;
        buffer.push_str("task");
        format_inline_comment(&task_keyword, buffer, state, true)?;

        let name = self.name();
        format_preceding_comments(&name.syntax_element(), buffer, state, true)?;
        state.space_or_indent(buffer)?;
        name.format(buffer, state)?;
        format_inline_comment(&name.syntax_element(), buffer, state, true)?;

        let open_brace = self
            .syntax()
            .children_with_tokens()
            .find(|c| c.kind() == SyntaxKind::OpenBrace)
            .expect("Task should have an open brace");
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
        let mut declaration_section_str = String::new();
        let mut command_section_str = String::new();
        let mut output_section_str = String::new();
        let mut runtime_section_str = String::new();
        let mut hints_section_str = String::new();
        let mut requirements_section_str = String::new();

        for item in self.items() {
            match item {
                TaskItem::Metadata(m) => {
                    m.format(&mut meta_section_str, state)?;
                }
                TaskItem::ParameterMetadata(pm) => {
                    pm.format(&mut parameter_meta_section_str, state)?;
                }
                TaskItem::Input(i) => {
                    i.format(&mut input_section_str, state)?;
                }
                TaskItem::Declaration(d) => {
                    let decl =
                        Decl::cast(d.syntax().clone()).expect("Task declaration should be a Decl");
                    decl.format(&mut declaration_section_str, state)?;
                }
                TaskItem::Command(c) => {
                    c.format(&mut command_section_str, state)?;
                }
                TaskItem::Output(o) => {
                    o.format(&mut output_section_str, state)?;
                }
                TaskItem::Runtime(r) => {
                    r.format(&mut runtime_section_str, state)?;
                }
                TaskItem::Hints(h) => {
                    h.format(&mut hints_section_str, state)?;
                }
                TaskItem::Requirements(r) => {
                    r.format(&mut requirements_section_str, state)?;
                }
            }
        }

        let mut first_section = true;

        if !meta_section_str.is_empty() {
            first_section = false;
            buffer.push_str(&meta_section_str);
        }
        if !parameter_meta_section_str.is_empty() {
            if !first_section {
                buffer.push_str(NEWLINE);
            }
            first_section = false;
            buffer.push_str(&parameter_meta_section_str);
        }
        if !input_section_str.is_empty() {
            if !first_section {
                buffer.push_str(NEWLINE);
            }
            first_section = false;
            buffer.push_str(&input_section_str);
        }
        if !declaration_section_str.is_empty() {
            if !first_section {
                buffer.push_str(NEWLINE);
            }
            first_section = false;
            buffer.push_str(&declaration_section_str);
        }
        // Command section is required
        if !first_section {
            buffer.push_str(NEWLINE);
        }
        buffer.push_str(&command_section_str);
        if !output_section_str.is_empty() {
            buffer.push_str(NEWLINE);
            buffer.push_str(&output_section_str);
        }
        if !runtime_section_str.is_empty() {
            buffer.push_str(NEWLINE);
            buffer.push_str(&runtime_section_str);
        }

        state.decrement_indent();

        let close_brace = self
            .syntax()
            .children_with_tokens()
            .find(|c| c.kind() == SyntaxKind::CloseBrace)
            .expect("Task should have a close brace");
        format_preceding_comments(&close_brace, buffer, state, true)?;
        state.indent(buffer)?;
        buffer.push('}');
        format_inline_comment(&self.syntax_element(), buffer, state, false)?;

        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Node(self.syntax().clone())
    }
}
