//! A module for formatting WDL code.

use std::fmt::Write;

use anyhow::Result;
use wdl_ast::v1::DocumentItem;
use wdl_ast::v1::LiteralBoolean;
use wdl_ast::v1::LiteralFloat;
use wdl_ast::v1::LiteralInteger;
use wdl_ast::v1::LiteralNull;
use wdl_ast::v1::LiteralString;
use wdl_ast::v1::MetadataArray;
use wdl_ast::v1::MetadataObject;
use wdl_ast::v1::MetadataObjectItem;
use wdl_ast::v1::MetadataSection;
use wdl_ast::v1::MetadataValue;
use wdl_ast::v1::ParameterMetadataSection;
use wdl_ast::v1::StringPart;
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

impl Formattable for Comment {
    fn format(&self, buffer: &mut String, _state: &mut FormatState) -> Result<()> {
        let comment = self.as_str().trim();
        write!(buffer, "{}{}", comment, NEWLINE)?;
        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Token(self.syntax().clone())
    }
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

// impl Ord for ImportStatement {
//     fn cmp(&self, other: &Self) -> std::cmp::Ordering {
//         self.uri().as_str().cmp(other.uri().as_str())
//     }
// }

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

        let open_brace = SyntaxElement::Token(
            self.syntax()
                .children_with_tokens()
                .find(|element| element.kind() == SyntaxKind::OpenBrace)
                .expect("Metadata Object should have an open brace")
                .as_token()
                .expect("Open brace should be a token")
                .clone(),
        );
        format_preceding_comments(&open_brace, buffer, state, true)?;
        if state.interrupted() {
            state.indent(buffer)?;
            state.reset_interrupted();
        }
        buffer.push('{');
        format_inline_comment(&open_brace, buffer, state, false)?;

        state.increment_indent();

        for item in self.items() {
            // state.indent(buffer)?;
            item.format(buffer, state)?;
            buffer.push(',');
            buffer.push_str(NEWLINE);
        }

        state.decrement_indent();

        let close_brace = SyntaxElement::Token(
            self.syntax()
                .children_with_tokens()
                .find(|element| element.kind() == SyntaxKind::CloseBrace)
                .expect("Metadata Object should have a close brace")
                .as_token()
                .expect("Close brace should be a token")
                .clone(),
        );
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

        let open_bracket = SyntaxElement::Token(
            self.syntax()
                .children_with_tokens()
                .find(|element| element.kind() == SyntaxKind::OpenBracket)
                .expect("Metadata Array should have an open bracket")
                .as_token()
                .expect("Open bracket should be a token")
                .clone(),
        );
        format_preceding_comments(&open_bracket, buffer, state, true)?;
        if state.interrupted() {
            state.indent(buffer)?;
            state.reset_interrupted();
        }
        buffer.push('[');
        format_inline_comment(&open_bracket, buffer, state, false)?;

        state.increment_indent();

        for item in self.elements() {
            state.indent(buffer)?;
            item.format(buffer, state)?;
            buffer.push(',');
            buffer.push_str(NEWLINE);
        }

        state.decrement_indent();

        let close_bracket = SyntaxElement::Token(
            self.syntax()
                .children_with_tokens()
                .find(|element| element.kind() == SyntaxKind::CloseBracket)
                .expect("Metadata Array should have a close bracket")
                .as_token()
                .expect("Close bracket should be a token")
                .clone(),
        );
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

        let colon = SyntaxElement::Token(
            self.syntax()
                .children_with_tokens()
                .find(|element| element.kind() == SyntaxKind::Colon)
                .expect("Metadata Object Item should have a colon")
                .as_token()
                .expect("Colon should be a token")
                .clone(),
        );
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
        format_inline_comment(&self.syntax_element(), buffer, state, false)?;

        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Node(self.syntax().clone())
    }
}

impl Formattable for MetadataSection {
    fn format(&self, buffer: &mut String, state: &mut FormatState) -> Result<()> {
        format_preceding_comments(&self.syntax_element(), buffer, state, false)?;

        let meta_keyword = SyntaxElement::Token(
            self.syntax()
                .first_token()
                .expect("Metadata Section should have a token")
                .clone(),
        );
        state.indent(buffer)?;
        buffer.push_str("meta");
        format_inline_comment(&meta_keyword, buffer, state, true)?;

        let open_brace = SyntaxElement::Token(
            self.syntax()
                .children_with_tokens()
                .find(|element| element.kind() == SyntaxKind::OpenBrace)
                .expect("Metadata Section should have an open brace")
                .as_token()
                .expect("Open brace should be a token")
                .clone(),
        );
        format_preceding_comments(&open_brace, buffer, state, true)?;
        if !state.interrupted() {
            buffer.push(' ');
        } else {
            state.reset_interrupted();
            state.indent(buffer)?;
        }
        buffer.push('{');
        format_inline_comment(&open_brace, buffer, state, false)?;

        state.increment_indent();

        for item in self.items() {
            item.format(buffer, state)?;
        }

        state.decrement_indent();

        let close_brace = SyntaxElement::Token(
            self.syntax()
                .children_with_tokens()
                .find(|element| element.kind() == SyntaxKind::CloseBrace)
                .expect("Metadata Section should have a close brace")
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

impl Formattable for ParameterMetadataSection {
    fn format(&self, buffer: &mut String, state: &mut FormatState) -> Result<()> {
        format_preceding_comments(&self.syntax_element(), buffer, state, false)?;

        let parameter_meta_keyword = SyntaxElement::Token(
            self.syntax()
                .first_token()
                .expect("Parameter Metadata Section should have a token")
                .clone(),
        );
        state.indent(buffer)?;
        buffer.push_str("parameter_meta");
        format_inline_comment(&parameter_meta_keyword, buffer, state, true)?;

        let open_brace = SyntaxElement::Token(
            self.syntax()
                .children_with_tokens()
                .find(|element| element.kind() == SyntaxKind::OpenBrace)
                .expect("Parameter Metadata Section should have an open brace")
                .as_token()
                .expect("Open brace should be a token")
                .clone(),
        );
        format_preceding_comments(&open_brace, buffer, state, true)?;
        if !state.interrupted() {
            buffer.push(' ');
        } else {
            state.reset_interrupted();
            state.indent(buffer)?;
        }
        buffer.push('{');
        format_inline_comment(&open_brace, buffer, state, false)?;

        state.increment_indent();

        for item in self.items() {
            item.format(buffer, state)?;
        }

        state.decrement_indent();

        let close_brace = SyntaxElement::Token(
            self.syntax()
                .children_with_tokens()
                .find(|element| element.kind() == SyntaxKind::CloseBrace)
                .expect("Parameter Metadata Section should have a close brace")
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
                    // i.format(&mut input_section_str, state)?;
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
