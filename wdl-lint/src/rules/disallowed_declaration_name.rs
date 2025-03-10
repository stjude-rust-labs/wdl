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
        "Declaration names should not include their type"
    }

    fn explanation(&self) -> &'static str {
        "Declaration names should not include their type as a prefix or suffix. \
        This makes the code more verbose and often redundant. For example, use \
        'input_file' instead of 'file_input' or 'myFile'."
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

/// Extracts all type names from a type, recursively handling nested types
fn extract_type_names(ty: &wdl_ast::v1::Type, type_names: &mut Vec<String>) {
    // Handle different type variants
    match ty {
        wdl_ast::v1::Type::Array(array_type) => {
            // Add "Array" as a type name
            type_names.push("Array".to_string());
            
            // Recursively extract from the element type
            extract_type_names(&array_type.element_type(), type_names);
        },
        wdl_ast::v1::Type::Map(map_type) => {
            // Add "Map" as a type name
            type_names.push("Map".to_string());
            
            // Recursively extract from key and value types
            let (key, value) = map_type.types();
            extract_type_names(&wdl_ast::v1::Type::Primitive(key), type_names);
            extract_type_names(&value, type_names);
        },
        wdl_ast::v1::Type::Pair(pair_type) => {
            // Add "Pair" as a type name
            type_names.push("Pair".to_string());
            
            // Recursively extract from left and right types
            let (left, right) = pair_type.types();
            extract_type_names(&left, type_names);
            extract_type_names(&right, type_names);
        },
        wdl_ast::v1::Type::Primitive(primitive_type) => {
            // Add the primitive type name using to_string()
            type_names.push(primitive_type.to_string().trim_end_matches('?').to_string());
        },
        wdl_ast::v1::Type::Ref(type_ref) => {
            // Add the reference name
            type_names.push(type_ref.name().as_str().to_string());
        },
        _ => {
            // Handle other cases (Object, etc.)
        }
    }
}

/// Check declaration name for type prefixes or suffixes
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
    
    // Extract base type name(s) using AST methods
    let mut type_names = Vec::new();
    
    // Recursively extract all type names from the type
    extract_type_names(&ty, &mut type_names);
    
    // Remove optional markers (?) from type names
    let type_names: Vec<String> = type_names
        .into_iter()
        .map(|t| t.trim_end_matches('?').to_string())
        .collect();
    
    // Check each type name against the declaration name
    for type_name in &type_names {
        let type_lower = type_name.to_lowercase();
        
        // Skip if the type name is too short (to avoid false positives)
        if type_lower.len() < 3 {
            continue;
        }
        
        // More precise checks for type names in declaration identifiers
        // Only check prefix and suffix to avoid common false positives
        if name_lower.starts_with(&type_lower) || name_lower.ends_with(&type_lower) {
            // Special cases for short types like "Int" that might appear in legitimate words
            if type_lower == "int" {
                // Skip prefixes like "inter-" or "intra-" or common words containing "int"
                let common_false_positives = ["inter", "intra", "integ", "winter", "point"];
                if common_false_positives.iter().any(|&prefix| name_lower.starts_with(prefix)) {
                    continue;
                }
            }
            
            state.exceptable_add(
                decl_identifier_with_type(decl.name().span(), name_str, type_name),
                SyntaxElement::from(decl.syntax().clone()),
                exceptable_nodes,
            );
            return;
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
        File fileInput
        Int my_int
        Array[String] stringArray
        String good_name
    }
}
"#;

        let (document, _) = Document::parse(source);
        let mut validator = Validator::default();
        validator.add_visitor(DisallowedDeclarationNameRule::default());
        
        let result = validator.validate(&document);
        assert!(result.is_err());
        let diagnostics = result.err().unwrap();
        
        // We expect 3 errors: for fileInput, my_int, and stringArray
        assert_eq!(diagnostics.len(), 3);
        
        // Check that the correct identifiers are flagged
        let diagnostic_texts: Vec<_> = diagnostics
            .iter()
            .map(|d| d.message().to_string())
            .collect();
            
        assert!(diagnostic_texts
            .iter()
            .any(|msg| msg.contains("fileInput") && msg.contains("File")));
            
        assert!(diagnostic_texts
            .iter()
            .any(|msg| msg.contains("my_int") && msg.contains("Int")));
            
        assert!(diagnostic_texts
            .iter()
            .any(|msg| msg.contains("stringArray") && msg.contains("String")));
    }

    #[test]
    fn test_allows_valid_names() {
        let source = r#"
version 1.0

workflow test {
    input {
        File data
        Int count
        Array[String] words
        Map[String, Int] counts
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