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
            SyntaxKind::InputSectionNode,
            SyntaxKind::OutputSectionNode,
        ])
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
    let type_str = decl.ty().to_string();
    
    // Extract the base type (remove Array[] and optional ? modifiers)
    let base_type = if type_str.starts_with("Array[") {
        // Extract the inner type from Array[Type]
        let inner_type = type_str
            .trim_start_matches("Array[")
            .trim_end_matches(']')
            .trim_end_matches('?');
        inner_type
    } else {
        type_str.trim_end_matches('?')
    };
    
    // Check if the declaration name contains the type name (case insensitive)
    let name_lower = name_str.to_lowercase();
    let type_lower = base_type.to_lowercase();
    
    // Check for type name at beginning or end
    if name_lower.starts_with(&type_lower) || name_lower.ends_with(&type_lower) ||
       name_lower.contains(&type_lower) {  // Also check for type name anywhere in the identifier
        state.exceptable_add(
            decl_identifier_with_type(decl.name().span(), name_str, base_type),
            SyntaxElement::from(decl.syntax().clone()),
            exceptable_nodes,
        );
        return;
    }
    
    // Check for common WDL types that might be embedded in the name
    let common_types = [
        "Int", "Float", "Boolean", "String", "File", "Directory",
        "Map", "Pair", "Array", "Struct"
    ];
    
    for &type_name in &common_types {
        let type_lower = type_name.to_lowercase();
        
        // Only check if this type matches the actual type of the declaration
        if base_type.to_lowercase().contains(&type_lower) {
            if name_lower.contains(&type_lower) {
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