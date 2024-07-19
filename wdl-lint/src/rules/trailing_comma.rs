//! A lint rule for trailing commas in lists/objects.

use wdl_ast::v1::MetadataArray;
use wdl_ast::AstNode;
use wdl_ast::Diagnostic;
use wdl_ast::Diagnostics;
use wdl_ast::Document;
use wdl_ast::Span;
use wdl_ast::SupportedVersion;
use wdl_ast::ToSpan;
use wdl_ast::VisitReason;
use wdl_ast::Visitor;

use crate::Rule;
use crate::Tag;
use crate::TagSet;

/// The identifier for the trailing comma rule.
const ID: &str = "TrailingComma";

/// Diagnostic message for missing trailing comma.
fn missing_trailing_comma(span: Span) -> Diagnostic {
    Diagnostic::note("item missing trailing comma")
        .with_rule(ID)
        .with_highlight(span)
        .with_fix("add a comma after this element")
}

/// Detects missing trailing commas.
#[derive(Default, Debug, Clone, Copy)]
pub struct TrailingCommaRule;

impl Rule for TrailingCommaRule {
    fn id(&self) -> &'static str {
        ID
    }

    fn description(&self) -> &'static str {
        "Ensures that lists and objects have a trailing comma."
    }

    fn explanation(&self) -> &'static str {
        "All items in a comma-delimited object or list should be followed by a comma, including \
         the last item. An exception is made for lists for which all items are on the same line, \
         in which case there should not be a trailing comma following the last item. Note that \
         single-line lists are not allowed in the `meta` or `parameter_meta` sections. See rule \
         `key_value_pairs` for more information."
    }

    fn tags(&self) -> TagSet {
        TagSet::new(&[Tag::Style])
    }
}

impl Visitor for TrailingCommaRule {
    type State = Diagnostics;

    fn document(
        &mut self,
        _: &mut Self::State,
        reason: VisitReason,
        _: &Document,
        _: SupportedVersion,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        // Reset the visitor upon document entry
        *self = Default::default();
    }

    fn metadata_object(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        item: &wdl_ast::v1::MetadataObject,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        // Check if object is multi-line
        if item.syntax().to_string().contains('\n') && item.items().count() > 1 {
            let last_child = item.items().last();
            if let Some(last_child) = last_child {
                if let Some(last_child_comma) = last_child.syntax().next_sibling_or_token() {
                    if last_child_comma.kind() != wdl_ast::SyntaxKind::Comma {
                        state.add(missing_trailing_comma(
                            last_child.syntax().text_range().to_span(),
                        ));
                    }
                } else {
                    // no next means no comma
                    state.add(missing_trailing_comma(
                        last_child.syntax().text_range().to_span(),
                    ));
                }
            }
        }
    }

    fn metadata_array(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        item: &MetadataArray,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        // Check if array is multi-line
        if item.syntax().to_string().contains('\n') && item.elements().count() > 1 {
            let last_child = item.elements().last();
            if let Some(last_child) = last_child {
                if let Some(last_child_comma) = last_child.syntax().next_sibling_or_token() {
                    if last_child_comma.kind() != wdl_ast::SyntaxKind::Comma {
                        state.add(missing_trailing_comma(
                            last_child.syntax().text_range().to_span(),
                        ));
                    }
                } else {
                    // no next means no comma
                    state.add(missing_trailing_comma(
                        last_child.syntax().text_range().to_span(),
                    ));
                }
            }
        }
    }
}
