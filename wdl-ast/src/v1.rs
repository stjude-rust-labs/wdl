//! AST representation for a 1.x WDL document.

use crate::AstChildren;
use crate::AstNode;
use crate::SyntaxKind;
use crate::SyntaxNode;
use crate::WorkflowDescriptionLanguage;
use crate::support::children;

mod decls;
mod expr;
mod import;
mod r#struct;
mod task;
mod workflow;

pub use decls::*;
pub use expr::*;
pub use import::*;
pub use r#struct::*;
pub use task::*;
pub use workflow::*;

/// Represents a WDL V1 Abstract Syntax Tree (AST).
///
/// The AST is a facade over a [SyntaxTree][1].
///
/// A syntax tree is comprised of nodes that have either
/// other nodes or tokens as children.
///
/// A token is a span of text from the WDL source text and
/// is terminal in the tree.
///
/// Elements of an AST are trivially cloned.
///
/// [1]: crate::SyntaxTree
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ast(SyntaxNode);

impl Ast {
    /// Gets all of the document items in the AST.
    pub fn items(&self) -> AstChildren<DocumentItem> {
        children(&self.0)
    }

    /// Gets the import statements in the AST.
    pub fn imports(&self) -> AstChildren<ImportStatement> {
        children(&self.0)
    }

    /// Gets the struct definitions in the AST.
    pub fn structs(&self) -> AstChildren<StructDefinition> {
        children(&self.0)
    }

    /// Gets the task definitions in the AST.
    pub fn tasks(&self) -> AstChildren<TaskDefinition> {
        children(&self.0)
    }

    /// Gets the workflow definitions in the AST.
    pub fn workflows(&self) -> AstChildren<WorkflowDefinition> {
        children(&self.0)
    }
}

impl AstNode for Ast {
    type Language = WorkflowDescriptionLanguage;

    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized,
    {
        kind == SyntaxKind::RootNode
    }

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        match syntax.kind() {
            SyntaxKind::RootNode => Some(Self(syntax)),
            _ => None,
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        &self.0
    }
}

/// Represents a document item.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum DocumentItem {
    /// The item is an import statement.
    Import(ImportStatement),
    /// The item is a struct definition.
    Struct(StructDefinition),
    /// The item is a task definition.
    Task(TaskDefinition),
    /// The item is a workflow definition.
    Workflow(WorkflowDefinition),
}

impl AstNode for DocumentItem {
    type Language = WorkflowDescriptionLanguage;

    fn can_cast(kind: SyntaxKind) -> bool
    where
        Self: Sized,
    {
        matches!(
            kind,
            SyntaxKind::ImportStatementNode
                | SyntaxKind::StructDefinitionNode
                | SyntaxKind::TaskDefinitionNode
                | SyntaxKind::WorkflowDefinitionNode
        )
    }

    fn cast(syntax: SyntaxNode) -> Option<Self>
    where
        Self: Sized,
    {
        match syntax.kind() {
            SyntaxKind::ImportStatementNode => Some(Self::Import(ImportStatement(syntax))),
            SyntaxKind::StructDefinitionNode => Some(Self::Struct(StructDefinition(syntax))),
            SyntaxKind::TaskDefinitionNode => Some(Self::Task(TaskDefinition(syntax))),
            SyntaxKind::WorkflowDefinitionNode => Some(Self::Workflow(WorkflowDefinition(syntax))),
            _ => None,
        }
    }

    fn syntax(&self) -> &SyntaxNode {
        match self {
            Self::Import(i) => &i.0,
            Self::Struct(s) => &s.0,
            Self::Task(t) => &t.0,
            Self::Workflow(w) => &w.0,
        }
    }
}
