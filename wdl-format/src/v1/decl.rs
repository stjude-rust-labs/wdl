//! Formatting functions for declarations.

use wdl_ast::SyntaxKind;

use crate::PreToken;
use crate::TokenStream;
use crate::Writable as _;
use crate::element::FormatElement;

/// Formats a [`PrimitiveType`](wdl_ast::v1::PrimitiveType).
pub fn format_primitive_type(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    for child in element.children().expect("primitive type children") {
        (&child).write(stream);
    }
}

/// Formats an [`ArrayType`](wdl_ast::v1::ArrayType).
pub fn format_array_type(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    for child in element.children().expect("array type children") {
        (&child).write(stream);
    }
}

/// Formats a [`MapType`](wdl_ast::v1::MapType).
pub fn format_map_type(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    for child in element.children().expect("map type children") {
        (&child).write(stream);
        if child.element().kind() == SyntaxKind::Comma {
            stream.end_word();
        }
    }
}

/// Formats an [`ObjectType`](wdl_ast::v1::ObjectType).
pub fn format_object_type(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    for child in element.children().expect("object type children") {
        (&child).write(stream);
    }
}

/// Formats a [`PairType`](wdl_ast::v1::PairType).
pub fn format_pair_type(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    for child in element.children().expect("pair type children") {
        (&child).write(stream);
        if child.element().kind() == SyntaxKind::Comma {
            stream.end_word();
        }
    }
}

/// Formats a [`TypeRef`](wdl_ast::v1::TypeRef).
pub fn format_type_ref(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    for child in element.children().expect("type ref children") {
        (&child).write(stream);
    }
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
