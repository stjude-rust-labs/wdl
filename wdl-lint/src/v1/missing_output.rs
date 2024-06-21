//! A lint rule for missing output sections.

use std::fmt;

use wdl_ast::v1::TaskDefinition;
use wdl_ast::v1::Visitor;
use wdl_ast::v1::WorkflowDefinition;
use wdl_ast::AstToken;
use wdl_ast::Diagnostic;
use wdl_ast::Diagnostics;
use wdl_ast::Span;
use wdl_ast::VisitReason;

use super::Rule;
use crate::Tag;
use crate::TagSet;

/// The context for where the output is missing.
enum Context {
    /// A task.
    Task,
    /// A workflow.
    Workflow,
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Task => write!(f, "task"),
            Self::Workflow => write!(f, "workflow"),
        }
    }
}

/// The identifier for the missing output rule.
const ID: &str = "MissingOutput";

/// Creates a "missing output section" diagnostic.
fn missing_output_section(name: &str, context: Context, span: Span) -> Diagnostic {
    Diagnostic::warning(format!("{context} `{name}` is missing an output section"))
        .with_rule(ID)
        .with_label(format!("this {context} is missing an output section"), span)
        .with_fix(format!("add an output section to the {context}"))
}

/// Detects missing `output` section for tasks and workflows.
#[derive(Debug, Clone, Copy)]
pub struct MissingOutputRule;

impl Rule for MissingOutputRule {
    fn id(&self) -> &'static str {
        ID
    }

    fn description(&self) -> &'static str {
        "Ensures that tasks and workflows have an output section."
    }

    fn explanation(&self) -> &'static str {
        "Some execution engines require an output be defined in order to enable call-caching. When \
         an output is not the result of a successful execution, it is recommended to define a \
         \"dummy\" output to enable call-caching. An example may be `String check = \"passed\"`."
    }

    fn tags(&self) -> TagSet {
        TagSet::new(&[Tag::Completeness, Tag::Portability])
    }

    fn visitor(&self) -> Box<dyn Visitor<State = Diagnostics>> {
        Box::new(MissingOutputVisitor)
    }
}

/// Implements the visitor for the missing output section rule.
struct MissingOutputVisitor;

impl Visitor for MissingOutputVisitor {
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

        if task.outputs().next().is_none() {
            let name = task.name();
            state.add(missing_output_section(
                name.as_str(),
                Context::Task,
                name.span(),
            ));
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

        if workflow.outputs().next().is_none() {
            let name = workflow.name();
            state.add(missing_output_section(
                name.as_str(),
                Context::Workflow,
                name.span(),
            ));
        }
    }
}
