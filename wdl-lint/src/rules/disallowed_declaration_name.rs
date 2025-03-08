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
fn decl_identifier_with_type(span: Span, type_name: &str) -> Diagnostic {
    Diagnostic::note(format!("declaration identifier contains type name '{}'", type_name))
        .with_rule(ID)
        .with_highlight(span)
        .with_fix("rename the identifier to not include the type name")
}

/// A lint rule for disallowed declaration names.
#[derive(Default, Debug, Clone, Copy)]
pub struct DisallowedDeclarationNameRule;

impl Rule for DisallowedDeclarationNameRule {
    fn id(&self) -> &'static str {
        ID
    }

    fn description(&self) -> &'static str {
        "Ensures declaration names do not have type prefixes or suffixes."
    }

    fn explanation(&self) -> &'static str {
        "Declaration names should be descriptive of their content and not include type prefixes or suffixes. \
         For example, 'File gtfFile' or 'Int my_int' are discouraged as they redundantly encode the type \
         information in the variable name. This rule helps maintain clean and concise code by avoiding \
         this form of Hungarian notation."
    }

    fn tags(&self) -> TagSet {
        TagSet::new(&[Tag::Naming])
    }

    fn exceptable_nodes(&self) -> Option<&'static [wdl_ast::SyntaxKind]> {
        Some(&[
            SyntaxKind::VersionStatementNode,
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
    
    // Check for exact type name at beginning or end
    if name_lower.starts_with(&type_lower) || name_lower.ends_with(&type_lower) {
        state.exceptable_add(
            decl_identifier_with_type(decl.name().span(), base_type),
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
        
        // Skip if this isn't the actual type of the declaration
        if type_lower != base_type.to_lowercase() {
            continue;
        }
        
        // Check for type name with various word boundaries
        if (name_lower.starts_with(&type_lower) && 
            (name_str.len() == type_name.len() || 
             name_str.chars().nth(type_name.len()).map_or(false, |c| c.is_uppercase() || c == '_'))) ||
           (name_lower.ends_with(&type_lower) && 
            (name_str.len() == type_name.len() || 
             name_str.chars().nth(name_str.len() - type_name.len() - 1).map_or(false, |c| c.is_lowercase() || c == '_')))
        {
            state.exceptable_add(
                decl_identifier_with_type(decl.name().span(), type_name),
                SyntaxElement::from(decl.syntax().clone()),
                exceptable_nodes,
            );
            return;
        }
    }
}