//! A module for formatting WDL code.

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
mod task;
mod workflow;

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

        let version_keyword = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::VersionKeyword)
            .expect("Version statement should have a version keyword");
        buffer.push_str("version");
        format_inline_comment(&version_keyword, buffer, state, true)?;

        let version = self.version();

        format_preceding_comments(&version.syntax_element(), buffer, state, true)?;
        state.space_or_indent(buffer)?;
        version.format(buffer, state)?;
        format_inline_comment(&self.syntax_element(), buffer, state, false)?;

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
            let eq = self
                .syntax()
                .children_with_tokens()
                .find(|element| element.kind() == SyntaxKind::Assignment)
                .expect("Bound declaration should have an equals sign");
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

        let input_keyword = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::InputKeyword)
            .expect("Input Section should have an input keyword");
        state.indent(buffer)?;
        buffer.push_str("input");
        format_inline_comment(&input_keyword, buffer, state, true)?;

        let open_brace = 
            self.syntax()
                .children_with_tokens()
                .find(|element| element.kind() == SyntaxKind::OpenBrace)
                .expect("Input Section should have an open brace");
        format_preceding_comments(&open_brace, buffer, state, true)?;
        state.space_or_indent(buffer)?;
        buffer.push('{');
        format_inline_comment(&open_brace, buffer, state, false)?;

        state.increment_indent();

        for decl in self.declarations() {
            decl.format(buffer, state)?;
        }

        state.decrement_indent();

        let close_brace = 
            self.syntax()
                .children_with_tokens()
                .find(|element| element.kind() == SyntaxKind::CloseBrace)
                .expect("Input Section should have a close brace");
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

impl Formattable for OutputSection {
    fn format(&self, buffer: &mut String, state: &mut FormatState) -> Result<()> {
        format_preceding_comments(&self.syntax_element(), buffer, state, false)?;

        let output_keyword = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::OutputKeyword)
            .expect("Output Section should have an output keyword");
        state.indent(buffer)?;
        buffer.push_str("output");
        format_inline_comment(&output_keyword, buffer, state, true)?;

        let open_brace = 
            self.syntax()
                .children_with_tokens()
                .find(|element| element.kind() == SyntaxKind::OpenBrace)
                .expect("Output Section should have an open brace");
        format_preceding_comments(&open_brace, buffer, state, true)?;
        state.space_or_indent(buffer)?;
        buffer.push('{');
        format_inline_comment(&open_brace, buffer, state, false)?;

        state.increment_indent();

        for decl in self.declarations() {
            Decl::Bound(decl).format(buffer, state)?;
        }

        state.decrement_indent();

        let close_brace = 
            self.syntax()
                .children_with_tokens()
                .find(|element| element.kind() == SyntaxKind::CloseBrace)
                .expect("Output Section should have a close brace");
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

impl Formattable for HintsItem {
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
            .expect("Hints Item should have a colon");
        format_preceding_comments(&colon, buffer, state, true)?;
        if state.interrupted() {
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

impl Formattable for HintsSection {
    fn format(&self, buffer: &mut String, state: &mut FormatState) -> Result<()> {
        format_preceding_comments(&self.syntax_element(), buffer, state, false)?;

        let hints_keyword = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::HintsKeyword)
            .expect("Hints Section should have a hints keyword");
        state.indent(buffer)?;
        buffer.push_str("hints");
        format_inline_comment(&hints_keyword, buffer, state, true)?;

        let open_brace = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::OpenBrace)
            .expect("Hints Section should have an open brace");
        format_preceding_comments(&open_brace, buffer, state, true)?;
        state.space_or_indent(buffer)?;
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
            .find(|element| element.kind() == SyntaxKind::CloseBrace)
            .expect("Hints Section should have a close brace");
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

impl Formattable for StructDefinition {
    fn format(&self, buffer: &mut String, state: &mut FormatState) -> Result<()> {
        format_preceding_comments(&self.syntax_element(), buffer, state, false)?;

        let struct_keyword = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::StructKeyword)
            .expect("Struct Definition should have a struct keyword");
        buffer.push_str("struct");
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
        state.space_or_indent(buffer)?;
        buffer.push('{');
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
        if item.syntax().kind() == SyntaxKind::ImportStatementNode {
            continue;
        }
        result.push_str(NEWLINE);
        match item.format(&mut result, &mut state) {
            Ok(_) => {}
            Err(_) => {
                return Err(vec![Diagnostic::error("Failed to format import statement")]);
            }
        }
    }

    Ok(result)
}
