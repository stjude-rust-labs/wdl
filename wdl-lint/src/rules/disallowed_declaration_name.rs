//! A lint rule that disallows declaration names with type suffixes.

use std::collections::HashSet;
use convert_case::{Case, Casing};
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
use wdl_ast::v1::UnboundDecl;

use crate::Rule;
use crate::Tag;
use crate::TagSet;

/// The identifier for the disallowed declaration name rule.
const ID: &str = "DisallowedDeclarationName";

/// Diagnostic for declaration identifiers with type suffixes
fn decl_identifier_with_type(span: Span, decl_name: &str, type_name: &str) -> Diagnostic {
    Diagnostic::warning(
        format!(
            "declaration identifier '{decl_name}' ends with type name '{type_name}'",
        ),
        span,
    )
}

/// A rule that checks for declaration names that contain their type.
#[derive(Debug, Default)]
pub struct DisallowedDeclarationNameRule;

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

impl Rule for DisallowedDeclarationNameRule {
    fn id(&self) -> &'static str {
        ID
    }

    fn description(&self) -> &'static str {
        "Disallows declaration names that end with their type name"
    }

    fn explanation(&self) -> &'static str {
        "Declaration names should not include their type as a suffix. \
        This makes the code more verbose and often redundant. For example, use \
        'counter' instead of 'counterInt' or 'is_active' instead of 'isActiveBoolean'."
    }

    fn tags(&self) -> TagSet {
        TagSet::from_iter([
            Tag::Style,
            Tag::Clarity,
            Tag::Maintainability,
        ])
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
    
    // Skip type reference types (user-defined structs)
    if ty.as_type_ref().is_some() {
        return;
    }
    
    // Skip File and String types as they cause too many false positives
    if let Some(primitive_type) = ty.as_primitive_type() {
        match primitive_type.kind() {
            PrimitiveTypeKind::File | PrimitiveTypeKind::String => return,
            _ => {},
        }
    }
    
    // Extract type names to check
    let mut type_names = HashSet::new();
    
    // Add primitive type names
    if let Some(primitive_type) = ty.as_primitive_type() {
        // Use the type's string representation
        type_names.insert(primitive_type.to_string());
        
        // For Boolean type, also check for "Bool"
        if primitive_type.kind() == PrimitiveTypeKind::Boolean {
            type_names.insert("Bool".to_string());
        }
    }
    
    // Add compound type names
    if let Some(array_type) = ty.as_array_type() {
        // Add "Array" for the compound type
        type_names.insert("Array".to_string());
        
        // Extract inner type names but handle plurals differently
        if let Some(primitive) = array_type.element_type().as_primitive_type() {
            // Skip File and String types
            match primitive.kind() {
                PrimitiveTypeKind::File | PrimitiveTypeKind::String => {},
                _ => {
                    // Don't check for inner primitive type suffixes in arrays
                    // since the plural form is often appropriate
                    // (e.g., Array[Int] integers is fine)
                }
            }
        }
    } else if let Some(map_type) = ty.as_map_type() {
        type_names.insert("Map".to_string());
        
        // Skip checking inner types of maps
    } else if let Some(pair_type) = ty.as_pair_type() {
        type_names.insert("Pair".to_string());
        
        // Skip checking inner types of pairs
    }
    
    // Check if the declaration name ends with one of the type names
    for type_name in &type_names {
        let type_lower = type_name.to_lowercase();
        
        // Skip if the type name is too short
        if type_lower.len() < 3 {
            continue;
        }
        
        // Special handling for Int
        if type_lower == "int" {
            // Split the identifier into words to check if "int" is used as a standalone suffix
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
    // Try to detect the case style
    let detected_case = detect_case_style(identifier);
    
    // Split the identifier into words based on the case style
    match detected_case {
        Case::Camel | Case::Pascal => {
            // For camelCase or PascalCase, convert to snake_case first to split words
            identifier.to_case(Case::Snake).split('_')
                .map(|s| s.to_string())
                .collect()
        },
        Case::Snake | Case::Kebab => {
            // For snake_case or kebab-case, split by separator
            let separator = if detected_case == Case::Snake { '_' } else { '-' };
            identifier.split(separator)
                .map(|s| s.to_string())
                .collect()
        },
        _ => {
            // For other cases or mixed cases, just return the identifier as is
            vec![identifier.to_string()]
        }
    }
}

/// Detect the case style of an identifier
fn detect_case_style(identifier: &str) -> Case {
    if identifier.contains('_') {
        Case::Snake
    } else if identifier.contains('-') {
        Case::Kebab
    } else if identifier.chars().next().map_or(false, |c| c.is_uppercase()) {
        Case::Pascal
    } else {
        Case::Camel
    }
}

#[cfg(test)]
mod tests {
    use wdl_ast::{Document, Validator};

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
        
        // We expect 7 errors for the suffixes
        assert_eq!(diagnostics.len(), 7);
        
        // Check that the correct identifiers are flagged
        let diagnostic_texts: Vec<_> = diagnostics
            .iter()
            .map(|d| d.message().to_string())
            .collect();
            
        assert!(diagnostic_texts
            .iter()
            .any(|msg| msg.contains("counterInt") && msg.contains("Int")));
            
        assert!(diagnostic_texts
            .iter()
            .any(|msg| msg.contains("isActiveBoolean") && msg.contains("Boolean")));
            
        assert!(diagnostic_texts
            .iter()
            .any(|msg| msg.contains("flagBool") && msg.contains("Bool")));
            
        assert!(diagnostic_texts
            .iter()
            .any(|msg| msg.contains("measureFloat") && msg.contains("Float")));

        assert!(diagnostic_texts
            .iter()
            .any(|msg| msg.contains("valuesArray") && msg.contains("Array")));

        assert!(diagnostic_texts
            .iter()
            .any(|msg| msg.contains("resultsMap") && msg.contains("Map")));

        assert!(diagnostic_texts
            .iter()
            .any(|msg| msg.contains("dataPair") && msg.contains("Pair")));
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