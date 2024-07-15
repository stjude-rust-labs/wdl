//! A lint rule to ensure each output is documented in `meta`.

use indexmap::IndexMap;
use wdl_ast::v1::MetadataSection;
use wdl_ast::v1::MetadataValue;
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
fn nonmatching_output(span: Span, name: &str, item_name: &str, ty: &str) -> Diagnostic {
    Diagnostic::warning(format!(
        "output `{name}` is missing from `meta.outputs` section in {ty} `{item_name}`"
    ))
    .with_rule(ID)
    .with_highlight(span)
    .with_fix(format!(
        "add a description of output `{name}` to documentation in `meta.outputs`"
    ))
}

/// Creates a missing outputs in meta diagnostic.
fn missing_outputs_in_meta(span: Span, item_name: &str, ty: &str) -> Diagnostic {
    Diagnostic::warning(format!(
        "`outputs` key missing in `meta` section for the {ty} `{item_name}`"
    ))
    .with_rule(ID)
    .with_highlight(span)
    .with_fix("add an `outputs` key to `meta` section describing the outputs")
}

/// Creates a diagnostic for extra `meta.outputs` entries.
fn extra_output_in_meta(span: Span, name: &str, item_name: &str, ty: &str) -> Diagnostic {
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
fn out_of_order(span: Span, output_span: Span, item_name: &str, ty: &str) -> Diagnostic {
    Diagnostic::warning(format!(
        "`outputs` section of `meta` for the {ty} `{item_name}` is out of order"
    ))
    .with_rule(ID)
    .with_highlight(span)
    .with_highlight(output_span)
    .with_fix(
        "ensure the keys within `meta.outputs` have the same order as they appear in `output`",
    )
}

/// Creates a diagnostic for non-object `meta.outputs` entries.
fn non_object_meta_outputs(span: Span, item_name: &str, ty: &str) -> Diagnostic {
    Diagnostic::warning(format!(
        "`outputs` key in `meta` section for the {ty} `{item_name}` is not an object"
    ))
    .with_rule(ID)
    .with_highlight(span)
    .with_fix("ensure `meta.outputs` is an object containing descriptions for each output")
}

/// Detects non-matching outputs.
#[derive(Default, Debug, Clone)]
pub struct NonmatchingOutputRule<'a> {
    /// The span of the `meta` section.
    current_meta_span: Option<Span>,
    /// The span of the `meta.outputs` section.
    current_meta_outputs_span: Option<Span>,
    /// The span of the `output` section.
    current_output_span: Option<Span>,
    /// The keys seen in `meta.outputs`.
    meta_outputs_keys: IndexMap<String, Span>,
    /// The keys seen in `output`.
    output_keys: IndexMap<String, Span>,
    /// The context type.
    ty: Option<&'a str>,
    /// The item name.
    name: Option<String>,
    /// Prior objects
    prior_objects: Vec<String>,
}

impl<'a> Rule for NonmatchingOutputRule<'a> {
    fn id(&self) -> &'static str {
        ID
    }

    fn description(&self) -> &'static str {
        "Ensures that each output field is documented in the meta section under `meta.outputs`."
    }

    fn explanation(&self) -> &'static str {
        "The meta section should have an `outputs` key and keys with descriptions for each output \
         of the task/workflow. These must match exactly. i.e. for each named output of a task or \
         workflow, there should be an entry under `meta.outputs` with that same name. \
         Additionally, these entries should be in the same order (that order is up to the \
         developer to decide). No extraneous `meta.outputs` entries are allowed."
    }

    fn tags(&self) -> TagSet {
        TagSet::new(&[Tag::Completeness])
    }
}

/// Check each output key exists in the `outputs` key within the `meta` section.
fn check_matching(state: &mut Diagnostics, rule: &mut NonmatchingOutputRule<'_>) {
    let mut exact_match = true;
    // Check for expected entries missing from `meta.outputs`.
    for (name, span) in &rule.output_keys {
        if !rule.meta_outputs_keys.contains_key(name) {
            exact_match = false;
            if rule.current_meta_span.is_some() {
                state.add(nonmatching_output(
                    *span,
                    name,
                    rule.name.as_deref().expect("should have a name"),
                    rule.ty.expect("should have a type"),
                ));
            }
        }
    }

    // Check for extra entries in `meta.outputs`.
    for (name, span) in &rule.meta_outputs_keys {
        if !rule.output_keys.contains_key(name) {
            exact_match = false;
            if rule.current_output_span.is_some() {
                state.add(extra_output_in_meta(
                    *span,
                    name,
                    rule.name.as_deref().expect("should have a name"),
                    rule.ty.expect("should have a type"),
                ));
            }
        }
    }

    // Check for out-of-order entries.
    if exact_match && !rule.meta_outputs_keys.keys().eq(rule.output_keys.keys()) {
        state.add(out_of_order(
            rule.current_meta_outputs_span
                .expect("should have a `meta.outputs` span"),
            rule.current_output_span
                .expect("should have an `output` span"),
            rule.name.as_deref().expect("should have a name"),
            rule.ty.expect("should have a type"),
        ));
    }
}

impl<'a> Visitor for NonmatchingOutputRule<'a> {
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
        match reason {
            VisitReason::Enter => {
                self.name = Some(workflow.name().as_str().to_string());
                self.ty = Some("workflow");
            }
            VisitReason::Exit => {
                if self.current_meta_span.is_some()
                    && self.current_meta_outputs_span.is_none()
                    && !self.output_keys.is_empty()
                {
                    state.add(missing_outputs_in_meta(
                        self.current_meta_span.expect("should have a `meta` span"),
                        self.name.as_deref().expect("should have a name"),
                        self.ty.expect("should have a type"),
                    ));
                } else {
                    check_matching(state, self);
                }

                self.name = None;
                self.current_meta_outputs_span = None;
                self.current_meta_span = None;
                self.current_output_span = None;
                self.output_keys.clear();
                self.meta_outputs_keys.clear();
            }
        }
    }

    fn task_definition(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        task: &TaskDefinition,
    ) {
        match reason {
            VisitReason::Enter => {
                self.name = Some(task.name().as_str().to_string());
                self.ty = Some("task");
            }
            VisitReason::Exit => {
                if self.current_meta_span.is_some()
                    && self.current_meta_outputs_span.is_none()
                    && !self.output_keys.is_empty()
                {
                    state.add(missing_outputs_in_meta(
                        self.current_meta_span.expect("should have a `meta` span"),
                        self.name.as_deref().expect("should have a name"),
                        self.ty.expect("should have a type"),
                    ));
                } else {
                    check_matching(state, self);
                }

                self.current_meta_outputs_span = None;
                self.current_meta_span = None;
                self.current_output_span = None;
                self.output_keys.clear();
                self.meta_outputs_keys.clear();
            }
        }
    }

    fn metadata_section(
        &mut self,
        _state: &mut Self::State,
        reason: VisitReason,
        section: &MetadataSection,
    ) {
        match reason {
            VisitReason::Enter => {
                self.current_meta_span = Some(section.syntax().text_range().to_span());
            }
            VisitReason::Exit => {}
        }
    }

    fn output_section(
        &mut self,
        _state: &mut Self::State,
        reason: VisitReason,
        section: &OutputSection,
    ) {
        if reason == VisitReason::Enter {
            self.current_output_span = Some(section.syntax().text_range().to_span());
        }
    }

    fn bound_decl(
        &mut self,
        _state: &mut Self::State,
        reason: VisitReason,
        decl: &wdl_ast::v1::BoundDecl,
    ) {
        if reason == VisitReason::Enter {
            if let Some(output) = &self.current_output_span {
                let decl_span = decl.syntax().text_range().to_span();
                if decl_span.start() > output.start() && decl_span.end() < output.end() {
                    self.output_keys.insert(
                        decl.name().as_str().to_string(),
                        decl.syntax().text_range().to_span(),
                    );
                }
            }
        }
    }

    fn metadata_object_item(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        item: &wdl_ast::v1::MetadataObjectItem,
    ) {
        match reason {
            VisitReason::Exit => {
                if let MetadataValue::Object(_) = item.value() {
                    self.prior_objects.pop();
                }
            }
            VisitReason::Enter => {
                if let Some(_meta_span) = self.current_meta_span {
                    if item.name().as_str() == "outputs" {
                        self.current_meta_outputs_span = Some(item.syntax().text_range().to_span());
                        match item.value() {
                            MetadataValue::Object(_) => {}
                            _ => {
                                state.add(non_object_meta_outputs(
                                    item.syntax().text_range().to_span(),
                                    self.name.as_deref().expect("should have a name"),
                                    self.ty.expect("should have a type"),
                                ));
                            }
                        }
                    } else if let Some(meta_outputs_span) = self.current_meta_outputs_span {
                        let span = item.syntax().text_range().to_span();
                        if span.start() > meta_outputs_span.start()
                            && span.end() < meta_outputs_span.end()
                            && self
                                .prior_objects
                                .last()
                                .expect("should have seen `meta.outputs`")
                                == "outputs"
                        {
                            self.meta_outputs_keys.insert(
                                item.name().as_str().to_string(),
                                item.syntax().text_range().to_span(),
                            );
                        }
                    }
                }
                if let MetadataValue::Object(_) = item.value() {
                    self.prior_objects.push(item.name().as_str().to_string());
                }
            }
        }
    }
}
