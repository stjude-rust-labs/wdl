//! Formatting for imports.

use wdl_ast::SyntaxKind;

use crate::PreToken;
use crate::TokenStream;
use crate::Writable as _;
use crate::element::FormatElement;
use crate::exactly_one;

/// Formats an [`ImportAlias`](wdl_ast::v1::ImportAlias).
pub fn format_import_alias(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children().expect("import alias children");

    let alias_keyword = children.next().expect("alias keyword");
    (&alias_keyword).write(stream);

    stream.end_word();

    let real_name = children.next().expect("ident");
    (&real_name).write(stream);

    stream.end_word();

    let as_keyword = children.next().expect("`as` keyword");
    (&as_keyword).write(stream);

    stream.end_word();

    let alias_name = children.next().expect("ident");
    (&alias_name).write(stream);

    stream.end_word();

    if children.next().is_some() {
        todo!("unhandled children for import alias");
    }
}

/// Formats an [`ImportStatement`](wdl_ast::v1::ImportStatement).
pub fn format_import_statement(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children_by_kind();

    if let Some(mut import_keywords) = children.remove(&SyntaxKind::ImportKeyword) {
        let import_keyword = exactly_one!(import_keywords, "import keywords");
        (&import_keyword).write(stream);
    }

    stream.end_word();

    if let Some(mut string_literals) = children.remove(&SyntaxKind::LiteralStringNode) {
        let string_literal = exactly_one!(string_literals, "string literals");
        (&string_literal).write(stream);
    }

    stream.end_word();

    if let Some(import_aliases) = children.remove(&SyntaxKind::ImportAliasNode) {
        for import_alias in import_aliases {
            (&import_alias).write(stream);
        }
    }

    stream.end_line();

    if !children.is_empty() {
        todo!("unhandled children for import: {:#?}", children.keys());
    }
}
