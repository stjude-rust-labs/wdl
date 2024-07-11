//! A lint rule to ensure each output is documented in `meta`.

use indexmap::IndexMap;
use wdl_ast::v1::MetadataSection;
use wdl_ast::v1::OutputSection;
use wdl_ast::v1::TaskDefinition;
use wdl_ast::v1::TaskOrWorkflow;
use wdl_ast::v1::WorkflowDefinition;
use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::Diagnostic;
use wdl_ast::Diagnostics;
use wdl_ast::Document;
use wdl_ast::Span;
use wdl_ast::ToSpan;
use wdl_ast::VisitReason;
use wdl_ast::Visitor;

use crate::Rule;
use crate::Tag;
use crate::TagSet;

/// The identifier for the non-matching output rule.
const ID: &str = "NonmatchingOutput";

/// Creates a "non-matching output" diagnostic.
fn nonmatching_output(span: Span, name: &str, context: &TaskOrWorkflow) -> Diagnostic {
    let (ty, item_name) = match context {
        TaskOrWorkflow::Task(t) => ("task", t.name().as_str().to_string()),
        TaskOrWorkflow::Workflow(w) => ("workflow", w.name().as_str().to_string()),
    };

    Diagnostic::warning(format!(
        "output `{name}` is missing from `meta` section in {ty} `{item_name}`"
    ))
    .with_rule(ID)
    .with_highlight(span)
    .with_fix(format!(
        "add output (`{name}`) key to `outputs` documentation in `meta` section"
    ))
}

/// Creates a missing outputs in meta diagnostic.
fn missing_outputs_in_meta(span: Span, context: &TaskOrWorkflow) -> Diagnostic {
    let (ty, name) = match context {
        TaskOrWorkflow::Task(t) => ("task", t.name().as_str().to_string()),
        TaskOrWorkflow::Workflow(w) => ("workflow", w.name().as_str().to_string()),
    };

    Diagnostic::warning(format!(
        "`outputs` key missing in `meta` section for the {ty} `{name}`"
    ))
    .with_rule(ID)
    .with_highlight(span)
    .with_fix("add an `outputs` key to `meta` section describing the outputs")
}

/// Creates a diagnostic for extra `meta.outputs` entries.
fn extra_output_in_meta(span: Span, name: &str, context: &TaskOrWorkflow) -> Diagnostic {
    let (ty, item_name) = match context {
        TaskOrWorkflow::Task(t) => ("task", t.name().as_str().to_string()),
        TaskOrWorkflow::Workflow(w) => ("workflow", w.name().as_str().to_string()),
    };

    Diagnostic::warning(format!(
        "`{name}` appears in `outputs` section of the {ty} `{item_name}` but is not a declared \
         `output`"
    ))
    .with_rule(ID)
    .with_highlight(span)
    .with_fix(format!(
        "ensure the output exists or remove the `{name}` key from `meta.outputs`"
    ))
}

/// Creates a diagnostic for out-of-order entries.
fn out_of_order(span: Span, output_span: Span, context: &TaskOrWorkflow) -> Diagnostic {
    let (ty, item_name) = match context {
        TaskOrWorkflow::Task(t) => ("task", t.name().as_str().to_string()),
        TaskOrWorkflow::Workflow(w) => ("workflow", w.name().as_str().to_string()),
    };

    Diagnostic::warning(format!(
        "`outputs` section of `meta` for the {ty} `{item_name}` is out of order"
    ))
    .with_rule(ID)
    .with_highlight(span)
    .with_highlight(output_span)
    .with_fix("ensure the keys within `meta.outputs` have the same order as they appear in `outputs`")
}

/// Detects non-matching outputs.
#[derive(Default, Debug, Clone, Copy)]
pub struct NonmatchingOutputRule;

impl Rule for NonmatchingOutputRule {
    fn id(&self) -> &'static str {
        ID
    }

    fn description(&self) -> &'static str {
        "Ensures that each output field is documented in the meta section under `meta.outputs`."
    }

    fn explanation(&self) -> &'static str {
        "The meta section should have an `outputs` key and keys with descriptions for each output of \
         the task/workflow. These must match exactly. i.e. for each named output of a task or \
         workflow, there should be an entry under `meta.outputs` with that same name. Additionally, \
         these entries should be in the same order (that order is up to the developer to decide). \
         No extraneous output entries are allowed."
    }

    fn tags(&self) -> TagSet {
        TagSet::new(&[Tag::Completeness])
    }
}

/// Check each output key exists in the outputs key within the `meta` section.
fn check_output_meta(
    state: &mut Diagnostics,
    meta: &MetadataSection,
    outputs: &OutputSection,
    context: TaskOrWorkflow,
) {
    // Get the output section from the meta section.
    if let Some(meta_outputs_key) = meta
        .items()
        .find(|entry| entry.name().syntax().to_string() == "outputs")
    {
        let actual: IndexMap<_, _> = meta_outputs_key
            .value()
            .unwrap_object()
            .items()
            .map(|entry| {
                (
                    entry.name().syntax().to_string(),
                    entry.syntax().text_range().to_span(),
                )
            })
            .collect();

        // Get the declared outputs.
        let expected: IndexMap<_, _> = outputs
            .declarations()
            .map(|d| {
                (
                    d.name().syntax().to_string(),
                    d.syntax().text_range().to_span(),
                )
            })
            .collect();

        let mut extra_found = false;

        // Check for entries missing from `meta`.
        for (name, span) in &expected {
            if !actual.contains_key(name) {
                if !extra_found {
                    extra_found = true;
                }
                state.add(nonmatching_output(*span, name, &context));
            }
        }

        // Check for extra entries in `meta`.
        for (name, span) in &actual {
            if !expected.contains_key(name) {
                if !extra_found {
                    extra_found = true;
                }
                state.add(extra_output_in_meta(*span, name, &context));
            }
        }

        // Check for out-of-order entries.
        if !extra_found && !actual.keys().eq(expected.keys()) {
            state.add(out_of_order(
                meta_outputs_key.syntax().text_range().to_span(),
                outputs.syntax().text_range().to_span(),
                &context,
            ));
        }
    } else {
        state.add(missing_outputs_in_meta(
            meta.syntax().first_token().unwrap().text_range().to_span(),
            &context,
        ));
    }
}

impl Visitor for NonmatchingOutputRule {
    type State = Diagnostics;

    fn document(&mut self, _: &mut Self::State, reason: VisitReason, _: &Document) {
        if reason == VisitReason::Exit {
            return;
        }

        // Reset the visitor upon document entry
        *self = Default::default();
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

        // If the `meta` section is missing, the MissingMeta rule will handle it.
        if let Some(meta) = workflow.metadata().next() {
            // If the `output` section is missing, the MissingOutput rule will handle it.
            if let Some(output) = workflow.outputs().next() {
                if output.declarations().count() > 0 {
                    check_output_meta(
                        state,
                        &meta,
                        &output,
                        TaskOrWorkflow::Workflow(workflow.clone()),
                    );
                }
            }
        }
    }

    fn task_definition(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        task: &TaskDefinition,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        // If the `meta` section is missing, the MissingMeta rule will handle it.
        if let Some(meta) = task.metadata().next() {
            // If the `output` section is missing, the MissingOutput rule will handle it.
            if let Some(output) = task.outputs().next() {
                if output.declarations().count() > 0 {
                    check_output_meta(state, &meta, &output, TaskOrWorkflow::Task(task.clone()));
                }
            }
        }
    }
}
