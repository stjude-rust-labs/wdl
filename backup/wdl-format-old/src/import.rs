//! Format import statements.

use wdl_ast::token_child;
use wdl_ast::v1::AliasKeyword;
use wdl_ast::v1::AsKeyword;
use wdl_ast::v1::ImportAlias;
use wdl_ast::v1::ImportKeyword;
use wdl_ast::v1::ImportStatement;
use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::Ident;
use wdl_ast::SyntaxElement;
use wdl_ast::SyntaxKind;
use wdl_grammar::SyntaxExt;

use crate::Formattable;
use crate::Formatter;

impl Formattable for ImportKeyword {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        _formatter: &mut Formatter,
    ) -> std::fmt::Result {
        write!(writer, "{}", self.as_str())
    }
}

impl Formattable for AsKeyword {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        _formatter: &mut Formatter,
    ) -> std::fmt::Result {
        write!(writer, "{}", self.as_str())
    }
}

impl Formattable for AliasKeyword {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        _formatter: &mut Formatter,
    ) -> std::fmt::Result {
        write!(writer, "{}", self.as_str())
    }
}

impl Formattable for ImportAlias {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        formatter: &mut Formatter,
    ) -> std::fmt::Result {
        formatter.format_preceding_trivia(writer, self.syntax().preceding_trivia(), true, false)?;

        let alias_keyword = self.alias_keyword();
        formatter.space_or_indent(writer)?;
        alias_keyword.format(writer, formatter)?;
        formatter.format_inline_comment(writer, alias_keyword.syntax().inline_comment(), true)?;

        let (source, target) = self.names();

        formatter.format_preceding_trivia(
            writer,
            source.syntax().preceding_trivia(),
            true,
            false,
        )?;
        formatter.space_or_indent(writer)?;
        source.format(writer, formatter)?;
        formatter.format_inline_comment(writer, source.syntax().inline_comment(), true)?;

        let as_keyword = self.as_keyword();
        formatter.format_preceding_trivia(
            writer,
            as_keyword.syntax().preceding_trivia(),
            true,
            false,
        )?;
        formatter.space_or_indent(writer)?;
        as_keyword.format(writer, formatter)?;
        formatter.format_inline_comment(writer, as_keyword.syntax().inline_comment(), true)?;

        formatter.format_preceding_trivia(
            writer,
            target.syntax().preceding_trivia(),
            true,
            false,
        )?;
        formatter.space_or_indent(writer)?;
        target.format(writer, formatter)?;

        formatter.format_inline_comment(writer, self.syntax().inline_comment(), true)
    }
}

impl Formattable for ImportStatement {
    fn format<T: std::fmt::Write>(
        &self,
        writer: &mut T,
        formatter: &mut Formatter,
    ) -> std::fmt::Result {
        formatter.format_preceding_trivia(
            writer,
            self.syntax().preceding_trivia(),
            false,
            false,
        )?;

        let import_keyword = self.keyword();
        import_keyword.format(writer, formatter)?;
        formatter.format_inline_comment(writer, import_keyword.syntax().inline_comment(), true)?;

        let uri = self.uri();
        formatter.format_preceding_trivia(writer, uri.syntax().preceding_trivia(), true, false)?;
        formatter.space_or_indent(writer)?;
        uri.format(writer, formatter)?;
        formatter.format_inline_comment(writer, uri.syntax().inline_comment(), true)?;

        let as_keyword = token_child::<AsKeyword>(self.syntax());
        if let Some(as_keyword) = as_keyword {
            formatter.format_preceding_trivia(
                writer,
                as_keyword.syntax().preceding_trivia(),
                true,
                false,
            )?;
            formatter.space_or_indent(writer)?;
            as_keyword.format(writer, formatter)?;
            formatter.format_inline_comment(writer, as_keyword.syntax().inline_comment(), true)?;

            let ident = self
                .explicit_namespace()
                .expect("import with as clause should have an explicit namespace");
            formatter.format_preceding_trivia(
                writer,
                ident.syntax().preceding_trivia(),
                true,
                false,
            )?;
            formatter.space_or_indent(writer)?;
            ident.format(writer, formatter)?;
            formatter.format_inline_comment(writer, ident.syntax().inline_comment(), true)?;
        }

        for alias in self.aliases() {
            alias.format(writer, formatter)?;
        }

        formatter.format_inline_comment(writer, self.syntax().inline_comment(), false)
    }
}

/// Sorts import statements by their core components.
///
/// The core components of an import statement are the URI and the namespace.
/// These two elements guarantee a unique import statement.
pub fn sort_imports(a: &ImportStatement, b: &ImportStatement) -> std::cmp::Ordering {
    (
        a.uri()
            .text()
            .expect("import URI cannot have placeholders")
            .as_str(),
        &a.namespace().expect("import namespace should exist").0,
    )
        .cmp(&(
            b.uri()
                .text()
                .expect("import URI cannot have placeholders")
                .as_str(),
            &b.namespace().expect("import namespace should exist").0,
        ))
}
