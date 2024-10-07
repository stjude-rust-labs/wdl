//! Formatting of WDL v1.x elements.

use wdl_ast::AstToken;
use wdl_ast::SyntaxKind;

pub mod import;
pub mod task;
pub mod workflow;

use crate::PreToken;
use crate::TokenStream;
use crate::Writable as _;
use crate::element::FormatElement;

/// Formats an [`Ast`](wdl_ast::Ast).
pub fn format_ast(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children().expect("AST children");

    let version_statement = children.next().expect("version statement");
    assert!(version_statement.element().kind() == SyntaxKind::VersionStatementNode);
    (&version_statement).write(stream);

    stream.blank_line();

    let mut imports = Vec::new();
    let mut remainder = Vec::new();

    for child in children {
        match child.element().kind() {
            SyntaxKind::ImportStatementNode => imports.push(child),
            _ => remainder.push(child),
        }
    }

    imports.sort_by(|a, b| {
        let a = a
            .element()
            .as_node()
            .expect("import statement node")
            .as_import_statement()
            .expect("import statement");
        let b = b
            .element()
            .as_node()
            .expect("import statement node")
            .as_import_statement()
            .expect("import statement");
        let a_uri = a.uri().text().expect("import uri");
        let b_uri = b.uri().text().expect("import uri");
        a_uri.as_str().cmp(b_uri.as_str())
    });

    for import in imports {
        (&import).write(stream);
    }

    stream.blank_line();

    for child in remainder {
        (&child).write(stream);
        stream.blank_line();
    }
}

/// Formats a [`VersionStatement`](wdl_ast::VersionStatement).
pub fn format_version_statement(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    for child in element.children().expect("version statement children") {
        (&child).write(stream);
        stream.end_word();
    }
    stream.end_line();
}

/// Formats a [`LiteralString`](wdl_ast::v1::LiteralString).
pub fn format_literal_string(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    for child in element.children().expect("literal string children") {
        match child.element().kind() {
            SyntaxKind::SingleQuote => {
                stream.push_literal_in_place_of_token(
                    child.element().as_token().expect("token"),
                    "\"".to_owned(),
                );
            }
            _ => {
                (&child).write(stream);
            }
        }
    }
}
