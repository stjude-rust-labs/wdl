//! Formatting of WDL v1.x elements.

use wdl_ast::AstToken;
use wdl_ast::SyntaxKind;

pub mod import;
pub mod r#struct;
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

/// Formats a [`LiteralBoolean`](wdl_ast::v1::LiteralBoolean).
pub fn format_literal_boolean(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children().expect("literal boolean children");
    let bool = children.next().expect("literal boolean token");
    (&bool).write(stream);
    assert!(children.next().is_none());
}

/// Formats a [`LiteralInteger`](wdl_ast::v1::LiteralInteger).
pub fn format_literal_integer(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    for child in element.children().expect("literal integer children") {
        (&child).write(stream);
    }
}
/// Formats a [`LiteralFloat`](wdl_ast::v1::LiteralFloat).
pub fn format_literal_float(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    for child in element.children().expect("literal float children") {
        (&child).write(stream);
    }
}

/// Formats a [`LiteralNull`](wdl_ast::v1::LiteralNull).
pub fn format_literal_null(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children().expect("literal null children");
    let null = children.next().expect("literal null token");
    (&null).write(stream);
    assert!(children.next().is_none());
}

/// Formats a [`PrimitiveType`](wdl_ast::v1::PrimitiveType).
pub fn format_primitive_type(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children().expect("primitive type children");
    let t = children.next().expect("primitive type token");
    (&t).write(stream);
    assert!(children.next().is_none());
}

/// Formats a [`TypeRef`](wdl_ast::v1::TypeRef).
pub fn format_type_ref(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children().expect("type ref children");
    let t = children.next().expect("type ref type");
    (&t).write(stream);
    assert!(children.next().is_none());
}

/// Formats an [`UnboundDecl`](wdl_ast::v1::UnboundDecl).
pub fn format_unbound_decl(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    for child in element.children().expect("unbound decl children") {
        (&child).write(stream);
        stream.end_word();
    }
    stream.end_line();
}

/// Formats a [`BoundDecl`](wdl_ast::v1::BoundDecl).
pub fn format_bound_decl(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    for child in element.children().expect("bound decl children") {
        (&child).write(stream);
        stream.end_word();
    }
    stream.end_line();
}

/// Formats an [`InputSection`](wdl_ast::v1::InputSection).
pub fn format_input_section(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children().expect("input section children");

    let input_keyword = children.next().expect("input section input keyword");
    assert!(input_keyword.element().kind() == SyntaxKind::InputKeyword);
    (&input_keyword).write(stream);
    stream.end_word();

    let open_brace = children.next().expect("input section open brace");
    assert!(open_brace.element().kind() == SyntaxKind::OpenBrace);
    (&open_brace).write(stream);
    stream.end_line();
    stream.increment_indent();

    let mut close_brace = None;
    let inputs = children
        .filter_map(|child| {
            if child.element().kind() == SyntaxKind::BoundDeclNode {
                Some(child)
            } else {
                assert!(child.element().kind() == SyntaxKind::CloseBrace);
                close_brace = Some(child.clone());
                None
            }
        })
        .collect::<Vec<_>>();

    // TODO: sort inputs
    for input in inputs {
        (&input).write(stream);
    }

    stream.decrement_indent();
    (&close_brace.expect("input section close brace")).write(stream);
    stream.end_line();
}

/// Formats a [`MetadataObject`](wdl_ast::v1::MetadataObject).
pub fn format_metadata_object(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children().expect("metadata object children");

    let open_brace = children.next().expect("metadata object open brace");
    assert!(open_brace.element().kind() == SyntaxKind::OpenBrace);
    (&open_brace).write(stream);
    stream.end_line();
    stream.increment_indent();

    let mut close_brace = None;
    let mut commas = Vec::new();
    let items = children
        .filter(|child| {
            if child.element().kind() == SyntaxKind::MetadataObjectItemNode {
                true
            } else if child.element().kind() == SyntaxKind::Comma {
                commas.push(child.to_owned());
                false
            } else {
                assert!(child.element().kind() == SyntaxKind::CloseBrace);
                close_brace = Some(child.to_owned());
                false
            }
        })
        .collect::<Vec<_>>();

    let mut commas = commas.iter();
    for item in items {
        (&item).write(stream);
        if let Some(comma) = commas.next() {
            (&comma).write(stream);
            stream.end_line();
        } else {
            stream.push_literal(",".to_string(), SyntaxKind::Comma);
            stream.end_line();
        }
    }
}

/// Formats a [`MetadataObjectItem`](wdl_ast::v1::MetadataObjectItem).
pub fn format_metadata_object_item(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children().expect("metadata object item children");

    let key = children.next().expect("metadata object item key");
    assert!(key.element().kind() == SyntaxKind::Ident);
    (&key).write(stream);

    let colon = children.next().expect("metadata object item colon");
    assert!(colon.element().kind() == SyntaxKind::Colon);
    (&colon).write(stream);
    stream.end_word();

    let value = children.next().expect("metadata object item value");
    (&value).write(stream);
    stream.end_line();

    assert!(children.next().is_none());
}

/// Formats a [MetadataSection](wdl_ast::v1::MetadataSection).
pub fn format_metadata_section(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children().expect("meta section children");

    let meta_keyword = children.next().expect("meta keyword");
    assert!(meta_keyword.element().kind() == SyntaxKind::MetaKeyword);
    (&meta_keyword).write(stream);
    stream.end_word();

    let open_brace = children.next().expect("metadata section open brace");
    assert!(open_brace.element().kind() == SyntaxKind::OpenBrace);
    (&open_brace).write(stream);
    stream.end_line();
    stream.increment_indent();

    let mut close_brace = None;
    let metadata = children
        .filter_map(|child| {
            if child.element().kind() == SyntaxKind::MetadataObjectItemNode {
                Some(child)
            } else {
                assert!(child.element().kind() == SyntaxKind::CloseBrace);
                close_brace = Some(child.clone());
                None
            }
        })
        .collect::<Vec<_>>();

    for item in metadata {
        (&item).write(stream);
    }

    stream.decrement_indent();
    (&close_brace.expect("metadata section close brace")).write(stream);
    stream.end_line();
}

/// Formats a [`ParameterMetadataSection`](wdl_ast::v1::ParameterMetadataSection).
pub fn format_parameter_metadata_section(
    element: &FormatElement,
    stream: &mut TokenStream<PreToken>,
) {
    let mut children = element.children().expect("parameter meta section children");

    let parameter_meta_keyword = children.next().expect("parameter meta keyword");
    assert!(parameter_meta_keyword.element().kind() == SyntaxKind::ParameterMetaKeyword);
    (&parameter_meta_keyword).write(stream);
    stream.end_word();

    let open_brace = children.next().expect("parameter metadata section open brace");
    assert!(open_brace.element().kind() == SyntaxKind::OpenBrace);
    (&open_brace).write(stream);
    stream.end_line();
    stream.increment_indent();

    let mut close_brace = None;
    let metadata = children
        .filter_map(|child| {
            if child.element().kind() == SyntaxKind::MetadataObjectItemNode {
                Some(child)
            } else {
                assert!(child.element().kind() == SyntaxKind::CloseBrace);
                close_brace = Some(child.clone());
                None
            }
        })
        .collect::<Vec<_>>();

    for item in metadata {
        (&item).write(stream);
    }

    stream.decrement_indent();
    (&close_brace.expect("parameter metadata section close brace")).write(stream);
    stream.end_line();
}
