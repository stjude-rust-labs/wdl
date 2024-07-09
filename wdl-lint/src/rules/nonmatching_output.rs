//! A lint rule to ensure each output is documented in `meta`.

use wdl_ast::v1::MetadataSection;
use wdl_ast::v1::OutputSection;
use wdl_ast::v1::TaskDefinition;
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
fn nonmatching_output(span: Span, name: String) -> Diagnostic {
    Diagnostic::warning(format!("output `{name}` is missing from `meta` section"))
        .with_rule(ID)
        .with_highlight(span)
        .with_fix(format!(
            "add output (`{name}`) key to `outputs` documentation in `meta` section"
        ))
}

/// Creates a missing outputs in meta diagnostic.
fn missing_outputs_in_meta(span: Span) -> Diagnostic {
    Diagnostic::warning("`outputs` key missing in `meta` section")
        .with_rule(ID)
        .with_highlight(span)
        .with_fix("add `outputs` key to `meta` section")
}

/// Detects non-matching outputs.
#[derive(Default, Debug, Clone, Copy)]
pub struct NonmatchingOutputRule;

impl Rule for NonmatchingOutputRule {
    fn id(&self) -> &'static str {
        ID
    }

    fn description(&self) -> &'static str {
        "Ensures that each output field is documented in the meta section."
    }

    fn explanation(&self) -> &'static str {
        "The meta section should have an output key and keys with descriptions for each output of \
         the task/workflow. These must match exactly. i.e. for each named output of a task or \
         workflow, there should be an entry under meta.output with that same name. Additionally, \
         these entries should be in the same order (that order is up to the developer to decide). \
         No extraneous output entries are allowed. There should not be any blank lines inside the \
         entire meta section."
    }

    fn tags(&self) -> TagSet {
        TagSet::new(&[Tag::Spacing, Tag::Style])
    }
}

/// Check each output key exists in the outputs key within the `meta` section.
fn check_output_meta(state: &mut Diagnostics, meta: &MetadataSection, outputs: &OutputSection) {
    // Get the output section from the meta section.
    if let Some(meta_outputs_key) = meta
        .items()
        .find(|entry| entry.name().syntax().to_string() == "outputs")
    {
        let meta_outputs: Vec<_> = meta_outputs_key
            .value()
            .unwrap_object()
            .items()
            .map(|entry| entry.name().syntax().to_string())
            .collect();

        // Get the declared outputs.
        outputs.declarations().for_each(|o| {
            let name = o.name().as_str().to_string();
            if !meta_outputs.contains(&name) {
                state.add(nonmatching_output(
                    o.name().span(),
                    o.name().as_str().to_string(),
                ));
            }
        });
    } else {
        state.add(missing_outputs_in_meta(
            meta.syntax().first_token().unwrap().text_range().to_span(),
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
                    check_output_meta(state, &meta, &output);
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
                    check_output_meta(state, &meta, &output);
                }
            }
        }
    }
}
