//! A module for formatting WDL code.

use std::fmt::Write;

use anyhow::Error;
use anyhow::Ok;
use anyhow::Result;
use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::Comment;
use wdl_ast::Diagnostic;
use wdl_ast::Direction;
use wdl_ast::Document;
use wdl_ast::SyntaxElement;
use wdl_ast::SyntaxKind;
use wdl_ast::Validator;
use wdl_ast::Version;
use wdl_ast::VersionStatement;

mod comments;

use comments::format_inline_comment;
use comments::format_preceding_comments;

/// Newline constant used for formatting.
pub const NEWLINE: &str = "\n";
/// Space constant used for formatting.
pub const SPACE: &str = " ";
/// Indentation constant used for formatting.
pub const INDENT: &str = "    ";

struct FormatState {
    indent_level: usize,
}

impl Default for FormatState {
    fn default() -> Self {
        FormatState { indent_level: 0 }
    }
}

impl FormatState {
    fn indent(&self, buffer: &mut String) -> Result<(), Error> {
        let indent = INDENT.repeat(self.indent_level);
        write!(buffer, "{}", indent)?;
        Ok(())
    }

    fn indent_extra(&self, buffer: &mut String) -> Result<(), Error> {
        let indent = INDENT.repeat(self.indent_level + 1);
        write!(buffer, "{}", indent)?;
        Ok(())
    }

    fn increment_indent(&mut self) {
        self.indent_level += 1;
    }

    fn decrement_indent(&mut self) {
        self.indent_level = self.indent_level.saturating_sub(1);
    }
}

trait Formattable {
    fn format(&self, buffer: &mut String, state: &mut FormatState) -> Result<(), Error>;
    fn syntax_element(&self) -> SyntaxElement;
}

impl Formattable for Comment {
    fn format(&self, buffer: &mut String, _state: &mut FormatState) -> Result<(), Error> {
        let comment = self.as_str().trim();
        write!(buffer, "{}{}", comment, NEWLINE)?;
        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Token(self.syntax().clone())
    }
}

impl Formattable for Version {
    fn format(&self, buffer: &mut String, _state: &mut FormatState) -> Result<(), Error> {
        write!(buffer, "{}", self.as_str())?;
        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Token(self.syntax().clone())
    }
}

impl Formattable for VersionStatement {
    fn format(&self, buffer: &mut String, state: &mut FormatState) -> Result<(), Error> {
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
        if format_inline_comment(&version_keyword, buffer, state)?
            || format_preceding_comments(&version.syntax_element(), buffer, state)?
        {
            state.indent_extra(buffer)?;
        } else {
            write!(buffer, "{}", SPACE)?;
        }

        version.format(buffer, state)?;
        if !format_inline_comment(&self.syntax_element(), buffer, state)? {
            buffer.push_str(NEWLINE);
        }

        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Node(self.syntax().clone())
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
        std::result::Result::Ok(_) => {}
        Err(_) => {
            return Err(vec![Diagnostic::error(
                "Failed to format version statement",
            )]);
        }
    }

    std::result::Result::Ok(result)
}
