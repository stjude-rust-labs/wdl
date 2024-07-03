//! A lint rule for section ordering.

use wdl_ast::v1::TaskDefinition;
use wdl_ast::v1::WorkflowDefinition;
use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::Diagnostic;
use wdl_ast::Diagnostics;
use wdl_ast::Span;
use wdl_ast::ToSpan;
use wdl_ast::VisitReason;
use wdl_ast::Visitor;

use crate::Rule;
use crate::Tag;
use crate::TagSet;

/// The identifier for the section ordering rule.
const ID: &str = "SectionOrdering";

/// Creates a workflow section order diagnostic.
fn workflow_section_order(span: Span, name: &str) -> Diagnostic {
    Diagnostic::note(format!("sections are not in order for workflow {}", name))
        .with_rule(ID)
        .with_highlight(span)
        .with_fix("order as `meta`, `parameter_meta`, `input`, private declarations, `output`")
}

/// Creates a task section order diagnostic.
fn task_section_order(span: Span, name: &str) -> Diagnostic {
    Diagnostic::note(format!("sections are not in order for task {}", name))
        .with_rule(ID)
        .with_highlight(span)
        .with_fix(
            "order as `meta`, `parameter_meta`, `input`, private declarations, `command`, \
             `output`, `runtime`",
        )
}

/// Detects section ordering issues.
#[derive(Debug, Clone, Copy)]
pub struct SectionOrderingRule;

impl Rule for SectionOrderingRule {
    fn id(&self) -> &'static str {
        ID
    }

    fn description(&self) -> &'static str {
        "Ensures that all sections are in the correct order."
    }

    fn explanation(&self) -> &'static str {
        "For workflows, the following sections must be present and in this order: meta, \
         parameter_meta, input, (body), output. \"(body)\" represents all calls and declarations.

        For tasks, the following sections must be present and in this order: meta, parameter_meta, \
         input, (private declarations), command, output, runtime"
    }

    fn tags(&self) -> TagSet {
        TagSet::new(&[Tag::Style, Tag::Sorting])
    }
}

impl Visitor for SectionOrderingRule {
    type State = Diagnostics;

    fn task_definition(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        task: &TaskDefinition,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        let mut sections: Vec<usize> = Vec::new();

        if let Some(meta) = task.metadata().next() {
            sections.push(meta.syntax().text_range().to_span().start());
        }
        if let Some(parameter_meta) = task.parameter_metadata().next() {
            sections.push(parameter_meta.syntax().text_range().to_span().start());
        }
        if let Some(inputs) = task.inputs().next() {
            sections.push(inputs.syntax().text_range().to_span().start());
        }
        task.declarations().for_each(|f| {
            sections.push(f.syntax().text_range().to_span().start());
        });
        if let Some(command) = task.commands().next() {
            sections.push(command.syntax().text_range().to_span().start());
        }
        if let Some(outputs) = task.outputs().next() {
            sections.push(outputs.syntax().text_range().to_span().start());
        }
        if let Some(runtime) = task.runtimes().next() {
            sections.push(runtime.syntax().text_range().to_span().start());
        }

        let mut sorted_sections: Vec<usize> = sections.clone();
        sorted_sections.sort();

        if sections
            .iter()
            .zip(sorted_sections.iter())
            .any(|(a, b)| a != b)
        {
            state.add(task_section_order(task.name().span(), task.name().as_str()));
        }
    }

    fn workflow_definition(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        workflow: &WorkflowDefinition,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        let mut sections: Vec<usize> = Vec::new();

        if let Some(meta) = workflow.metadata().next() {
            println!("{:?}", meta);
            sections.push(meta.syntax().text_range().to_span().start());
        }
        if let Some(parameter_meta) = workflow.parameter_metadata().next() {
            sections.push(parameter_meta.syntax().text_range().to_span().start());
        }
        if let Some(inputs) = workflow.inputs().next() {
            sections.push(inputs.syntax().text_range().to_span().start());
        }

        // Collect all calls and declarations
        // Ensure they are between inputs and outputs.
        // Internal ordering does not matter.
        let mut calls_and_declarations: Vec<usize> = Vec::new();
        workflow.declarations().for_each(|f| {
            calls_and_declarations.push(f.syntax().text_range().to_span().start());
        });
        workflow.statements().for_each(|f| {
            calls_and_declarations.push(f.syntax().text_range().to_span().start());
        });

        calls_and_declarations.sort();
        sections.append(&mut calls_and_declarations);

        if let Some(outputs) = workflow.outputs().next() {
            sections.push(outputs.syntax().text_range().to_span().start());
        }

        let mut sorted_sections: Vec<usize> = sections.clone();
        sorted_sections.sort();

        if sections
            .iter()
            .zip(sorted_sections.iter())
            .any(|(a, b)| a != b)
        {
            state.add(workflow_section_order(
                workflow.name().span(),
                workflow.name().as_str(),
            ));
        }
    }
}
