use std::any::TypeId;
use std::collections::HashMap;
use std::sync::LazyLock;

use wdl_grammar::WorkflowDescriptionLanguage;
use wdl_grammar::ALL_SYNTAX_KIND;

use crate::v1;
use crate::AstNode;
use crate::AstToken;
use crate::Comment;
use crate::Document;
use crate::Ident;
use crate::SyntaxKind;
use crate::Version;
use crate::VersionStatement;
use crate::Whitespace;

/// A private module for sealed traits.
mod private {
    /// The sealed trait for [`AstNodeRegistrant`](super::AstNodeRegistrant).
    pub trait SealedNode {}

    /// The sealed trait for [`AstTokenRegistrant`](super::AstTokenRegistrant).
    pub trait SealedToken {}
}

/// A registry of all known mappings between types that implement [`AstNode`]
/// and the [`SyntaxKind`] they can map to.
static REGISTRY: LazyLock<HashMap<TypeId, Box<[SyntaxKind]>>> = LazyLock::new(|| {
    let types = vec![
        Comment::register(),
        Document::register(),
        Ident::register(),
        v1::AccessExpr::register(),
        v1::AdditionExpr::register(),
        v1::ArrayType::register(),
        v1::Ast::register(),
        v1::BoundDecl::register(),
        v1::CallAfter::register(),
        v1::CallAlias::register(),
        v1::CallExpr::register(),
        v1::CallInputItem::register(),
        v1::CallTarget::register(),
        v1::CommandSection::register(),
        v1::CommandText::register(),
        v1::ConditionalStatement::register(),
        v1::Decl::register(),
        v1::DefaultOption::register(),
        v1::DivisionExpr::register(),
        v1::DocumentItem::register(),
        v1::EqualityExpr::register(),
        v1::ExponentiationExpr::register(),
        v1::Expr::register(),
        v1::Float::register(),
        v1::GreaterEqualExpr::register(),
        v1::GreaterExpr::register(),
        v1::HintsItem::register(),
        v1::HintsSection::register(),
        v1::IfExpr::register(),
        v1::ImportAlias::register(),
        v1::ImportStatement::register(),
        v1::IndexExpr::register(),
        v1::InequalityExpr::register(),
        v1::InputSection::register(),
        v1::Integer::register(),
        v1::LessEqualExpr::register(),
        v1::LessExpr::register(),
        v1::LiteralArray::register(),
        v1::LiteralBoolean::register(),
        v1::LiteralExpr::register(),
        v1::LiteralFloat::register(),
        v1::LiteralHints::register(),
        v1::LiteralHintsItem::register(),
        v1::LiteralInput::register(),
        v1::LiteralInputItem::register(),
        v1::LiteralInteger::register(),
        v1::LiteralMap::register(),
        v1::LiteralMapItem::register(),
        v1::LiteralNone::register(),
        v1::LiteralObject::register(),
        v1::LiteralObjectItem::register(),
        v1::LiteralOutput::register(),
        v1::LiteralOutputItem::register(),
        v1::LiteralPair::register(),
        v1::LiteralString::register(),
        v1::LiteralStruct::register(),
        v1::LiteralStructItem::register(),
        v1::LogicalAndExpr::register(),
        v1::LogicalNotExpr::register(),
        v1::LogicalOrExpr::register(),
        v1::MapType::register(),
        v1::MetadataArray::register(),
        v1::MetadataObjectItem::register(),
        v1::MetadataSection::register(),
        v1::ModuloExpr::register(),
        v1::MultiplicationExpr::register(),
        v1::NameRef::register(),
        v1::NegationExpr::register(),
        v1::ObjectType::register(),
        v1::OutputSection::register(),
        v1::PairType::register(),
        v1::ParameterMetadataSection::register(),
        v1::ParenthesizedExpr::register(),
        v1::Placeholder::register(),
        v1::PlaceholderOption::register(),
        v1::PrimitiveType::register(),
        v1::RequirementsItem::register(),
        v1::RequirementsSection::register(),
        v1::RuntimeItem::register(),
        v1::RuntimeSection::register(),
        v1::ScatterStatement::register(),
        v1::SectionParent::register(),
        v1::SepOption::register(),
        v1::StringText::register(),
        v1::StructDefinition::register(),
        v1::StructItem::register(),
        v1::SubtractionExpr::register(),
        v1::TaskDefinition::register(),
        v1::TaskItem::register(),
        v1::TrueFalseOption::register(),
        v1::Type::register(),
        v1::TypeRef::register(),
        v1::UnboundDecl::register(),
        v1::WorkflowDefinition::register(),
        v1::WorkflowItem::register(),
        v1::WorkflowStatement::register(),
        Version::register(),
        VersionStatement::register(),
        Whitespace::register(),
    ];

    let mut result = HashMap::new();

    // NOTE: this is done this way instead of collecting to check on the fly to
    // make sure that no keys are duplicated.
    for (r#type, kinds) in types {
        if result.contains_key(&r#type) {
            panic!("the `{:?}` key is duplicated", r#type);
        }

        result.insert(r#type, kinds);
    }

    result
});

/// Computes the inverse of the registry (maps [`SyntaxKind`]s to every type
/// that can cast from them).

fn inverse_registry() -> HashMap<SyntaxKind, Box<[TypeId]>> {
    let mut result = HashMap::<SyntaxKind, Vec<TypeId>>::new();

    for (key, values) in REGISTRY.iter() {
        for value in values.into_iter() {
            result.entry(value.to_owned()).or_default().push(*key);
        }
    }

    result
        .into_iter()
        .map(|(key, values)| (key, values.into_boxed_slice()))
        .collect()
}

trait AstNodeRegistrant: private::SealedNode {
    /// Registers the AST element.
    fn register() -> (TypeId, Box<[SyntaxKind]>);
}

impl<T: AstNode<Language = WorkflowDescriptionLanguage> + 'static> private::SealedNode for T {}

impl<T: AstNode<Language = WorkflowDescriptionLanguage> + 'static> AstNodeRegistrant for T {
    fn register() -> (TypeId, Box<[SyntaxKind]>) {
        (
            TypeId::of::<T>(),
            ALL_SYNTAX_KIND
                .iter()
                .filter(|kind| T::can_cast(**kind))
                .cloned()
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        )
    }
}

trait AstTokenRegistrant: private::SealedToken {
    /// Registers a type implementing `AstToken` that can be  .
    fn register() -> (TypeId, Box<[SyntaxKind]>);
}

impl<T: AstToken + 'static> private::SealedToken for T {}

impl<T: AstToken + 'static> AstTokenRegistrant for T {
    fn register() -> (TypeId, Box<[SyntaxKind]>) {
        (
            TypeId::of::<T>(),
            ALL_SYNTAX_KIND
                .iter()
                .filter(|kind| T::can_cast(**kind))
                .cloned()
                .collect::<Vec<_>>()
                .into_boxed_slice(),
        )
    }
}

mod tests {
    use super::*;

    #[test]
    fn ensure_each_syntax_element_has_an_ast_node() {
        let mut missing = Vec::new();
        let inverse_registry = inverse_registry();

        for kind in ALL_SYNTAX_KIND {
            // NOTE: these are pseudo elements and should not be reported.
            if *kind == SyntaxKind::Abandoned || *kind == SyntaxKind::MAX {
                continue;
            }

            if !inverse_registry.contains_key(kind) {
                missing.push(kind);
            }
        }

        if !missing.is_empty() {
            let mut missing = missing
                .into_iter()
                .map(|kind| format!("{:?}", kind))
                .collect::<Vec<_>>();
            missing.sort();

            panic!(
                "detected `SyntaxKind`s without an associated `AstNode` (n={}): {}",
                missing.len(),
                missing.join(", ")
            )
        }
    }
}
