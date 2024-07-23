//! A module for formatting WDL code.

use std::fmt::Write;

use anyhow::Result;
use wdl_ast::v1::Decl;
use wdl_ast::v1::DocumentItem;
use wdl_ast::v1::Expr;
use wdl_ast::v1::InputSection;
use wdl_ast::v1::LiteralBoolean;
use wdl_ast::v1::LiteralFloat;
use wdl_ast::v1::LiteralInteger;
use wdl_ast::v1::LiteralString;
use wdl_ast::v1::StringPart;
use wdl_ast::v1::Type;
use wdl_ast::v1::WorkflowDefinition;
use wdl_ast::v1::WorkflowItem;
use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::Comment;
use wdl_ast::Diagnostic;
use wdl_ast::Direction;
use wdl_ast::Document;
use wdl_ast::Ident;
use wdl_ast::SyntaxElement;
use wdl_ast::SyntaxKind;
use wdl_ast::Validator;
use wdl_ast::Version;
use wdl_ast::VersionStatement;

mod comments;
mod format_state;
mod import;
mod metadata;

use comments::format_inline_comment;
use comments::format_preceding_comments;
use format_state::FormatState;

/// Newline constant used for formatting.
pub const NEWLINE: &str = "\n";

/// A trait for elements that can be formatted.
pub trait Formattable {
    /// Format the element and write it to the buffer.
    fn format(&self, buffer: &mut String, state: &mut FormatState) -> Result<()>;
    /// Get the syntax element of the element.
    fn syntax_element(&self) -> SyntaxElement;
}

impl Formattable for Version {
    fn format(&self, buffer: &mut String, _state: &mut FormatState) -> Result<()> {
        write!(buffer, "{}", self.as_str())?;
        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Token(self.syntax().clone())
    }
}

impl Formattable for VersionStatement {
    fn format(&self, buffer: &mut String, state: &mut FormatState) -> Result<()> {
        let mut preceding_comments = Vec::new();
        let comment_buffer = &mut String::new();
        for sibling in self.syntax().siblings_with_tokens(Direction::Prev) {
            match sibling.kind() {
                SyntaxKind::Comment => {
                    let comment = Comment::cast(
                        sibling
                            .as_token()
                            .expect("Comment should be a token")
                            .clone(),
                    )
                    .expect("Comment should cast to a comment");
                    comment.format(comment_buffer, state)?;
                    preceding_comments.push(comment_buffer.clone());
                    comment_buffer.clear();
                }
                SyntaxKind::Whitespace => {
                    // Ignore
                }
                SyntaxKind::VersionStatementNode => {
                    // Ignore the root node
                }
                _ => {
                    unreachable!("Unexpected syntax kind: {:?}", sibling.kind());
                }
            }
        }

        for comment in preceding_comments.iter().rev() {
            buffer.push_str(comment);
        }

        // If there are preamble comments, ensure a blank line is inserted
        if !preceding_comments.is_empty() {
            buffer.push_str(NEWLINE);
        }

        buffer.push_str("version");
        let version_keyword = SyntaxElement::Token(
            self.syntax()
                .first_token()
                .expect("Version Statement should have a token")
                .clone(),
        );

        let version = self.version();

        format_inline_comment(&version_keyword, buffer, state, true)?;
        format_preceding_comments(&version.syntax_element(), buffer, state, true)?;
        state.space_or_indent(buffer)?;
        version.format(buffer, state)?;
        format_inline_comment(&self.syntax_element(), buffer, state, false)?;
        state.reset_interrupted();

        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Node(self.syntax().clone())
    }
}

impl Formattable for LiteralString {
    fn format(&self, buffer: &mut String, _state: &mut FormatState) -> Result<()> {
        buffer.push('"');
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
        buffer.push('"');
        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Node(self.syntax().clone())
    }
}

impl Formattable for Ident {
    fn format(&self, buffer: &mut String, _state: &mut FormatState) -> Result<()> {
        write!(buffer, "{}", self.as_str())?;
        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Token(self.syntax().clone())
    }
}

impl Formattable for LiteralBoolean {
    fn format(&self, buffer: &mut String, _state: &mut FormatState) -> Result<()> {
        match self.value() {
            true => buffer.push_str("true"),
            false => buffer.push_str("false"),
        }
        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Node(self.syntax().clone())
    }
}

impl Formattable for LiteralFloat {
    fn format(&self, buffer: &mut String, _state: &mut FormatState) -> Result<()> {
        write!(buffer, "{}", self.syntax().to_string())?;
        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Node(self.syntax().clone())
    }
}

impl Formattable for LiteralInteger {
    fn format(&self, buffer: &mut String, _state: &mut FormatState) -> Result<()> {
        write!(buffer, "{}", self.syntax().to_string())?;
        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Node(self.syntax().clone())
    }
}

impl Formattable for Type {
    fn format(&self, buffer: &mut String, _state: &mut FormatState) -> Result<()> {
        write!(buffer, "{}", self.syntax().to_string())?;
        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Node(self.syntax().clone())
    }
}

impl Formattable for Expr {
    fn format(&self, buffer: &mut String, _state: &mut FormatState) -> Result<()> {
        write!(buffer, "{}", self.syntax().to_string())?;
        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Node(self.syntax().clone())
    }
}

impl Formattable for Decl {
    fn format(&self, buffer: &mut String, state: &mut FormatState) -> Result<()> {
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
            let eq = SyntaxElement::Token(
                self.syntax()
                    .children_with_tokens()
                    .find(|element| element.kind() == SyntaxKind::Assignment)
                    .expect("Bound declaration should have an equals sign")
                    .as_token()
                    .expect("Equals sign should be a token")
                    .clone(),
            );
            format_preceding_comments(&eq, buffer, state, true)?;
            state.space_or_indent(buffer)?;
            buffer.push('=');
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
    fn format(&self, buffer: &mut String, state: &mut FormatState) -> Result<()> {
        format_preceding_comments(&self.syntax_element(), buffer, state, false)?;

        let input_keyword = SyntaxElement::Token(
            self.syntax()
                .first_token()
                .expect("Input Section should have a token")
                .clone(),
        );
        state.indent(buffer)?;
        buffer.push_str("input");
        format_inline_comment(&input_keyword, buffer, state, true)?;

        let open_brace = SyntaxElement::Token(
            self.syntax()
                .children_with_tokens()
                .find(|element| element.kind() == SyntaxKind::OpenBrace)
                .expect("Input Section should have an open brace")
                .as_token()
                .expect("Open brace should be a token")
                .clone(),
        );
        format_preceding_comments(&open_brace, buffer, state, true)?;    
        state.space_or_indent(buffer)?;
        buffer.push('{');
        format_inline_comment(&open_brace, buffer, state, false)?;

        state.increment_indent();

        for decl in self.declarations() {
            decl.format(buffer, state)?;
        }

        state.decrement_indent();

        let close_brace = SyntaxElement::Token(
            self.syntax()
                .children_with_tokens()
                .find(|element| element.kind() == SyntaxKind::CloseBrace)
                .expect("Input Section should have a close brace")
                .as_token()
                .expect("Close brace should be a token")
                .clone(),
        );
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

impl Formattable for WorkflowDefinition {
    fn format(&self, buffer: &mut String, state: &mut FormatState) -> Result<()> {
        format_preceding_comments(&self.syntax_element(), buffer, state, false)?;

        let workflow_keyword = SyntaxElement::Token(
            self.syntax()
                .first_token()
                .expect("Workflow should have a token")
                .clone(),
        );
        buffer.push_str("workflow");
        format_inline_comment(&workflow_keyword, buffer, state, true)?;

        let name = self.name();
        format_preceding_comments(&name.syntax_element(), buffer, state, true)?;
        state.space_or_indent(buffer)?;
        name.format(buffer, state)?;
        format_inline_comment(&name.syntax_element(), buffer, state, true)?;

        let open_brace = SyntaxElement::Token(
            self.syntax()
                .children_with_tokens()
                .find(|element| element.kind() == SyntaxKind::OpenBrace)
                .expect("Workflow should have an open brace")
                .as_token()
                .expect("Open brace should be a token")
                .clone(),
        );
        format_preceding_comments(&open_brace, buffer, state, true)?;
        if !state.interrupted() {
            buffer.push(' ');
        } else {
            state.reset_interrupted();
        }
        buffer.push('{');
        format_inline_comment(&open_brace, buffer, state, false)?;

        state.increment_indent();

        let mut meta_section_str = String::new();
        let mut parameter_meta_section_str = String::new();
        let mut input_section_str = String::new();
        let mut body_str = String::new();
        let mut output_section_str = String::new();

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
                    // c.format(&mut body_str, state)?;
                }
                WorkflowItem::Conditional(c) => {
                    // c.format(&mut body_str, state)?;
                }
                WorkflowItem::Scatter(s) => {
                    // s.format(&mut body_str, state)?;
                }
                WorkflowItem::Declaration(d) => {
                    // d.format(&mut body_str, state)?;
                }
                WorkflowItem::Output(o) => {
                    // o.format(&mut output_section_str, state)?;
                }
                WorkflowItem::Hints(h) => {
                    // h.format(&mut body_str, state)?;
                }
            }
        }

        if !meta_section_str.is_empty() {
            buffer.push_str(&meta_section_str);
        }
        if !parameter_meta_section_str.is_empty() {
            buffer.push_str(&parameter_meta_section_str);
        }
        if !input_section_str.is_empty() {
            buffer.push_str(&input_section_str);
        }

        state.decrement_indent();

        let close_brace = SyntaxElement::Token(
            self.syntax()
                .children_with_tokens()
                .find(|element| element.kind() == SyntaxKind::CloseBrace)
                .expect("Workflow should have a close brace")
                .as_token()
                .expect("Close brace should be a token")
                .clone(),
        );
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

impl Formattable for DocumentItem {
    fn format(&self, buffer: &mut String, state: &mut FormatState) -> Result<()> {
        match self {
            DocumentItem::Import(_) => {
                // Import statements are handled separately
                Ok(())
            }
            DocumentItem::Workflow(workflow) => workflow.format(buffer, state),
            DocumentItem::Task(task) => Ok(()), // task.format(buffer, state),
            DocumentItem::Struct(structure) => Ok(()), // structure.format(buffer, state),
        }
    }

    fn syntax_element(&self) -> SyntaxElement {
        match self {
            DocumentItem::Import(_) => {
                unreachable!("Import statements should not be formatted as a DocumentItem")
            }
            DocumentItem::Workflow(workflow) => workflow.syntax_element(),
            DocumentItem::Task(task) => todo!(), // task.syntax_element(),
            DocumentItem::Struct(structure) => todo!(), // structure.syntax_element(),
        }
    }
}

/// Format a WDL document.
pub fn format_document(code: &str) -> Result<String, Vec<Diagnostic>> {
    let (document, diagnostics) = Document::parse(code);
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let mut validator = Validator::default();
    match validator.validate(&document) {
        std::result::Result::Ok(_) => {
            // The document is valid, so we can format it.
        }
        Err(diagnostics) => return Err(diagnostics),
    }

    let mut result = String::new();
    let mut state = FormatState::default();

    let version_statement = document
        .version_statement()
        .expect("Document should have a version statement");
    match version_statement.format(&mut result, &mut state) {
        Ok(_) => {}
        Err(_) => {
            return Err(vec![Diagnostic::error(
                "Failed to format version statement",
            )]);
        }
    }

    let ast = document.ast();
    let ast = ast.as_v1().expect("Document should be a v1 document");
    let mut imports = ast.imports().collect::<Vec<_>>();
    if !imports.is_empty() {
        result.push_str(NEWLINE);
    }
    imports.sort_by(import::sort_imports);
    for import in imports {
        match import.format(&mut result, &mut state) {
            Ok(_) => {}
            Err(_) => {
                return Err(vec![Diagnostic::error("Failed to format import statement")]);
            }
        }
    }

    for item in ast.items() {
        match item.format(&mut result, &mut state) {
            Ok(_) => {}
            Err(_) => {
                return Err(vec![Diagnostic::error("Failed to format import statement")]);
            }
        }
    }

    Ok(result)
}
