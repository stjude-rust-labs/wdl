//! A lint rule for trailing commas in lists/objects.

use wdl_ast::v1::CallStatement;
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

/// Diagnostic message for extraneous content before trailing comma.
fn extraneous_content(span: Span) -> Diagnostic {
    Diagnostic::note("extraneous content before trailing comma")
        .with_rule(ID)
        .with_highlight(span)
        .with_fix("remove this extraneous content")
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
         single-line lists are not allowed in the `meta` or `parameter_meta` sections."
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
                let (next_comma, comma_is_next) = find_next_comma(last_child.syntax());
                if let Some(comma) = next_comma {
                    if !comma_is_next {
                        // Comma found, but not next, extraneous trivia
                        state.add(extraneous_content(Span::new(
                            usize::from(last_child.syntax().text_range().end()),
                            usize::from(
                                comma.text_range().start() - last_child.syntax().text_range().end(),
                            ),
                        )));
                    }
                } else {
                    // No comma found, report missing
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
                let (next_comma, comma_is_next) = find_next_comma(last_child.syntax());
                if let Some(comma) = next_comma {
                    if !comma_is_next {
                        // Comma found, but not next, extraneous trivia
                        state.add(extraneous_content(Span::new(
                            usize::from(last_child.syntax().text_range().end()),
                            usize::from(
                                comma.text_range().start() - last_child.syntax().text_range().end(),
                            ),
                        )));
                    }
                } else {
                    // No comma found, report missing
                    state.add(missing_trailing_comma(
                        last_child.syntax().text_range().to_span(),
                    ));
                }
            }
        }
    }

    fn call_statement(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        call: &CallStatement,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        let inputs = call.inputs().count();

        if inputs < 2 {
            return;
        }

        call.inputs().for_each(|input| {
            // check each input for trailing comma
            let (next_comma, comma_is_next) = find_next_comma(input.syntax());
            if let Some(nc) = next_comma {
                if !comma_is_next {
                    state.add(extraneous_content(Span::new(
                        usize::from(input.syntax().text_range().end()),
                        usize::from(nc.text_range().start() - input.syntax().text_range().end()),
                    )));
                }
            } else {
                state.add(missing_trailing_comma(
                    input.syntax().text_range().to_span(),
                ));
            }
        });
    }
}

/// Find the next comma by consuming until we find a comma or a node.
fn find_next_comma(node: &wdl_ast::SyntaxNode) -> (Option<wdl_ast::SyntaxToken>, bool) {
    let mut next = node.next_sibling_or_token();
    let mut comma_is_next = true;
    while let Some(next_node) = next {
        // If we find a node before a comma, then treat as no comma
        // If we find other tokens, then mark that they precede any potential comma
        if next_node.as_node().is_some() {
            return (None, false);
        } else if next_node.kind() == wdl_ast::SyntaxKind::Comma {
            return (Some(next_node.into_token().unwrap()), comma_is_next);
        } else {
            comma_is_next = false;
        }
        next = next_node.next_sibling_or_token();
    }
    (None, false)
}
