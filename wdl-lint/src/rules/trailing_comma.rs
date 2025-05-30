//! A lint rule for trailing commas in lists/objects.

use wdl_analysis::Diagnostics;
use wdl_analysis::VisitReason;
use wdl_analysis::Visitor;
use wdl_ast::AstNode;
use wdl_ast::Diagnostic;
use wdl_ast::Span;
use wdl_ast::SyntaxElement;
use wdl_ast::SyntaxKind;
use wdl_ast::v1::CallStatement;
use wdl_ast::v1::Expr;
use wdl_ast::v1::LiteralExpr;
use wdl_ast::v1::MetadataArray;

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
        .with_fix("add a trailing comma")
}

/// Diagnostic message for extraneous content before trailing comma.
fn extraneous_content(span: Span) -> Diagnostic {
    Diagnostic::note("extraneous whitespace and/or comments before trailing comma")
        .with_rule(ID)
        .with_highlight(span)
        .with_fix("remove the extraneous content before the trailing comma")
}

/// Detects missing trailing commas.
#[derive(Default, Debug, Clone, Copy)]
pub struct TrailingCommaRule;

impl Rule for TrailingCommaRule {
    fn id(&self) -> &'static str {
        ID
    }

    fn description(&self) -> &'static str {
        "Ensures that lists and objects have a trailing comma and that there's not extraneous \
         whitespace and/or comments before the trailing comma."
    }

    fn explanation(&self) -> &'static str {
        "All items in a comma-delimited object or list should be followed by a comma, including \
         the last item. An exception is made for lists for which all items are on the same line, \
         in which case there should not be a trailing comma following the last item. Note that \
         single-line lists are not allowed in the `meta` or `parameter_meta` sections. This method \
         checks `arrays` and `objects` in `meta` and `parameter_meta` sections. It also checks \
         `call` input blocks as well as `Array`, `Map`, `Object`, and `Struct` literals."
    }

    fn tags(&self) -> TagSet {
        TagSet::new(&[Tag::Style])
    }

    fn exceptable_nodes(&self) -> Option<&'static [SyntaxKind]> {
        Some(&[
            SyntaxKind::VersionStatementNode,
            SyntaxKind::MetadataSectionNode,
            SyntaxKind::ParameterMetadataSectionNode,
            SyntaxKind::MetadataArrayNode,
            SyntaxKind::MetadataObjectNode,
            SyntaxKind::CallStatementNode,
            SyntaxKind::LiteralStructNode,
            SyntaxKind::LiteralArrayNode,
            SyntaxKind::LiteralMapNode,
            SyntaxKind::LiteralObjectNode,
        ])
    }

    fn related_rules(&self) -> &[&'static str] {
        &[]
    }
}

impl Visitor for TrailingCommaRule {
    fn reset(&mut self) {
        *self = Self;
    }

    fn metadata_object(
        &mut self,
        diagnostics: &mut Diagnostics,
        reason: VisitReason,
        item: &wdl_ast::v1::MetadataObject,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        // Check if object is multi-line
        if item.inner().to_string().contains('\n') && item.items().count() > 1 {
            let last_child = item.items().last();
            if let Some(last_child) = last_child {
                let (next_comma, comma_is_next) = find_next_comma(last_child.inner());
                match next_comma {
                    Some(comma) => {
                        if !comma_is_next {
                            // Comma found, but not next, extraneous trivia
                            diagnostics.exceptable_add(
                                extraneous_content(Span::new(
                                    last_child.inner().text_range().end().into(),
                                    (comma.text_range().start()
                                        - last_child.inner().text_range().end())
                                    .into(),
                                )),
                                SyntaxElement::from(item.inner().clone()),
                                &self.exceptable_nodes(),
                            );
                        }
                    }
                    _ => {
                        // No comma found, report missing
                        diagnostics.exceptable_add(
                            missing_trailing_comma(
                                last_child
                                    .inner()
                                    .last_token()
                                    .expect("object should have tokens")
                                    .text_range()
                                    .into(),
                            ),
                            SyntaxElement::from(item.inner().clone()),
                            &self.exceptable_nodes(),
                        );
                    }
                }
            }
        }
    }

    fn metadata_array(
        &mut self,
        diagnostics: &mut Diagnostics,
        reason: VisitReason,
        item: &MetadataArray,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        // Check if array is multi-line
        if item.inner().to_string().contains('\n') && item.elements().count() > 1 {
            let last_child = item.elements().last();
            if let Some(last_child) = last_child {
                let (next_comma, comma_is_next) = find_next_comma(last_child.inner());
                match next_comma {
                    Some(comma) => {
                        if !comma_is_next {
                            // Comma found, but not next, extraneous trivia
                            diagnostics.exceptable_add(
                                extraneous_content(Span::new(
                                    last_child.inner().text_range().end().into(),
                                    (comma.text_range().start()
                                        - last_child.inner().text_range().end())
                                    .into(),
                                )),
                                SyntaxElement::from(item.inner().clone()),
                                &self.exceptable_nodes(),
                            );
                        }
                    }
                    _ => {
                        // No comma found, report missing
                        diagnostics.exceptable_add(
                            missing_trailing_comma(
                                last_child
                                    .inner()
                                    .last_token()
                                    .expect("array should have tokens")
                                    .text_range()
                                    .into(),
                            ),
                            SyntaxElement::from(item.inner().clone()),
                            &self.exceptable_nodes(),
                        );
                    }
                }
            }
        }
    }

    fn call_statement(
        &mut self,
        diagnostics: &mut Diagnostics,
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
            let (next_comma, comma_is_next) = find_next_comma(input.inner());
            match next_comma {
                Some(nc) => {
                    if !comma_is_next {
                        diagnostics.exceptable_add(
                            extraneous_content(Span::new(
                                input.inner().text_range().end().into(),
                                (nc.text_range().start() - input.inner().text_range().end()).into(),
                            )),
                            SyntaxElement::from(call.inner().clone()),
                            &self.exceptable_nodes(),
                        );
                    }
                }
                _ => {
                    diagnostics.exceptable_add(
                        missing_trailing_comma(
                            input
                                .inner()
                                .last_token()
                                .expect("input should have tokens")
                                .text_range()
                                .into(),
                        ),
                        SyntaxElement::from(call.inner().clone()),
                        &self.exceptable_nodes(),
                    );
                }
            }
        });
    }

    fn expr(&mut self, diagnostics: &mut Diagnostics, reason: VisitReason, expr: &Expr) {
        if reason == VisitReason::Exit {
            return;
        }
        if let Expr::Literal(l) = expr {
            match l {
                // items: map, object, struct
                // elements: array
                LiteralExpr::Array(_)
                | LiteralExpr::Map(_)
                | LiteralExpr::Object(_)
                | LiteralExpr::Struct(_) => {
                    // Check if array is multi-line
                    if l.inner().to_string().contains('\n') && l.inner().children().count() > 1 {
                        let last_child = l.inner().children().last();
                        if let Some(last_child) = last_child {
                            let (next_comma, comma_is_next) = find_next_comma(&last_child);
                            match next_comma {
                                Some(comma) => {
                                    if !comma_is_next {
                                        // Comma found, but not next, extraneous trivia
                                        diagnostics.exceptable_add(
                                            extraneous_content(Span::new(
                                                last_child.text_range().end().into(),
                                                (comma.text_range().start()
                                                    - last_child.text_range().end())
                                                .into(),
                                            )),
                                            SyntaxElement::from(l.inner().clone()),
                                            &self.exceptable_nodes(),
                                        );
                                    }
                                }
                                _ => {
                                    // No comma found, report missing
                                    diagnostics.exceptable_add(
                                        missing_trailing_comma(
                                            last_child
                                                .last_token()
                                                .expect("item should have tokens")
                                                .text_range()
                                                .into(),
                                        ),
                                        SyntaxElement::from(l.inner().clone()),
                                        &self.exceptable_nodes(),
                                    );
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

/// Find the next comma by consuming until we find a comma or a node.
pub(crate) fn find_next_comma(node: &wdl_ast::SyntaxNode) -> (Option<wdl_ast::SyntaxToken>, bool) {
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
