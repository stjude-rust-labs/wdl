//! Formatting functions for meta and parameter_meta sections.

use wdl_ast::SyntaxKind;

use crate::PreToken;
use crate::TokenStream;
use crate::Writable as _;
use crate::element::FormatElement;

/// Formats a [`LiteralNull`](wdl_ast::v1::LiteralNull).
pub fn format_literal_null(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children().expect("literal null children");
    let null = children.next().expect("literal null token");
    (&null).write(stream);
    assert!(children.next().is_none());
}

/// Formats a [`MetadataArray`](wdl_ast::v1::MetadataArray).
pub fn format_metadata_array(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children().expect("metadata array children");

    let open_bracket = children.next().expect("metadata array open bracket");
    assert!(open_bracket.element().kind() == SyntaxKind::OpenBracket);
    (&open_bracket).write(stream);
    stream.increment_indent();

    let mut close_bracket = None;
    let mut commas = Vec::new();
    let items = children
        .filter(|child| {
            if child.element().kind() == SyntaxKind::CloseBracket {
                close_bracket = Some(child.to_owned());
                false
            } else if child.element().kind() == SyntaxKind::Comma {
                commas.push(child.to_owned());
                false
            } else {
                true
            }
        })
        .collect::<Vec<_>>();

    let mut commas = commas.iter();
    for item in items {
        (&item).write(stream);
        if let Some(comma) = commas.next() {
            (comma).write(stream);
        } else {
            stream.push_literal(",".to_string(), SyntaxKind::Comma);
        }
        stream.end_line();
    }

    stream.decrement_indent();
    (&close_bracket.expect("metadata array close bracket")).write(stream);
}

/// Formats a [`MetadataObject`](wdl_ast::v1::MetadataObject).
pub fn format_metadata_object(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children().expect("metadata object children");

    let open_brace = children.next().expect("metadata object open brace");
    assert!(open_brace.element().kind() == SyntaxKind::OpenBrace);
    (&open_brace).write(stream);
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
            (comma).write(stream);
        } else {
            stream.push_literal(",".to_string(), SyntaxKind::Comma);
        }
        stream.end_line();
    }

    stream.decrement_indent();
    (&close_brace.expect("metadata object close brace")).write(stream);
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
    stream.increment_indent();

    let mut close_brace = None;
    let items = children
        .filter(|child| {
            if child.element().kind() == SyntaxKind::MetadataObjectItemNode {
                true
            } else {
                assert!(child.element().kind() == SyntaxKind::CloseBrace);
                close_brace = Some(child.to_owned());
                false
            }
        })
        .collect::<Vec<_>>();

    for item in items {
        (&item).write(stream);
        stream.end_line();
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

    let open_brace = children
        .next()
        .expect("parameter metadata section open brace");
    assert!(open_brace.element().kind() == SyntaxKind::OpenBrace);
    (&open_brace).write(stream);
    stream.increment_indent();

    let mut close_brace = None;
    let items = children
        .filter(|child| {
            if child.element().kind() == SyntaxKind::MetadataObjectItemNode {
                true
            } else {
                assert!(child.element().kind() == SyntaxKind::CloseBrace);
                close_brace = Some(child.to_owned());
                false
            }
        })
        .collect::<Vec<_>>();

    for item in items {
        (&item).write(stream);
        stream.end_line();
    }

    stream.decrement_indent();
    (&close_brace.expect("parameter metadata section close brace")).write(stream);
    stream.end_line();
}
