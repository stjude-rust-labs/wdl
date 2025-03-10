//! A lint rule that disallows declaration names with type prefixes or suffixes.

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

/// Diagnostic for declaration identifiers with type prefixes or suffixes
fn decl_identifier_with_type(span: Span, decl_name: &str, type_name: &str) -> Diagnostic {
    Diagnostic::note(format!("declaration identifier '{}' contains type name '{}'", decl_name, type_name))
        .with_rule(ID)
        .with_highlight(span)
        .with_fix("rename the identifier to not include the type name")
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
        "Declaration names should not end with their type"
    }

    fn explanation(&self) -> &'static str {
        "Declaration names should not include their type as a suffix. \
        This makes the code more verbose and often redundant. For example, use \
        'counter' instead of 'counterInt', or 'is_active' instead of 'activeBool'."
    }

    fn tags(&self) -> TagSet {
        TagSet::new(&[Tag::Style, Tag::Clarity])
    }

    fn exceptable_nodes(&self) -> Option<&'static [SyntaxKind]> {
        Some(&[
            SyntaxKind::VersionStatementNode,
            SyntaxKind::BoundDeclNode,
            SyntaxKind::UnboundDeclNode,
            SyntaxKind::InputSectionNode,
            SyntaxKind::OutputSectionNode,
        ])
    }
}

/// Check declaration name for type suffixes only
fn check_decl_name(
    state: &mut Diagnostics,
    decl: &Decl,
    exceptable_nodes: &Option<&'static [SyntaxKind]>,
) {
    let name = decl.name();
    let name_str = name.as_str();
    let name_lower = name_str.to_lowercase();
    
    // Get the declaration type
    let ty = decl.ty();
    
    // Skip type reference types (user-defined structs)
    if ty.as_type_ref().is_some() {
        return;
    }
    
    // Extract type names to check
    let mut type_names = Vec::new();
    
    // Add primitive type if present
    if let Some(primitive_type) = ty.as_primitive_type() {
        // Skip File and String types as they cause too many false positives
        match primitive_type.kind() {
            PrimitiveTypeKind::File | PrimitiveTypeKind::String => {},
            PrimitiveTypeKind::Boolean => {
                // For Boolean, check both "Boolean" and "Bool"
                type_names.push("Boolean".to_string());
                type_names.push("Bool".to_string());
            },
            _ => type_names.push(primitive_type.to_string()),
        }
    }
    
    // Add compound types
    if ty.as_array_type().is_some() {
        type_names.push("Array".to_string());
    } else if ty.as_map_type().is_some() {
        type_names.push("Map".to_string());
    } else if ty.as_pair_type().is_some() {
        type_names.push("Pair".to_string());
    }
    
    // Check each type name against the declaration name (only as a suffix)
    for type_name in &type_names {
        let type_lower = type_name.to_lowercase();
        
        // Skip if the type name is too short
        if type_lower.len() < 3 {
            continue;
        }
        
        // Special handling for Int - check only as a full suffix
        if type_lower == "int" {
            // Only check if it appears as a complete suffix
            if name_lower.ends_with(&type_lower) {
                // Skip common false positive suffixes
                let common_false_positives = ["point", "print", "hint", "lint", "mint", "tint", "stint"];
                if common_false_positives.iter().any(|&suffix| name_lower.ends_with(suffix)) {
                    continue;
                }
                
                state.exceptable_add(
                    decl_identifier_with_type(decl.name().span(), name_str, type_name),
                    SyntaxElement::from(decl.syntax().clone()),
                    exceptable_nodes,
                );
                return;
            }
        } else {
            // For all other types, check only as a suffix
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
        Array[Int] valuesArray
        Map[String, Int] resultsMap
        Pair[Int, Float] dataPair
        Float measureFloat
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
            .any(|msg| msg.contains("valuesArray") && msg.contains("Array")));

        assert!(diagnostic_texts
            .iter()
            .any(|msg| msg.contains("resultsMap") && msg.contains("Map")));

        assert!(diagnostic_texts
            .iter()
            .any(|msg| msg.contains("dataPair") && msg.contains("Pair")));

        assert!(diagnostic_texts
            .iter()
            .any(|msg| msg.contains("measureFloat") && msg.contains("Float")));
    }

    #[test]
    fn test_allows_valid_names() {
        let source = r#"
version 1.0

workflow test {
    input {
        # File and String types are allowed with any naming pattern
        File fileInput
        File input_file
        String stringValue
        String value_string
        
        # These don't end with the type name
        Int counter
        Boolean is_active
        Array[String] word_list
        Map[String, Int] count_values
        
        # Special cases for Int that should be allowed
        Int checkpoint
        Int footprint
        Int mint_token
        
        # Using type name as prefix is allowed
        Int intValue
        Boolean boolFlag
        Float floatResult
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