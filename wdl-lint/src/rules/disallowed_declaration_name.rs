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
         verbose and often redundant. For example, use 'counter' instead of 'counterInt' or \
         'is_active' instead of 'isActiveBoolean'."
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
    let name = decl.name();
    let name_str = name.as_str();

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
                    type_names.insert(primitive_type.to_string());
                }

                PrimitiveTypeKind::Float => {
                    type_names.insert(primitive_type.to_string());
                }

                PrimitiveTypeKind::Directory => {
                    type_names.insert(primitive_type.to_string());
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

    // Check if the declaration name ends with one of the type names
    for type_name in &type_names {
        let type_lower = type_name.to_lowercase();

        // Special handling for Int
        if type_lower == "int" {
            // Split the identifier into words using convert_case
            let words = split_to_words(name_str);

            // Check if "int" appears as the last word
            if let Some(last_word) = words.last() {
                if last_word.to_lowercase() == "int" {
                    state.exceptable_add(
                        decl_identifier_with_type(decl.name().span(), name_str, type_name),
                        SyntaxElement::from(decl.syntax().clone()),
                        exceptable_nodes,
                    );
                    return;
                }
            }
        } else {
            // For other types, check if the identifier ends with the type name
            let name_lower = name_str.to_lowercase();
            if name_lower.ends_with(&type_lower) {
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

#[cfg(test)]
mod tests {
    use wdl_ast::Document;
    use wdl_ast::Validator;

    use super::*;

    #[test]
    fn test_disallowed_declaration_name() {
        let source = r#"
            version 1.0

            workflow test {
                input {
                    Int counterInt
                    Boolean isActiveBoolean
                    Boolean flagBool
                    Float measureFloat
                    Array[Int] valuesArray
                    Map[String, Int] resultsMap
                    Pair[Int, Int] dataPair
                }
            }
        "#;

        let (document, _) = Document::parse(source);
        let mut validator = Validator::default();
        validator.add_visitor(DisallowedDeclarationNameRule::default());

        let result = validator.validate(&document);
        assert!(result.is_err());
        let diagnostics = result.err().unwrap();

        // Should flag all declarations with type suffixes
        assert_eq!(diagnostics.len(), 7);

        // Verify each diagnostic
        let diagnostic_texts: Vec<_> = diagnostics
            .iter()
            .map(|d| d.message().to_string())
            .collect();

        assert!(
            diagnostic_texts
                .iter()
                .any(|msg| msg.contains("'counterInt'") && msg.contains("'Int'"))
        );

        assert!(
            diagnostic_texts
                .iter()
                .any(|msg| msg.contains("'isActiveBoolean'") && msg.contains("'Boolean'"))
        );

        assert!(
            diagnostic_texts
                .iter()
                .any(|msg| msg.contains("'flagBool'") && msg.contains("'Bool'"))
        );

        assert!(
            diagnostic_texts
                .iter()
                .any(|msg| msg.contains("'measureFloat'") && msg.contains("'Float'"))
        );

        assert!(
            diagnostic_texts
                .iter()
                .any(|msg| msg.contains("'valuesArray'") && msg.contains("'Array'"))
        );

        assert!(
            diagnostic_texts
                .iter()
                .any(|msg| msg.contains("'resultsMap'") && msg.contains("'Map'"))
        );

        assert!(
            diagnostic_texts
                .iter()
                .any(|msg| msg.contains("'dataPair'") && msg.contains("'Pair'"))
        );
    }

    #[test]
    fn test_allows_valid_names() {
        let source = r#"
            version 1.0

            workflow test {
                input {
                    # These should be allowed (File and String types)
                    File fileInput
                    String stringValue
                    File input_file
                    String value_string
                    
                    # These should be allowed (no type suffix)
                    Int counter
                    Boolean is_active
                    Float measure
                    Array[Int] values
                    Map[String, Int] results
                    Pair[Int, Int] coordinates
                    
                    # Special Int cases that should be allowed
                    Int checkpoint
                    Int footprint
                    Int fingerprint
                    
                    # Type prefixes (allowed)
                    Int intValue
                    Boolean boolFlag
                    Float floatMeasure
                    
                    # Arrays with pluralized inner type (allowed)
                    Array[Int] integers
                    Array[Float] floats
                }
            }
        "#;

        let (document, _) = Document::parse(source);
        let mut validator = Validator::default();
        validator.add_visitor(DisallowedDeclarationNameRule::default());

        let result = validator.validate(&document);
        assert!(result.is_ok());
    }
}
