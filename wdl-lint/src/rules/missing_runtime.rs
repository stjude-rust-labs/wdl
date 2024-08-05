//! A lint rule for missing `runtime` sections.

use wdl_ast::v1::TaskDefinition;
use wdl_ast::version::V1;
use wdl_ast::AstToken;
use wdl_ast::Diagnostic;
use wdl_ast::Diagnostics;
use wdl_ast::Document;
use wdl_ast::Span;
use wdl_ast::SupportedVersion;
use wdl_ast::VisitReason;
use wdl_ast::Visitor;

use crate::Rule;
use crate::Tag;
use crate::TagSet;

/// The identifier for the missing runtime rule.
const ID: &str = "MissingRuntime";

/// Creates a "missing runtime section" diagnostic.
fn missing_runtime_section(task: &str, span: Span) -> Diagnostic {
    Diagnostic::warning(format!("task `{task}` is missing a `runtime` section"))
        .with_rule(ID)
        .with_label("this task is missing a `runtime` section", span)
        .with_fix("add a `runtime` section to the task")
}

/// Detects missing `runtime` section for tasks.
#[derive(Default, Debug, Clone, Copy)]
pub struct MissingRuntimeRule(Option<SupportedVersion>);

impl Rule for MissingRuntimeRule {
    fn id(&self) -> &'static str {
        ID
    }

    fn description(&self) -> &'static str {
        "Ensures that tasks have a `runtime` section (for WDL v1.1 and prior)."
    }

    fn explanation(&self) -> &'static str {
        "Tasks that don't declare `runtime` sections are unlikely to be portable."
    }

    fn tags(&self) -> TagSet {
        TagSet::new(&[Tag::Completeness, Tag::Portability])
    }
}

impl Visitor for MissingRuntimeRule {
    type State = Diagnostics;

    fn document(
        &mut self,
        _: &mut Self::State,
        reason: VisitReason,
        _: &Document,
        version: SupportedVersion,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        // Reset the visitor upon document entry
        *self = Self(Some(version));
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

        // This rule should only be present for WDL v1.1 or earlier, as the
        // `requirements` section replaces it in WDL v1.2.
        if let SupportedVersion::V1(minor_version) = self.0.expect("version should exist here") {
            if minor_version <= V1::One && task.runtimes().next().is_none() {
                let name = task.name();
                state.add(missing_runtime_section(name.as_str(), name.span()));
            }
        }
    }
}
