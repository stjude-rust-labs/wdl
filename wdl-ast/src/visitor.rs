//! Implementation for AST visitation.
//!
//! An AST visitor is called when a WDL document is being visited (see
//! [Document::visit]); callbacks correspond to specific nodes and tokens in the
//! AST based on [SyntaxKind]. As `SyntaxKind` is the union of nodes and tokens
//! from _every_ version of WDL, the `Visitor` trait is also the union of
//! visitation callbacks.
//!
//! The [Visitor] trait is not WDL version-specific, meaning that the trait's
//! methods currently receive V1 representation of AST nodes.
//!
//! In the future, a major version change to the WDL specification will
//! introduce V2 representations for AST nodes that are either brand new or have
//! changed since V1.
//!
//! When this occurs, the `Visitor` trait will be extended to support the new
//! syntax; however, syntax that has not changed since V1 will continue to use
//! the V1 AST types.
//!
//! That means it is possible to receive callbacks for V1 nodes and tokens when
//! visiting a V2 document; the hope is that enables some visitors to be
//! "shared" across different WDL versions.

use rowan::WalkEvent;

use crate::v1::BoundDecl;
use crate::v1::CallStatement;
use crate::v1::CommandSection;
use crate::v1::CommandText;
use crate::v1::ConditionalStatement;
use crate::v1::Expr;
use crate::v1::ImportStatement;
use crate::v1::InputSection;
use crate::v1::MetadataObject;
use crate::v1::MetadataSection;
use crate::v1::OutputSection;
use crate::v1::ParameterMetadataSection;
use crate::v1::RuntimeSection;
use crate::v1::ScatterStatement;
use crate::v1::StringText;
use crate::v1::StructDefinition;
use crate::v1::TaskDefinition;
use crate::v1::UnboundDecl;
use crate::v1::WorkflowDefinition;
use crate::AstNode;
use crate::Comment;
use crate::Document;
use crate::SyntaxKind;
use crate::SyntaxNode;
use crate::VersionStatement;
use crate::VisitReason;
use crate::Whitespace;

/// A trait used to implement an AST visitor.
///
/// Each encountered node will receive a corresponding method call
/// that receives both a [VisitReason::Enter] call and a
/// matching [VisitReason::Exit] call.
#[allow(unused_variables)]
pub trait Visitor: Send + Sync {
    /// Represents the external visitation state.
    type State;

    /// Visits the root document node.
    fn document(&mut self, state: &mut Self::State, reason: VisitReason, doc: &Document) {}

    /// Visits a whitespace token.
    fn whitespace(&mut self, state: &mut Self::State, whitespace: &Whitespace) {}

    /// Visit a comment token.
    fn comment(&mut self, state: &mut Self::State, comment: &Comment) {}

    /// Visits a top-level version statement node.
    fn version_statement(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        stmt: &VersionStatement,
    ) {
    }

    /// Visits a top-level import statement node.
    fn import_statement(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        stmt: &ImportStatement,
    ) {
    }

    /// Visits a struct definition node.
    fn struct_definition(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        def: &StructDefinition,
    ) {
    }

    /// Visits a task definition node.
    fn task_definition(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        task: &TaskDefinition,
    ) {
    }

    /// Visits a workflow definition node.
    fn workflow_definition(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        workflow: &WorkflowDefinition,
    ) {
    }

    /// Visits an input section node.
    fn input_section(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        section: &InputSection,
    ) {
    }

    /// Visits an output section node.
    fn output_section(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        section: &OutputSection,
    ) {
    }

    /// Visits a command section node.
    fn command_section(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        section: &CommandSection,
    ) {
    }

    /// Visits a command text token in a command section node.
    fn command_text(&mut self, state: &mut Self::State, text: &CommandText) {}

    /// Visits a runtime section node.
    fn runtime_section(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        section: &RuntimeSection,
    ) {
    }

    /// Visits a metadata section node.
    fn metadata_section(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        section: &MetadataSection,
    ) {
    }

    /// Visits a parameter metadata section node.
    fn parameter_metadata_section(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        section: &ParameterMetadataSection,
    ) {
    }

    /// Visits a metadata object in a metadata or parameter metadata section.
    fn metadata_object(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        object: &MetadataObject,
    ) {
    }

    /// Visits an unbound declaration node.
    fn unbound_decl(&mut self, state: &mut Self::State, reason: VisitReason, decl: &UnboundDecl) {}

    /// Visits a bound declaration node.
    fn bound_decl(&mut self, state: &mut Self::State, reason: VisitReason, decl: &BoundDecl) {}

    /// Visits an expression node.
    fn expr(&mut self, state: &mut Self::State, reason: VisitReason, expr: &Expr) {}

    /// Visits a string text token in a literal string node.
    fn string_text(&mut self, state: &mut Self::State, text: &StringText) {}

    /// Visits a conditional statement node in a workflow.
    fn conditional_statement(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        stmt: &ConditionalStatement,
    ) {
    }

    /// Visits a scatter statement node in a workflow.
    fn scatter_statement(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        stmt: &ScatterStatement,
    ) {
    }

    /// Visits a call statement node in a workflow.
    fn call_statement(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        stmt: &CallStatement,
    ) {
    }
}

/// Used to visit each descendant node of the given root in a preorder
/// traversal.
pub(crate) fn visit<V: Visitor>(root: &SyntaxNode, state: &mut V::State, visitor: &mut V) {
    for event in root.preorder_with_tokens() {
        let (reason, element) = match event {
            WalkEvent::Enter(node) => (VisitReason::Enter, node),
            WalkEvent::Leave(node) => (VisitReason::Exit, node),
        };

        match element.kind() {
            SyntaxKind::RootNode => {
                visitor.document(state, reason, &Document(element.into_node().unwrap()))
            }
            SyntaxKind::VersionStatementNode => visitor.version_statement(
                state,
                reason,
                &VersionStatement(element.into_node().unwrap()),
            ),
            SyntaxKind::ImportStatementNode => visitor.import_statement(
                state,
                reason,
                &ImportStatement(element.into_node().unwrap()),
            ),
            SyntaxKind::ImportAliasNode => {
                // Skip these nodes as they're part of an import statement
            }
            SyntaxKind::StructDefinitionNode => visitor.struct_definition(
                state,
                reason,
                &StructDefinition(element.into_node().unwrap()),
            ),
            SyntaxKind::TaskDefinitionNode => visitor.task_definition(
                state,
                reason,
                &TaskDefinition(element.into_node().unwrap()),
            ),
            SyntaxKind::WorkflowDefinitionNode => visitor.workflow_definition(
                state,
                reason,
                &WorkflowDefinition(element.into_node().unwrap()),
            ),
            SyntaxKind::UnboundDeclNode => {
                visitor.unbound_decl(state, reason, &UnboundDecl(element.into_node().unwrap()))
            }
            SyntaxKind::BoundDeclNode => {
                visitor.bound_decl(state, reason, &BoundDecl(element.into_node().unwrap()))
            }
            SyntaxKind::PrimitiveTypeNode
            | SyntaxKind::MapTypeNode
            | SyntaxKind::ArrayTypeNode
            | SyntaxKind::PairTypeNode
            | SyntaxKind::ObjectTypeNode
            | SyntaxKind::TypeRefNode => {
                // Skip these nodes as they're part of declarations
            }
            SyntaxKind::InputSectionNode => {
                visitor.input_section(state, reason, &InputSection(element.into_node().unwrap()))
            }
            SyntaxKind::OutputSectionNode => {
                visitor.output_section(state, reason, &OutputSection(element.into_node().unwrap()))
            }
            SyntaxKind::CommandSectionNode => visitor.command_section(
                state,
                reason,
                &CommandSection(element.into_node().unwrap()),
            ),
            SyntaxKind::RuntimeSectionNode => visitor.runtime_section(
                state,
                reason,
                &RuntimeSection(element.into_node().unwrap()),
            ),
            SyntaxKind::RuntimeItemNode => {
                // Skip this node as it's part of a runtime section
            }
            SyntaxKind::MetadataSectionNode => visitor.metadata_section(
                state,
                reason,
                &MetadataSection(element.into_node().unwrap()),
            ),
            SyntaxKind::ParameterMetadataSectionNode => visitor.parameter_metadata_section(
                state,
                reason,
                &ParameterMetadataSection(element.into_node().unwrap()),
            ),
            SyntaxKind::MetadataObjectNode => visitor.metadata_object(
                state,
                reason,
                &MetadataObject(element.into_node().unwrap()),
            ),
            SyntaxKind::MetadataObjectItemNode
            | SyntaxKind::MetadataArrayNode
            | SyntaxKind::LiteralNullNode => {
                // Skip these nodes as they're part of a metadata section
            }
            k if Expr::can_cast(k) => visitor.expr(
                state,
                reason,
                &Expr::cast(element.into_node().unwrap()).expect("node should cast"),
            ),
            SyntaxKind::LiteralMapItemNode
            | SyntaxKind::LiteralObjectItemNode
            | SyntaxKind::LiteralStructItemNode => {
                // Skip these nodes as they're part of literal expressions
            }
            k @ (SyntaxKind::LiteralIntegerNode
            | SyntaxKind::LiteralFloatNode
            | SyntaxKind::LiteralBooleanNode
            | SyntaxKind::LiteralNoneNode
            | SyntaxKind::LiteralStringNode
            | SyntaxKind::LiteralPairNode
            | SyntaxKind::LiteralArrayNode
            | SyntaxKind::LiteralMapNode
            | SyntaxKind::LiteralObjectNode
            | SyntaxKind::LiteralStructNode
            | SyntaxKind::ParenthesizedExprNode
            | SyntaxKind::NameRefNode
            | SyntaxKind::IfExprNode
            | SyntaxKind::LogicalNotExprNode
            | SyntaxKind::NegationExprNode
            | SyntaxKind::LogicalOrExprNode
            | SyntaxKind::LogicalAndExprNode
            | SyntaxKind::EqualityExprNode
            | SyntaxKind::InequalityExprNode
            | SyntaxKind::LessExprNode
            | SyntaxKind::LessEqualExprNode
            | SyntaxKind::GreaterExprNode
            | SyntaxKind::GreaterEqualExprNode
            | SyntaxKind::AdditionExprNode
            | SyntaxKind::SubtractionExprNode
            | SyntaxKind::MultiplicationExprNode
            | SyntaxKind::DivisionExprNode
            | SyntaxKind::ModuloExprNode
            | SyntaxKind::CallExprNode
            | SyntaxKind::IndexExprNode
            | SyntaxKind::AccessExprNode) => {
                unreachable!("`{k:?}` should be handled by `Expr::can_cast`")
            }
            SyntaxKind::PlaceholderNode
            | SyntaxKind::PlaceholderSepOptionNode
            | SyntaxKind::PlaceholderDefaultOptionNode
            | SyntaxKind::PlaceholderTrueFalseOptionNode => {
                // Skip these nodes as they're part of a placeholder
            }
            SyntaxKind::ConditionalStatementNode => visitor.conditional_statement(
                state,
                reason,
                &ConditionalStatement(element.into_node().unwrap()),
            ),
            SyntaxKind::ScatterStatementNode => visitor.scatter_statement(
                state,
                reason,
                &ScatterStatement(element.into_node().unwrap()),
            ),
            SyntaxKind::CallStatementNode => {
                visitor.call_statement(state, reason, &CallStatement(element.into_node().unwrap()))
            }
            SyntaxKind::CallTargetNode
            | SyntaxKind::CallAliasNode
            | SyntaxKind::CallAfterNode
            | SyntaxKind::CallInputItemNode => {
                // Skip these nodes as they're part of a call statement
            }
            SyntaxKind::Abandoned | SyntaxKind::MAX => {
                unreachable!("node should not exist in the tree")
            }
            SyntaxKind::Whitespace if reason == VisitReason::Enter => {
                visitor.whitespace(state, &Whitespace(element.into_token().unwrap()))
            }
            SyntaxKind::Comment if reason == VisitReason::Enter => {
                visitor.comment(state, &Comment(element.into_token().unwrap()))
            }
            SyntaxKind::LiteralStringText if reason == VisitReason::Enter => {
                visitor.string_text(state, &StringText(element.into_token().unwrap()))
            }
            SyntaxKind::LiteralCommandText if reason == VisitReason::Enter => {
                visitor.command_text(state, &CommandText(element.into_token().unwrap()))
            }
            _ => {
                // Skip remaining tokens
            }
        }
    }
}
