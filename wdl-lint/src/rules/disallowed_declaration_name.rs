//! A lint rule that disallows declaration names with type suffixes.

use std::collections::HashSet;

use wdl_ast::AstToken;
use wdl_ast::Diagnostic;
use wdl_ast::Diagnostics;
use wdl_ast::Document;
use wdl_ast::Span;
use wdl_ast::SupportedVersion;
use wdl_ast::SyntaxElement;
use wdl_ast::SyntaxKind;
use wdl_ast::VisitReason;
use wdl_ast::Visitor;
use wdl_ast::v1::BoundDecl;
use wdl_ast::v1::Decl;
use wdl_ast::v1::PrimitiveTypeKind;
use wdl_ast::v1::Type;
use wdl_ast::v1::UnboundDecl;

use crate::Rule;
use crate::Tag;
use crate::TagSet;

/// A rule that identifies declaration names that include their type names as a
/// suffix.
#[derive(Debug, Default)]
pub struct DisallowedDeclarationNameRule;

/// Create a diagnostic for a declaration identifier that contains its type name
fn decl_identifier_with_type(span: Span, decl_name: &str, type_name: &str) -> Diagnostic {
    Diagnostic::warning(format!(
        "declaration identifier '{decl_name}' ends with type name '{type_name}'",
    ))
    .with_rule("DisallowedDeclarationName")
    .with_highlight(span)
    .with_fix("rename the identifier to not include the type name")
}

impl Rule for DisallowedDeclarationNameRule {
    fn id(&self) -> &'static str {
        "DisallowedDeclarationName"
    }

    fn description(&self) -> &'static str {
        "Disallows declaration names that end with their type name"
    }

    fn explanation(&self) -> &'static str {
        "Declaration names should not include their type as a suffix. This makes the code more \
         verbose and often redundant. For example, use 'counter' instead of 'counter_int' or \
         'is_active' instead of 'is_active_boolean'."
    }

    fn tags(&self) -> TagSet {
        TagSet::new(&[Tag::Style, Tag::Clarity])
    }

    fn exceptable_nodes(&self) -> Option<&'static [SyntaxKind]> {
        Some(&[
            SyntaxKind::VersionStatementNode,
            SyntaxKind::InputSectionNode,
            SyntaxKind::OutputSectionNode,
            SyntaxKind::BoundDeclNode,
            SyntaxKind::UnboundDeclNode,
            SyntaxKind::TaskNode,
            SyntaxKind::WorkflowNode,
        ])
    }
}

impl Visitor for DisallowedDeclarationNameRule {
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

    fn bound_decl(&mut self, state: &mut Self::State, reason: VisitReason, decl: &BoundDecl) {
        if reason == VisitReason::Enter {
            check_decl_name(state, &Decl::Bound(decl.clone()), &self.exceptable_nodes());
        }
    }

    fn unbound_decl(&mut self, state: &mut Self::State, reason: VisitReason, decl: &UnboundDecl) {
        if reason == VisitReason::Enter {
            check_decl_name(
                state,
                &Decl::Unbound(decl.clone()),
                &self.exceptable_nodes(),
            );
        }
    }
}

/// Check declaration name for type suffixes
fn check_decl_name(
    state: &mut Diagnostics,
    decl: &Decl,
    exceptable_nodes: &Option<&'static [SyntaxKind]>,
) {
    // Get the declaration type
    let ty = decl.ty();

    // Extract type names to check based on the type
    let mut type_names = HashSet::new();

    // Handle different type variants
    match ty {
        Type::Ref(_) => return, // Skip type reference types (user-defined structs)
        Type::Primitive(primitive_type) => {
            match primitive_type.kind() {
                // Skip File and String types as they cause too many false positives
                PrimitiveTypeKind::File | PrimitiveTypeKind::String => return,
                PrimitiveTypeKind::Boolean => {
                    // Add the primitive type name
                    type_names.insert(primitive_type.to_string());
                    // Also check for "Bool"
                    type_names.insert("Bool".to_string());
                }
                PrimitiveTypeKind::Integer => {
                    // Integer is shortened to Int in WDL
                    type_names.insert(primitive_type.to_string());
                    // Also check for "Integer" explicitly
                    type_names.insert("Integer".to_string());
                }
                PrimitiveTypeKind::Float => {
                    type_names.insert(primitive_type.to_string());
                }
                PrimitiveTypeKind::Directory => {
                    type_names.insert(primitive_type.to_string());
                    // Also check for "Dir"
                    type_names.insert("Dir".to_string());
                }
            }
        }
        Type::Array(_) => {
            // Add "Array" for the compound type
            type_names.insert("Array".to_string());
        }
        Type::Map(_) => {
            // Add "Map" for the compound type
            type_names.insert("Map".to_string());
        }
        Type::Pair(_) => {
            // Add "Pair" for the compound type
            type_names.insert("Pair".to_string());
        }
        Type::Object(_) => {
            // Add "Object" for the object type
            type_names.insert("Object".to_string());
        }
    }

    let name_str = decl.name().as_str();

    // Check if the declaration name ends with one of the type names
    for type_name in &type_names {
        let type_lower = type_name.to_lowercase();

        // Special handling for short type names (3 characters or less)
        // These require word-based checks to avoid false positives
        if type_lower.len() <= 3 {
            // Split the identifier into words
            let words = split_to_words(name_str);

            // Check if the short type name appears as the last word
            if let Some(last_word) = words.last() {
                if last_word.to_lowercase() == type_lower {
                    state.exceptable_add(
                        decl_identifier_with_type(decl.name().span(), name_str, type_name),
                        SyntaxElement::from(decl.syntax().clone()),
                        exceptable_nodes,
                    );
                    return;
                }
            }
        } else {
            // For longer types, check if the identifier ends with the type name
            if name_str.to_lowercase().ends_with(&type_lower) {
                state.exceptable_add(
                    decl_identifier_with_type(decl.name().span(), name_str, type_name),
                    SyntaxElement::from(decl.syntax().clone()),
                    exceptable_nodes,
                );
                return;
            }
        }
    }
}

/// Split an identifier into words using convert_case
fn split_to_words(identifier: &str) -> Vec<String> {
    // Use convert_case's built-in split functionality with default boundaries
    convert_case::split(&identifier, &convert_case::Boundary::defaults())
        .into_iter()
        .map(|s| s.to_string())
        .collect()
}
