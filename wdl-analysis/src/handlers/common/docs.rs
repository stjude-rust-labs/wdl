//! Utilities for generating documentation for LSP handlers.

use lsp_types::Documentation;
use lsp_types::MarkupContent;
use rowan::TextSize;
use wdl_ast::AstNode;
use wdl_ast::v1::StructDefinition;
use wdl_ast::v1::TaskDefinition;
use wdl_ast::v1::WorkflowDefinition;

use crate::document::Struct;
use crate::document::Task;
use crate::document::Workflow;

/// Makes a LSP documentation from a definition text.
pub fn make_md_docs(definition: String) -> Option<Documentation> {
    Some(Documentation::MarkupContent(MarkupContent {
        kind: lsp_types::MarkupKind::Markdown,
        value: definition,
    }))
}

/// Provides documentation for tasks which includes `inputs`, `outputs`,
/// `metadata`, `runtime`
pub fn provide_task_documentation(task: &Task, root: &wdl_ast::Document) -> Option<String> {
    match TextSize::try_from(task.name_span().start()) {
        Ok(offset) => root
            .inner()
            .token_at_offset(offset)
            .left_biased()
            .and_then(|t| t.parent_ancestors().find_map(TaskDefinition::cast))
            .as_ref()
            .and_then(|n| {
                let mut s = String::new();
                n.markdown_description(&mut s).ok()?;
                Some(s)
            }),
        Err(_) => None,
    }
}

/// Provides documentation for workflows which includes `inputs`, `outputs`,
/// `metadata`
pub fn provide_workflow_documentation(
    workflow: &Workflow,
    root: &wdl_ast::Document,
) -> Option<String> {
    match TextSize::try_from(workflow.name_span().start()) {
        Ok(offset) => root
            .inner()
            .token_at_offset(offset)
            .left_biased()
            .and_then(|t| t.parent_ancestors().find_map(WorkflowDefinition::cast))
            .as_ref()
            .and_then(|n| {
                let mut s = String::new();
                n.markdown_description(&mut s).ok()?;
                Some(s)
            }),
        Err(_) => None,
    }
}

/// Provides documentation for structs.
pub fn provide_struct_documentation(
    struct_info: &Struct,
    root: &wdl_ast::Document,
) -> Option<String> {
    match TextSize::try_from(struct_info.name_span().start()) {
        Ok(offset) => root
            .inner()
            .token_at_offset(offset)
            .left_biased()
            .and_then(|t| t.parent_ancestors().find_map(StructDefinition::cast))
            .as_ref()
            .and_then(|n| {
                let mut s = String::new();
                n.markdown_description(&mut s).ok()?;
                Some(s)
            }),
        Err(_) => None,
    }
}
