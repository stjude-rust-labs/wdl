//! A library for auto-formatting WDL code.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(missing_debug_implementations)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(rustdoc::broken_intra_doc_links)]

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
mod import;
mod metadata;
mod state;
mod task;
mod v1;
mod workflow;

use comments::format_inline_comment;
use comments::format_preceding_comments;
use state::State;

/// Newline constant used for formatting on windows platforms.
#[cfg(windows)]
pub const NEWLINE: &str = "\r\n";
/// Newline constant used for formatting on non-windows platforms.
#[cfg(not(windows))]
pub const NEWLINE: &str = "\n";
/// String terminator constant used for formatting.
const STRING_TERMINATOR: char = '"';

/// A trait for elements that can be formatted.
pub trait Formattable {
    /// Format the element and write it to the writer.
    fn format<T: std::fmt::Write>(&self, writer: &mut T, state: &mut State) -> std::fmt::Result;
}

impl Formattable for Version {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, _state: &mut State) -> std::fmt::Result {
        write!(writer, "{}", self.as_str())
    }
}

impl Formattable for VersionStatement {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, state: &mut State) -> std::fmt::Result {
        let mut preamble_comments = Vec::new();
        let mut lint_directives = Vec::new();
        let comment_buffer = &mut String::new();
        for sibling in self.syntax().siblings_with_tokens(Direction::Prev).skip(1) {
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
                    if comment_buffer.starts_with("#@") {
                        lint_directives.push(comment_buffer.clone());
                    } else {
                        preamble_comments.push(comment_buffer.clone());
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

        for comment in preamble_comments.iter().rev() {
            write!(writer, "{}", comment)?;
        }

        // If there are preamble comments, ensure a blank line is inserted
        if !preamble_comments.is_empty() {
            write!(writer, "{}", NEWLINE)?;
        }

        for comment in lint_directives.iter().rev() {
            write!(writer, "{}", comment)?;
        }

        let version_keyword = first_child_of_kind(self.syntax(), SyntaxKind::VersionKeyword);
        write!(writer, "{}", version_keyword)?;
        format_inline_comment(&version_keyword, writer, state, true)?;

        let version = self.version();

        format_preceding_comments(
            &SyntaxElement::from(version.syntax().clone()),
            writer,
            state,
            true,
        )?;
        state.space_or_indent(writer)?;
        version.format(writer, state)?;
        format_inline_comment(
            &SyntaxElement::from(self.syntax().clone()),
            writer,
            state,
            false,
        )
    }
}

impl Formattable for Ident {
    fn format<T: std::fmt::Write>(&self, writer: &mut T, _state: &mut State) -> std::fmt::Result {
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
    fn format<T: std::fmt::Write>(&self, writer: &mut T, state: &mut State) -> std::fmt::Result {
        let ast = self.ast();
        let ast = ast.as_v1().expect("Document should be a v1 document");
        let version_statement = self
            .version_statement()
            .expect("Document should have a version statement");
        version_statement.format(writer, state)?;
        let mut imports = ast.imports().collect::<Vec<_>>();
        if !imports.is_empty() {
            write!(writer, "{}", NEWLINE)?;
        }
        imports.sort_by(import::sort_imports);
        for import in imports {
            import.format(writer, state)?;
        }
        for item in ast.items() {
            if item.syntax().kind() == SyntaxKind::ImportStatementNode {
                continue;
            }
            write!(writer, "{}", NEWLINE)?;
            item.format(writer, state)?;
        }
        Ok(())
    }
}

/// A formatter for WDL elements.
#[derive(Debug, Default)]
pub struct Formatter(State);

impl Formatter {
    /// Format an element.
    pub fn format<F: std::fmt::Write, T: Formattable>(
        &mut self,
        element: T,
        writer: &mut F,
    ) -> std::fmt::Result {
        element.format(writer, &mut self.0)
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

    match formatter.format(document, &mut result) {
        Ok(_) => {}
        Err(error) => {
            let msg = format!("Failed to format document: {}", error);
            return Err(vec![Diagnostic::error(msg)]);
        }
    }

    Ok(result)
}
