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

use std::sync::Arc;

use rowan::WalkEvent;
use wdl_ast::SyntaxElement;

use wdl_ast::AstToken;
use wdl_ast::Comment;
use wdl_ast::Diagnostic;
use wdl_ast::Document;
use wdl_ast::SupportedVersion;
use wdl_ast::SyntaxKind;
use wdl_ast::SyntaxNode;
use crate::SyntaxNodeExt;
use wdl_ast::VersionStatement;
use wdl_ast::Whitespace;
use wdl_ast::v1::BoundDecl;
use wdl_ast::v1::CallStatement;
use wdl_ast::v1::CommandSection;
use wdl_ast::v1::CommandText;
use wdl_ast::v1::ConditionalStatement;
use wdl_ast::v1::Expr;
use wdl_ast::v1::ImportStatement;
use wdl_ast::v1::InputSection;
use wdl_ast::v1::MetadataArray;
use wdl_ast::v1::MetadataObject;
use wdl_ast::v1::MetadataObjectItem;
use wdl_ast::v1::MetadataSection;
use wdl_ast::v1::OutputSection;
use wdl_ast::v1::ParameterMetadataSection;
use wdl_ast::v1::Placeholder;
use wdl_ast::v1::RequirementsSection;
use wdl_ast::v1::RuntimeItem;
use wdl_ast::v1::RuntimeSection;
use wdl_ast::v1::ScatterStatement;
use wdl_ast::v1::StringText;
use wdl_ast::v1::StructDefinition;
use wdl_ast::v1::TaskDefinition;
use wdl_ast::v1::TaskHintsSection;
use wdl_ast::v1::UnboundDecl;
use wdl_ast::v1::WorkflowDefinition;
use wdl_ast::v1::WorkflowHintsSection;

/// Represents the reason an AST node has been visited.
///
/// Each node is visited exactly once, but the visitor will receive
/// a call for entering the node and a call for exiting the node.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum VisitReason {
    /// The visit has entered the node.
    Enter,
    /// The visit has exited the node.
    Exit,
}

/// Represents a collection of validation diagnostics.
///
/// Validation visitors receive a diagnostics collection during
/// visitation of the AST.
#[derive(Debug, Default)]
pub struct Diagnostics(pub(crate) Vec<Diagnostic>);

impl Diagnostics {
    /// Adds a diagnostic to the collection.
    pub fn add(&mut self, diagnostic: Diagnostic) {
        self.0.push(diagnostic);
    }

    /// Adds a diagnostic to the collection, unless the diagnostic is for an
    /// element that has an exception for the given rule.
    ///
    /// If the diagnostic does not have a rule, the diagnostic is always added.
    pub fn exceptable_add(
        &mut self,
        diagnostic: Diagnostic,
        element: SyntaxElement,
        exceptable_nodes: &Option<&'static [SyntaxKind]>,
    ) {
        if let Some(rule) = diagnostic.rule() {
            for node in element.ancestors().filter(|node| {
                exceptable_nodes
                    .as_ref()
                    .is_none_or(|nodes| nodes.contains(&node.kind()))
            }) {
                if node.is_rule_excepted(rule) {
                    // Rule is currently excepted, don't add the diagnostic
                    return;
                }
            }
        }

        self.add(diagnostic);
    }

    /// Returns the number of diagnostics in the collection.
    pub fn len(&self) -> usize {
        self.0.len()
    }

    /// Extends the collection with another collection of diagnostics.
    pub fn extend(&mut self, diagnostics: Diagnostics) {
        self.0.extend(diagnostics.0);
    }

    /// Returns whether the collection is empty.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    /// Sorts the diagnostics in the collection.
    pub fn sort(&mut self) {
        self.0.sort();
    }

    /// Returns the first diagnostic in the collection if it exists.
    pub fn first(&self) -> Option<&Diagnostic> {
        self.0.first()
    }
}

impl FromIterator<Diagnostic> for Diagnostics {
    fn from_iter<T: IntoIterator<Item = Diagnostic>>(iter: T) -> Self {
        Self(iter.into_iter().collect())
    }
}

impl From<Arc<Diagnostics>> for Diagnostics {
    fn from(diagnostics: Arc<Diagnostics>) -> Self {
        Self(diagnostics.0.clone())
    }
}

impl Iterator for Diagnostics {
    type Item = Diagnostic;

    fn next(&mut self) -> Option<Self::Item> {
        self.0.pop()
    }
}

impl<'a> IntoIterator for &'a Diagnostics {
    type IntoIter = std::slice::Iter<'a, Diagnostic>;
    type Item = &'a Diagnostic;

    fn into_iter(self) -> Self::IntoIter {
        self.0.iter()
    }
}

/// A trait used to implement an AST visitor.
///
/// Each encountered node will receive a corresponding method call
/// that receives both a [VisitReason::Enter] call and a
/// matching [VisitReason::Exit] call.
#[allow(unused_variables)]
pub trait Visitor {
    /// Visits the root document node.
    ///
    /// A visitor must implement this method and response to
    /// `VisitReason::Enter` with resetting any internal state so that a visitor
    /// may be reused between documents.
    fn document(
        &mut self,
        diagnostics: &mut Diagnostics,
        reason: VisitReason,
        doc: &Document,
        version: SupportedVersion,
    );

    /// Visits a whitespace token.
    fn whitespace(&mut self, diagnostics: &mut Diagnostics, whitespace: &Whitespace) {}

    /// Visit a comment token.
    fn comment(&mut self, diagnostics: &mut Diagnostics, comment: &Comment) {}

    /// Visits a top-level version statement node.
    fn version_statement(
        &mut self,
        diagnostics: &mut Diagnostics,
        reason: VisitReason,
        stmt: &VersionStatement,
    ) {
    }

    /// Visits a top-level import statement node.
    fn import_statement(
        &mut self,
        diagnostics: &mut Diagnostics,
        reason: VisitReason,
        stmt: &ImportStatement,
    ) {
    }

    /// Visits a struct definition node.
    fn struct_definition(
        &mut self,
        diagnostics: &mut Diagnostics,
        reason: VisitReason,
        def: &StructDefinition,
    ) {
    }

    /// Visits a task definition node.
    fn task_definition(
        &mut self,
        diagnostics: &mut Diagnostics,
        reason: VisitReason,
        task: &TaskDefinition,
    ) {
    }

    /// Visits a workflow definition node.
    fn workflow_definition(
        &mut self,
        diagnostics: &mut Diagnostics,
        reason: VisitReason,
        workflow: &WorkflowDefinition,
    ) {
    }

    /// Visits an input section node.
    fn input_section(
        &mut self,
        diagnostics: &mut Diagnostics,
        reason: VisitReason,
        section: &InputSection,
    ) {
    }

    /// Visits an output section node.
    fn output_section(
        &mut self,
        diagnostics: &mut Diagnostics,
        reason: VisitReason,
        section: &OutputSection,
    ) {
    }

    /// Visits a command section node.
    fn command_section(
        &mut self,
        diagnostics: &mut Diagnostics,
        reason: VisitReason,
        section: &CommandSection,
    ) {
    }

    /// Visits a command text token in a command section node.
    fn command_text(&mut self, diagnostics: &mut Diagnostics, text: &CommandText) {}

    /// Visits a requirements section node.
    fn requirements_section(
        &mut self,
        diagnostics: &mut Diagnostics,
        reason: VisitReason,
        section: &RequirementsSection,
    ) {
    }

    /// Visits a task hints section node.
    fn task_hints_section(
        &mut self,
        diagnostics: &mut Diagnostics,
        reason: VisitReason,
        section: &TaskHintsSection,
    ) {
    }

    /// Visits a workflow hints section node.
    fn workflow_hints_section(
        &mut self,
        diagnostics: &mut Diagnostics,
        reason: VisitReason,
        section: &WorkflowHintsSection,
    ) {
    }

    /// Visits a runtime section node.
    fn runtime_section(
        &mut self,
        diagnostics: &mut Diagnostics,
        reason: VisitReason,
        section: &RuntimeSection,
    ) {
    }

    /// Visits a runtime item node.
    fn runtime_item(
        &mut self,
        diagnostics: &mut Diagnostics,
        reason: VisitReason,
        item: &RuntimeItem,
    ) {
    }

    /// Visits a metadata section node.
    fn metadata_section(
        &mut self,
        diagnostics: &mut Diagnostics,
        reason: VisitReason,
        section: &MetadataSection,
    ) {
    }

    /// Visits a parameter metadata section node.
    fn parameter_metadata_section(
        &mut self,
        diagnostics: &mut Diagnostics,
        reason: VisitReason,
        section: &ParameterMetadataSection,
    ) {
    }

    /// Visits a metadata object in a metadata or parameter metadata section.
    fn metadata_object(
        &mut self,
        diagnostics: &mut Diagnostics,
        reason: VisitReason,
        object: &MetadataObject,
    ) {
    }

    /// Visits a metadata object item in a metadata object.
    fn metadata_object_item(
        &mut self,
        diagnostics: &mut Diagnostics,
        reason: VisitReason,
        item: &MetadataObjectItem,
    ) {
    }

    /// Visits a metadata array node in a metadata or parameter metadata
    /// section.
    fn metadata_array(
        &mut self,
        diagnostics: &mut Diagnostics,
        reason: VisitReason,
        item: &MetadataArray,
    ) {
    }

    /// Visits an unbound declaration node.
    fn unbound_decl(
        &mut self,
        diagnostics: &mut Diagnostics,
        reason: VisitReason,
        decl: &UnboundDecl,
    ) {
    }

    /// Visits a bound declaration node.
    fn bound_decl(&mut self, diagnostics: &mut Diagnostics, reason: VisitReason, decl: &BoundDecl) {
    }

    /// Visits an expression node.
    fn expr(&mut self, diagnostics: &mut Diagnostics, reason: VisitReason, expr: &Expr) {}

    /// Visits a string text token in a literal string node.
    fn string_text(&mut self, diagnostics: &mut Diagnostics, text: &StringText) {}

    /// Visits a placeholder node.
    fn placeholder(
        &mut self,
        diagnostics: &mut Diagnostics,
        reason: VisitReason,
        placeholder: &Placeholder,
    ) {
    }

    /// Visits a conditional statement node in a workflow.
    fn conditional_statement(
        &mut self,
        diagnostics: &mut Diagnostics,
        reason: VisitReason,
        stmt: &ConditionalStatement,
    ) {
    }

    /// Visits a scatter statement node in a workflow.
    fn scatter_statement(
        &mut self,
        diagnostics: &mut Diagnostics,
        reason: VisitReason,
        stmt: &ScatterStatement,
    ) {
    }

    /// Visits a call statement node in a workflow.
    fn call_statement(
        &mut self,
        diagnostics: &mut Diagnostics,
        reason: VisitReason,
        stmt: &CallStatement,
    ) {
    }
}

/// Used to visit each descendant node of the given root in a preorder
/// traversal.
pub(crate) fn visit<V: Visitor>(root: &SyntaxNode, diagnostics: &mut Diagnostics, visitor: &mut V) {
    for event in root.preorder_with_tokens() {
        let (reason, element) = match event {
            WalkEvent::Enter(node) => (VisitReason::Enter, node),
            WalkEvent::Leave(node) => (VisitReason::Exit, node),
        };

        match element.kind() {
            SyntaxKind::RootNode => {
                let document = Document(element.into_node().unwrap());

                let version = document
                    .version_statement()
                    .and_then(|s| s.version().text().parse::<SupportedVersion>().ok())
                    .expect("only WDL documents with supported versions can be visited");

                visitor.document(diagnostics, reason, &document, version)
            }
            SyntaxKind::VersionStatementNode => visitor.version_statement(
                diagnostics,
                reason,
                &VersionStatement(element.into_node().unwrap()),
            ),
            SyntaxKind::ImportStatementNode => visitor.import_statement(
                diagnostics,
                reason,
                &ImportStatement(element.into_node().unwrap()),
            ),
            SyntaxKind::ImportAliasNode => {
                // Skip these nodes as they're part of an import statement
            }
            SyntaxKind::StructDefinitionNode => visitor.struct_definition(
                diagnostics,
                reason,
                &StructDefinition(element.into_node().unwrap()),
            ),
            SyntaxKind::TaskDefinitionNode => visitor.task_definition(
                diagnostics,
                reason,
                &TaskDefinition(element.into_node().unwrap()),
            ),
            SyntaxKind::WorkflowDefinitionNode => visitor.workflow_definition(
                diagnostics,
                reason,
                &WorkflowDefinition(element.into_node().unwrap()),
            ),
            SyntaxKind::UnboundDeclNode => visitor.unbound_decl(
                diagnostics,
                reason,
                &UnboundDecl(element.into_node().unwrap()),
            ),
            SyntaxKind::BoundDeclNode => visitor.bound_decl(
                diagnostics,
                reason,
                &BoundDecl(element.into_node().unwrap()),
            ),
            SyntaxKind::PrimitiveTypeNode
            | SyntaxKind::MapTypeNode
            | SyntaxKind::ArrayTypeNode
            | SyntaxKind::PairTypeNode
            | SyntaxKind::ObjectTypeNode
            | SyntaxKind::TypeRefNode => {
                // Skip these nodes as they're part of declarations
            }
            SyntaxKind::InputSectionNode => visitor.input_section(
                diagnostics,
                reason,
                &InputSection(element.into_node().unwrap()),
            ),
            SyntaxKind::OutputSectionNode => visitor.output_section(
                diagnostics,
                reason,
                &OutputSection(element.into_node().unwrap()),
            ),
            SyntaxKind::CommandSectionNode => visitor.command_section(
                diagnostics,
                reason,
                &CommandSection(element.into_node().unwrap()),
            ),
            SyntaxKind::RequirementsSectionNode => visitor.requirements_section(
                diagnostics,
                reason,
                &RequirementsSection(element.into_node().unwrap()),
            ),
            SyntaxKind::TaskHintsSectionNode => visitor.task_hints_section(
                diagnostics,
                reason,
                &TaskHintsSection(element.into_node().unwrap()),
            ),
            SyntaxKind::WorkflowHintsSectionNode => visitor.workflow_hints_section(
                diagnostics,
                reason,
                &WorkflowHintsSection(element.into_node().unwrap()),
            ),
            SyntaxKind::TaskHintsItemNode | SyntaxKind::WorkflowHintsItemNode => {
                // Skip this node as it's part of a hints section
            }
            SyntaxKind::RequirementsItemNode => {
                // Skip this node as it's part of a requirements section
            }
            SyntaxKind::RuntimeSectionNode => visitor.runtime_section(
                diagnostics,
                reason,
                &RuntimeSection(element.into_node().unwrap()),
            ),
            SyntaxKind::RuntimeItemNode => visitor.runtime_item(
                diagnostics,
                reason,
                &RuntimeItem(element.into_node().unwrap()),
            ),
            SyntaxKind::MetadataSectionNode => visitor.metadata_section(
                diagnostics,
                reason,
                &MetadataSection(element.into_node().unwrap()),
            ),
            SyntaxKind::ParameterMetadataSectionNode => visitor.parameter_metadata_section(
                diagnostics,
                reason,
                &ParameterMetadataSection(element.into_node().unwrap()),
            ),
            SyntaxKind::MetadataObjectNode => visitor.metadata_object(
                diagnostics,
                reason,
                &MetadataObject(element.into_node().unwrap()),
            ),
            SyntaxKind::MetadataObjectItemNode => visitor.metadata_object_item(
                diagnostics,
                reason,
                &MetadataObjectItem(element.into_node().unwrap()),
            ),
            SyntaxKind::MetadataArrayNode => visitor.metadata_array(
                diagnostics,
                reason,
                &MetadataArray(element.into_node().unwrap()),
            ),
            SyntaxKind::LiteralNullNode => {
                // Skip these nodes as they're part of a metadata section
            }
            k if Expr::<SyntaxNode>::can_cast(k) => {
                visitor.expr(
                    diagnostics,
                    reason,
                    &Expr::cast(element.into_node().expect(
                        "any element that is able to be turned into an expr should be a node",
                    ))
                    .expect("expr should be built"),
                )
            }
            SyntaxKind::LiteralMapItemNode
            | SyntaxKind::LiteralObjectItemNode
            | SyntaxKind::LiteralStructItemNode
            | SyntaxKind::LiteralHintsItemNode
            | SyntaxKind::LiteralInputItemNode
            | SyntaxKind::LiteralOutputItemNode => {
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
            | SyntaxKind::LiteralHintsNode
            | SyntaxKind::LiteralInputNode
            | SyntaxKind::LiteralOutputNode
            | SyntaxKind::ParenthesizedExprNode
            | SyntaxKind::NameRefExprNode
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
            SyntaxKind::PlaceholderNode => visitor.placeholder(
                diagnostics,
                reason,
                &Placeholder(element.into_node().unwrap()),
            ),
            SyntaxKind::PlaceholderSepOptionNode
            | SyntaxKind::PlaceholderDefaultOptionNode
            | SyntaxKind::PlaceholderTrueFalseOptionNode => {
                // Skip these nodes as they're part of a placeholder
            }
            SyntaxKind::ConditionalStatementNode => visitor.conditional_statement(
                diagnostics,
                reason,
                &ConditionalStatement(element.into_node().unwrap()),
            ),
            SyntaxKind::ScatterStatementNode => visitor.scatter_statement(
                diagnostics,
                reason,
                &ScatterStatement(element.into_node().unwrap()),
            ),
            SyntaxKind::CallStatementNode => visitor.call_statement(
                diagnostics,
                reason,
                &CallStatement(element.into_node().unwrap()),
            ),
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
                visitor.whitespace(diagnostics, &Whitespace(element.into_token().unwrap()))
            }
            SyntaxKind::Comment if reason == VisitReason::Enter => {
                visitor.comment(diagnostics, &Comment(element.into_token().unwrap()))
            }
            SyntaxKind::LiteralStringText if reason == VisitReason::Enter => {
                visitor.string_text(diagnostics, &StringText(element.into_token().unwrap()))
            }
            SyntaxKind::LiteralCommandText if reason == VisitReason::Enter => {
                visitor.command_text(diagnostics, &CommandText(element.into_token().unwrap()))
            }
            _ => {
                // Skip remaining tokens
            }
        }
    }
}

impl Document<SyntaxNode> {
    /// Visits the document with a pre-order traversal using the provided
    /// visitor to visit each element in the document.
    pub fn visit<V: Visitor>(&self, diagnostics: &mut Diagnostics, visitor: &mut V) {
        visit(&self.0, diagnostics, visitor)
    }
}
