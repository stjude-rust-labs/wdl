//! A module for formatting elements in tasks.

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
use super::state::SPACE;
use super::Formattable;
use super::State;
use super::NEWLINE;

impl Formattable for CommandSection {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, state: &mut State) -> std::fmt::Result {
        format_preceding_comments(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            state,
            false,
        )?;

        let command_keyword = self
            .syntax()
            .children_with_tokens()
            .find(|c| c.kind() == SyntaxKind::CommandKeyword)
            .expect("Command section should have a command keyword");
        state.indent(writer)?;
        write!(writer, "{}", command_keyword)?;
        format_inline_comment(&command_keyword, writer, state, true)?;

        if self.is_heredoc() {
            let open_heredoc = self
                .syntax()
                .children_with_tokens()
                .find(|c| c.kind() == SyntaxKind::OpenHeredoc)
                .expect("Command section should have an open heredoc");
            format_preceding_comments(&open_heredoc, writer, state, true)?;
            // Open braces should ignore the "+1 rule" followed by other interrupted
            // elements.
            if state.interrupted() {
                state.reset_interrupted();
                state.indent(writer)?;
            } else {
                write!(writer, "{}", SPACE)?;
            }
            write!(writer, "{}", open_heredoc)?;
        } else {
            let open_brace = self
                .syntax()
                .children_with_tokens()
                .find(|c| c.kind() == SyntaxKind::OpenBrace)
                .expect("Command section should have an open brace");
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
        }

        for part in self.parts() {
            match part {
                CommandPart::Text(t) => {
                    write!(writer, "{}", t.as_str())?;
                }
                CommandPart::Placeholder(p) => {
                    write!(writer, "{}", p.syntax())?;
                }
            }
        }

        if self.is_heredoc() {
            write!(writer, ">>>")?;
        } else {
            write!(writer, "}}")?;
        }
        format_inline_comment(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            state,
            false,
        )
    }
}

impl Formattable for RuntimeItem {
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

        let colon = self
            .syntax()
            .children_with_tokens()
            .find(|c| c.kind() == SyntaxKind::Colon)
            .expect("Runtime item should have a colon");
        format_preceding_comments(&colon, writer, state, true)?;
        if state.interrupted() {
            state.reset_interrupted();
            state.indent(writer)?;
        }
        write!(writer, "{}", colon)?;
        format_inline_comment(&colon, writer, state, true)?;

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
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            state,
            false,
        )
    }
}

impl Formattable for RuntimeSection {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, state: &mut State) -> std::fmt::Result {
        format_preceding_comments(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            state,
            false,
        )?;

        let runtime_keyword = self
            .syntax()
            .children_with_tokens()
            .find(|c| c.kind() == SyntaxKind::RuntimeKeyword)
            .expect("Runtime section should have a runtime keyword");
        state.indent(writer)?;
        write!(writer, "{}", runtime_keyword)?;
        format_inline_comment(&runtime_keyword, writer, state, true)?;

        let open_brace = self
            .syntax()
            .children_with_tokens()
            .find(|c| c.kind() == SyntaxKind::OpenBrace)
            .expect("Runtime section should have an open brace");
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
        }

        state.decrement_indent();

        let close_brace = self
            .syntax()
            .children_with_tokens()
            .find(|c| c.kind() == SyntaxKind::CloseBrace)
            .expect("Runtime section should have a close brace");
        format_preceding_comments(&close_brace, writer, state, true)?;
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

impl Formattable for RequirementsItem {
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

        let colon = self
            .syntax()
            .children_with_tokens()
            .find(|c| c.kind() == SyntaxKind::Colon)
            .expect("Requirements item should have a colon");
        format_preceding_comments(&colon, writer, state, true)?;
        if state.interrupted() {
            state.reset_interrupted();
            state.indent(writer)?;
        }
        write!(writer, "{}", colon)?;
        format_inline_comment(&colon, writer, state, true)?;

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
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            state,
            false,
        )
    }
}

impl Formattable for RequirementsSection {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, state: &mut State) -> std::fmt::Result {
        format_preceding_comments(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            state,
            false,
        )?;

        let requirements_keyword = self
            .syntax()
            .children_with_tokens()
            .find(|c| c.kind() == SyntaxKind::RequirementsKeyword)
            .expect("Requirements section should have a requirements keyword");
        state.indent(writer)?;
        write!(writer, "{}", requirements_keyword)?;
        format_inline_comment(&requirements_keyword, writer, state, true)?;

        let open_brace = self
            .syntax()
            .children_with_tokens()
            .find(|c| c.kind() == SyntaxKind::OpenBrace)
            .expect("Requirements section should have an open brace");
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
        }

        state.decrement_indent();

        let close_brace = self
            .syntax()
            .children_with_tokens()
            .find(|c| c.kind() == SyntaxKind::CloseBrace)
            .expect("Requirements section should have a close brace");
        format_preceding_comments(&close_brace, writer, state, true)?;
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

impl Formattable for TaskDefinition {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, state: &mut State) -> std::fmt::Result {
        format_preceding_comments(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            state,
            false,
        )?;

        let task_keyword = self
            .syntax()
            .children_with_tokens()
            .find(|c| c.kind() == SyntaxKind::TaskKeyword)
            .expect("Task should have a task keyword");
        state.indent(writer)?;
        write!(writer, "{}", task_keyword)?;
        format_inline_comment(&task_keyword, writer, state, true)?;

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
            .find(|c| c.kind() == SyntaxKind::OpenBrace)
            .expect("Task should have an open brace");
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
                    Decl::Bound(d).format(&mut declaration_section_str, state)?;
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
            write!(writer, "{}", meta_section_str)?;
        }
        if !parameter_meta_section_str.is_empty() {
            if !first_section {
                write!(writer, "{}", NEWLINE)?;
            }
            first_section = false;
            write!(writer, "{}", parameter_meta_section_str)?;
        }
        if !input_section_str.is_empty() {
            if !first_section {
                write!(writer, "{}", NEWLINE)?;
            }
            first_section = false;
            write!(writer, "{}", input_section_str)?;
        }
        if !declaration_section_str.is_empty() {
            if !first_section {
                write!(writer, "{}", NEWLINE)?;
            }
            first_section = false;
            write!(writer, "{}", declaration_section_str)?;
        }
        // Command section is required
        if !first_section {
            write!(writer, "{}", NEWLINE)?;
        }
        write!(writer, "{}", command_section_str)?;
        if !output_section_str.is_empty() {
            write!(writer, "{}", NEWLINE)?;
            write!(writer, "{}", output_section_str)?;
        }
        if !runtime_section_str.is_empty() {
            write!(writer, "{}", NEWLINE)?;
            write!(writer, "{}", runtime_section_str)?;
        }
        if !hints_section_str.is_empty() {
            write!(writer, "{}", NEWLINE)?;
            write!(writer, "{}", hints_section_str)?;
        }
        if !requirements_section_str.is_empty() {
            write!(writer, "{}", NEWLINE)?;
            write!(writer, "{}", requirements_section_str)?;
        }

        state.decrement_indent();

        let close_brace = self
            .syntax()
            .children_with_tokens()
            .find(|c| c.kind() == SyntaxKind::CloseBrace)
            .expect("Task should have a close brace");
        format_preceding_comments(&close_brace, writer, state, true)?;
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
