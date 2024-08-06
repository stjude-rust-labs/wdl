//! A library for auto-formatting WDL code.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(missing_debug_implementations)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(rustdoc::broken_intra_doc_links)]

use std::collections::VecDeque;

use anyhow::Result;
use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::Comment;
use wdl_ast::Diagnostic;
use wdl_ast::Direction;
use wdl_ast::Document;
use wdl_ast::Ident;
use wdl_ast::SyntaxElement;
use wdl_ast::SyntaxKind;
use wdl_ast::SyntaxNode;
use wdl_ast::Validator;
use wdl_ast::Version;
use wdl_ast::VersionStatement;

mod comments;
mod formatter;
mod import;
mod metadata;
mod task;
mod v1;
mod workflow;

use comments::format_inline_comment;
use comments::format_preceding_comments;
use formatter::Formatter;

/// Newline constant used for formatting on windows platforms.
#[cfg(windows)]
pub const NEWLINE: &str = "\r\n";
/// Newline constant used for formatting on non-windows platforms.
#[cfg(not(windows))]
pub const NEWLINE: &str = "\n";
/// String terminator constant used for formatting.
const STRING_TERMINATOR: char = '"';
/// Lint directive prefix constant used for formatting.
const LINT_DIRECTIVE_PREFIX: &str = "#@";

/// A trait for elements that can be formatted.
pub trait Formattable {
    /// Format the element and write it to the writer.
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        formatter: &mut Formatter,
    ) -> std::fmt::Result;
}

impl Formattable for Version {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        _formatter: &mut Formatter,
    ) -> std::fmt::Result {
        write!(writer, "{}", self.as_str())
    }
}

impl Formattable for VersionStatement {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        formatter: &mut Formatter,
    ) -> std::fmt::Result {
        let mut preamble_comments = VecDeque::new();
        let mut lint_directives = VecDeque::new();
        let comment_buffer = &mut String::new();
        for sibling in self.syntax().siblings_with_tokens(Direction::Prev).skip(1) {
            match sibling.kind() {
                SyntaxKind::Comment => {
                    let comment = Comment::cast(
                        sibling
                            .as_token()
                            .expect("comment should be a token")
                            .clone(),
                    )
                    .expect("comment should cast to a comment");
                    comment.format(comment_buffer, formatter)?;

                    if comment_buffer.starts_with(LINT_DIRECTIVE_PREFIX) {
                        lint_directives.push_front(comment_buffer.clone());
                    } else {
                        preamble_comments.push_front(comment_buffer.clone());
                    }
                    comment_buffer.clear();
                }
                SyntaxKind::Whitespace => {
                    // Ignore
                }
                _ => {
                    unreachable!("Unexpected syntax kind: {:?}", sibling.kind());
                }
            }
        }

        for comment in preamble_comments.iter() {
            write!(writer, "{}", comment)?;
        }

        // If there are preamble comments, ensure a blank line is inserted
        if !preamble_comments.is_empty() {
            write!(writer, "{}", NEWLINE)?;
        }

        for comment in lint_directives.iter() {
            write!(writer, "{}", comment)?;
        }

        let version_keyword = first_child_of_kind(self.syntax(), SyntaxKind::VersionKeyword);
        write!(writer, "{}", version_keyword)?;
        format_inline_comment(&version_keyword, writer, formatter, true)?;

        let version = self.version();

        format_preceding_comments(
            &SyntaxElement::from(version.syntax().clone()),
            writer,
            formatter,
            true,
        )?;
        formatter.space_or_indent(writer)?;
        version.format(writer, formatter)?;
        format_inline_comment(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            formatter,
            false,
        )
    }
}

impl Formattable for Ident {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        _formatter: &mut Formatter,
    ) -> std::fmt::Result {
        write!(writer, "{}", self.as_str())
    }
}

/// Find an expected child element of the specified kind.
///
/// # Panics
/// Panics if the child element is not found.
pub fn first_child_of_kind(node: &SyntaxNode, kind: SyntaxKind) -> SyntaxElement {
    node.children_with_tokens()
        .find(|element| element.kind() == kind)
        .unwrap_or_else(|| panic!("Expected to find a child of kind: {kind:?}"))
}

impl Formattable for Document {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        formatter: &mut Formatter,
    ) -> std::fmt::Result {
        let ast = self.ast();
        let ast = ast.as_v1().expect("document should be a v1 document");
        let version_statement = self
            .version_statement()
            .expect("document should have a version statement");
        version_statement.format(writer, formatter)?;
        let mut imports = ast.imports().collect::<Vec<_>>();
        if !imports.is_empty() {
            write!(writer, "{}", NEWLINE)?;
        }
        imports.sort_by(import::sort_imports);
        for import in imports {
            import.format(writer, formatter)?;
        }
        for item in ast.items() {
            if item.syntax().kind() == SyntaxKind::ImportStatementNode {
                continue;
            }
            write!(writer, "{}", NEWLINE)?;
            item.format(writer, formatter)?;
        }
        Ok(())
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
    let formatter = &mut Formatter::default();

    match formatter.format(&document, &mut result) {
        Ok(_) => {}
        Err(error) => {
            let msg = format!("Failed to format document: {}", error);
            return Err(vec![Diagnostic::error(msg)]);
        }
    }

    Ok(result)
}
