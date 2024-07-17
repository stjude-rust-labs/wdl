//! A library for auto-formatting WDL code.

#![warn(missing_docs)]
#![warn(rust_2018_idioms)]
#![warn(rust_2021_compatibility)]
#![warn(missing_debug_implementations)]
#![warn(clippy::missing_docs_in_private_items)]
#![warn(rustdoc::broken_intra_doc_links)]

use anyhow::Result;
use wdl_ast::token_child;
use wdl_ast::v1::VersionKeyword;
use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::Diagnostic;
use wdl_ast::Document;
use wdl_ast::Ident;
use wdl_ast::SyntaxElement;
use wdl_ast::SyntaxKind;
use wdl_ast::SyntaxNode;
use wdl_ast::Validator;
use wdl_ast::Version;
use wdl_ast::VersionStatement;
use wdl_grammar::SyntaxExt;

mod formatter;
mod import;
// mod metadata;
mod task;
mod v1;
mod workflow;

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

impl Formattable for VersionKeyword {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        _formatter: &mut Formatter,
    ) -> std::fmt::Result {
        write!(writer, "{}", self.as_str())
    }
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
        let mut preamble_comments = Vec::new();
        let mut lint_directives = Vec::new();

        for comment in self.syntax().preceding_trivia() {
            if comment.starts_with(LINT_DIRECTIVE_PREFIX) {
                lint_directives.push(comment);
            } else if comment.starts_with('#') {
                preamble_comments.push(comment);
            } // else is just a newline
        }

        for comment in preamble_comments.iter() {
            write!(writer, "{}{}", comment, NEWLINE)?;
        }

        // If there are preamble comments, ensure a blank line is inserted
        if !preamble_comments.is_empty() {
            write!(writer, "{}", NEWLINE)?;
        }

        for comment in lint_directives.iter() {
            write!(writer, "{}{}", comment, NEWLINE)?;
        }

        let version_keyword = self.keyword();
        version_keyword.format(writer, formatter)?;
        formatter.format_inline_comment(writer, version_keyword.syntax().inline_comment(), true)?;

        let version = self.version();
        formatter.format_preceding_trivia(
            writer,
            version.syntax().preceding_trivia(),
            true,
            false,
        )?;
        formatter.space_or_indent(writer)?;
        version.format(writer, formatter)?;
        formatter.format_inline_comment(writer, self.syntax().inline_comment(), false)
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
            // write!(writer, "{}", NEWLINE)?;
            // item.format(writer, formatter)?;
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
