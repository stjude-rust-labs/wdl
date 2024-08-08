use std::any::type_name;
use std::collections::HashMap;
use std::sync::LazyLock;

use wdl_grammar::WorkflowDescriptionLanguage;
use wdl_grammar::ALL_SYNTAX_KIND;

use crate::v1;
use crate::AstNode;
use crate::AstToken;
use crate::Comment;
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
static REGISTRY: LazyLock<HashMap<&'static str, Box<[SyntaxKind]>>> = LazyLock::new(|| {
    let types = vec![
        Comment::register(),
        Ident::register(),
        v1::AccessExpr::register(),
        v1::AdditionExpr::register(),
        v1::AfterKeyword::register(),
        v1::AliasKeyword::register(),
        v1::ArrayType::register(),
        v1::ArrayTypeKeyword::register(),
        v1::AsKeyword::register(),
        v1::Assignment::register(),
        v1::Ast::register(),
        v1::Asterisk::register(),
        v1::BooleanTypeKeyword::register(),
        v1::BoundDecl::register(),
        v1::CallAfter::register(),
        v1::CallAlias::register(),
        v1::CallExpr::register(),
        v1::CallInputItem::register(),
        v1::CallKeyword::register(),
        v1::CallStatement::register(),
        v1::CallTarget::register(),
        v1::CloseBrace::register(),
        v1::CloseBracket::register(),
        v1::CloseHeredoc::register(),
        v1::CloseParen::register(),
        v1::Colon::register(),
        v1::Comma::register(),
        v1::CommandKeyword::register(),
        v1::CommandSection::register(),
        v1::CommandText::register(),
        v1::ConditionalStatement::register(),
        v1::DefaultOption::register(),
        v1::DirectoryTypeKeyword::register(),
        v1::DivisionExpr::register(),
        v1::Dot::register(),
        v1::DoubleQuote::register(),
        v1::ElseKeyword::register(),
        v1::Equal::register(),
        v1::EqualityExpr::register(),
        v1::Exclamation::register(),
        v1::Exponentiation::register(),
        v1::ExponentiationExpr::register(),
        v1::FalseKeyword::register(),
        v1::FileTypeKeyword::register(),
        v1::Float::register(),
        v1::FloatTypeKeyword::register(),
        v1::Greater::register(),
        v1::GreaterEqual::register(),
        v1::GreaterEqualExpr::register(),
        v1::GreaterExpr::register(),
        v1::HintsItem::register(),
        v1::HintsKeyword::register(),
        v1::HintsSection::register(),
        v1::IfExpr::register(),
        v1::IfKeyword::register(),
        v1::ImportAlias::register(),
        v1::ImportKeyword::register(),
        v1::ImportStatement::register(),
        v1::IndexExpr::register(),
        v1::InequalityExpr::register(),
        v1::InKeyword::register(),
        v1::InputKeyword::register(),
        v1::InputSection::register(),
        v1::Integer::register(),
        v1::IntTypeKeyword::register(),
        v1::Less::register(),
        v1::LessEqual::register(),
        v1::LessEqualExpr::register(),
        v1::LessExpr::register(),
        v1::LiteralArray::register(),
        v1::LiteralBoolean::register(),
        v1::LiteralFloat::register(),
        v1::LiteralHints::register(),
        v1::LiteralHintsItem::register(),
        v1::LiteralInput::register(),
        v1::LiteralInputItem::register(),
        v1::LiteralInteger::register(),
        v1::LiteralMap::register(),
        v1::LiteralMapItem::register(),
        v1::LiteralNone::register(),
        v1::LiteralNull::register(),
        v1::LiteralObject::register(),
        v1::LiteralObjectItem::register(),
        v1::LiteralOutput::register(),
        v1::LiteralOutputItem::register(),
        v1::LiteralPair::register(),
        v1::LiteralString::register(),
        v1::LiteralStruct::register(),
        v1::LiteralStructItem::register(),
        v1::LogicalAnd::register(),
        v1::LogicalAndExpr::register(),
        v1::LogicalNotExpr::register(),
        v1::LogicalOr::register(),
        v1::LogicalOrExpr::register(),
        v1::MapType::register(),
        v1::MapTypeKeyword::register(),
        v1::MetadataArray::register(),
        v1::MetadataObject::register(),
        v1::MetadataObjectItem::register(),
        v1::MetadataSection::register(),
        v1::MetaKeyword::register(),
        v1::Minus::register(),
        v1::ModuloExpr::register(),
        v1::MultiplicationExpr::register(),
        v1::NameRef::register(),
        v1::NegationExpr::register(),
        v1::NoneKeyword::register(),
        v1::NotEqual::register(),
        v1::NullKeyword::register(),
        v1::ObjectKeyword::register(),
        v1::ObjectType::register(),
        v1::ObjectTypeKeyword::register(),
        v1::OpenBrace::register(),
        v1::OpenBracket::register(),
        v1::OpenHeredoc::register(),
        v1::OpenParen::register(),
        v1::OutputKeyword::register(),
        v1::OutputSection::register(),
        v1::PairType::register(),
        v1::PairTypeKeyword::register(),
        v1::ParameterMetadataSection::register(),
        v1::ParameterMetaKeyword::register(),
        v1::ParenthesizedExpr::register(),
        v1::Percent::register(),
        v1::Placeholder::register(),
        v1::PlaceholderOpen::register(),
        v1::Plus::register(),
        v1::PrimitiveType::register(),
        v1::QuestionMark::register(),
        v1::RequirementsItem::register(),
        v1::RequirementsKeyword::register(),
        v1::RequirementsSection::register(),
        v1::RuntimeItem::register(),
        v1::RuntimeKeyword::register(),
        v1::RuntimeSection::register(),
        v1::ScatterKeyword::register(),
        v1::ScatterStatement::register(),
        v1::SepOption::register(),
        v1::SingleQuote::register(),
        v1::Slash::register(),
        v1::StringText::register(),
        v1::StringTypeKeyword::register(),
        v1::StructDefinition::register(),
        v1::StructKeyword::register(),
        v1::SubtractionExpr::register(),
        v1::TaskDefinition::register(),
        v1::TaskKeyword::register(),
        v1::ThenKeyword::register(),
        v1::TrueFalseOption::register(),
        v1::TrueKeyword::register(),
        v1::TypeRef::register(),
        v1::UnboundDecl::register(),
        v1::Unknown::register(),
        v1::VersionKeyword::register(),
        v1::WorkflowDefinition::register(),
        v1::WorkflowKeyword::register(),
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

fn inverse_registry() -> HashMap<SyntaxKind, Box<[&'static str]>> {
    let mut result = HashMap::<SyntaxKind, Vec<&'static str>>::new();

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
    /// Returns all [`SyntaxKind`]s that can be cast into this AST node type.
    fn register() -> (&'static str, Box<[SyntaxKind]>);
}

impl<T: AstNode<Language = WorkflowDescriptionLanguage> + 'static> private::SealedNode for T {}

impl<T: AstNode<Language = WorkflowDescriptionLanguage> + 'static> AstNodeRegistrant for T {
    fn register() -> (&'static str, Box<[SyntaxKind]>) {
        (
            type_name::<T>(),
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
    /// Returns all [`SyntaxKind`]s that can be cast into this AST token type.
    fn register() -> (&'static str, Box<[SyntaxKind]>);
}

impl<T: AstToken + 'static> private::SealedToken for T {}

impl<T: AstToken + 'static> AstTokenRegistrant for T {
    fn register() -> (&'static str, Box<[SyntaxKind]>) {
        (
            type_name::<T>(),
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
    fn ensure_each_syntax_element_has_exactly_one_ast_node_or_ast_token() {
        let mut missing = Vec::new();
        let mut multiple = Vec::new();

        let inverse_registry = inverse_registry();

        for kind in ALL_SYNTAX_KIND {
            // NOTE: these are pseudo elements and should not be reported.
            if kind.is_pseudokind() {
                continue;
            }

            match inverse_registry.get(kind) {
                // SAFETY: because this is an inverse registry, only
                // [`SyntaxKind`]s with at least one registered implementing
                // type would be registered here. Thus, by design of the
                // `inverse_registry()` method, this will never occur.
                Some(values) if values.is_empty() => {
                    unreachable!("the inverse registry should never contain an empty array")
                }
                Some(values) if values.len() > 1 => multiple.push((kind, values)),
                None => missing.push(kind),
                // NOTE: this is essentially only if the values exist and the
                // length is 1—in that case, there is a one to one mapping,
                // which is what we would like the case to be.
                _ => {}
            }
        }

        if !missing.is_empty() {
            let mut missing = missing
                .into_iter()
                .map(|kind| format!("{:?}", kind))
                .collect::<Vec<_>>();
            missing.sort();

            panic!(
                "detected `SyntaxKind`s without an associated `AstNode`/`AstToken` (n={}): {}",
                missing.len(),
                missing.join(", ")
            )
        }

        if !multiple.is_empty() {
            multiple.sort();
            let mut multiple = multiple
                .into_iter()
                .map(|(kind, types)| {
                    let mut types = types.clone();
                    types.sort();

                    let mut result = format!("== {:?} ==", kind);
                    for r#type in types {
                        result.push_str("\n* ");
                        result.push_str(r#type);
                    }

                    result
                })
                .collect::<Vec<_>>();
            multiple.sort();

            panic!(
                "detected `SyntaxKind`s associated with multiple `AstNode`s/`AstToken`s \
                 (n={}):\n\n{}",
                multiple.len(),
                multiple.join("\n\n")
            )
        }
    }
}
