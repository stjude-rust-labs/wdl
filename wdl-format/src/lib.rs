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
mod expr;
mod import;
mod metadata;
mod state;
mod task;
mod v1;
mod workflow;

use comments::format_inline_comment;
use comments::format_preceding_comments;
use state::State;

/// Newline constant used for formatting.
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
            write!(writer, "{}", comment)?;
        }

        // If there are preamble comments, ensure a blank line is inserted
        if !preceding_comments.is_empty() {
            write!(writer, "{}", NEWLINE)?;
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
    let mut state = State::default();

    let version_statement = document
        .version_statement()
        .expect("Document should have a version statement");
    match version_statement.format(&mut result, &mut state) {
        Ok(_) => {}
        Err(_) => {
            return Err(vec![Diagnostic::error(
                "failed to format version statement",
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
                return Err(vec![Diagnostic::error("failed to format import statement")]);
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
                return Err(vec![Diagnostic::error("failed to format document item")]);
            }
        }
    }

    Ok(result)
}
