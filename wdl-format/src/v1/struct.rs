//! Formatting for structs.

use wdl_ast::SyntaxKind;

use crate::PreToken;
use crate::TokenStream;
use crate::Writable as _;
use crate::element::FormatElement;

/// Formats a [`StructDefinition`](wdl_ast::v1::StructDefinition).
pub fn format_struct_definition(element: &FormatElement, stream: &mut TokenStream<PreToken>) {
    let mut children = element.children().expect("struct definition children");

    let struct_keyword = children.next().expect("struct keyword");
    assert!(struct_keyword.element().kind() == SyntaxKind::StructKeyword);
    (&struct_keyword).write(stream);
    stream.end_word();

    let name = children.next().expect("struct name");
    assert!(name.element().kind() == SyntaxKind::Ident);
    (&name).write(stream);
    stream.end_word();

    let open_brace = children.next().expect("open brace");
    assert!(open_brace.element().kind() == SyntaxKind::OpenBrace);
    (&open_brace).write(stream);
    stream.end_line();
    stream.increment_indent();

    let mut meta = None;
    let mut parameter_meta = None;
    let mut members = Vec::new();
    let mut close_brace = None;

    for child in children {
        match child.element().kind() {
            SyntaxKind::MetadataSectionNode => {
                meta = Some(child.clone());
            }
            SyntaxKind::ParameterMetadataSectionNode => {
                parameter_meta = Some(child.clone());
            }
            SyntaxKind::UnboundDeclNode => {
                members.push(child.clone());
            }
            SyntaxKind::CloseBrace => {
                close_brace = Some(child.clone());
            }
            _ => {
                unreachable!(
                    "unexpected child in struct definition: {:?}",
                    child.element().kind()
                );
            }
        }
    }

    if let Some(meta) = meta {
        (&meta).write(stream);
        stream.blank_line();
    }

    if let Some(parameter_meta) = parameter_meta {
        (&parameter_meta).write(stream);
        stream.blank_line();
    }

    for member in members {
        (&member).write(stream);
    }

    stream.decrement_indent();
    (&close_brace.expect("struct definition close brace")).write(stream);
    stream.end_line();
}
