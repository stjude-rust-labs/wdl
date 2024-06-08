//! A lint rule for ensuring tasks, workflows, and variables are named using
//! snake_case.

use std::fmt;

use convert_case::Boundary;
use convert_case::Case;
use convert_case::Converter;
use wdl_ast::experimental::v1::BoundDecl;
use wdl_ast::experimental::v1::TaskDefinition;
use wdl_ast::experimental::v1::UnboundDecl;
use wdl_ast::experimental::v1::Visitor;
use wdl_ast::experimental::v1::WorkflowDefinition;
use wdl_ast::experimental::AstToken;
use wdl_ast::experimental::Diagnostic;
use wdl_ast::experimental::Diagnostics;
use wdl_ast::experimental::Span;
use wdl_ast::experimental::VisitReason;

use super::Rule;
use crate::Tag;
use crate::TagSet;

/// Represents context of an warning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Context {
    /// The warning occurred in a task.
    Task,
    /// The warning occurred in a workflow.
    Workflow,
    /// The warning occurred in a variable.
    Variable,
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Task => write!(f, "task"),
            Self::Workflow => write!(f, "workflow"),
            Self::Variable => write!(f, "variable"),
        }
    }
}

/// The identifier for the snake_case rule.
const ID: &str = "SnakeCase";

/// Creates a "snake case" diagnostic.
fn snake_case(context: Context, name: &str, properly_cased_name: &str, span: Span) -> Diagnostic {
    Diagnostic::warning(format!("{context} identifier `{name}` is not snake_case"))
        .with_rule(ID)
        .with_label("this identifier must be snake_case", span)
        .with_fix(format!("replace `{name}` with `{properly_cased_name}`"))
}

/// Converts the given identifier to snake case.
fn convert_to_snake_case(name: &str) -> String {
    let converter = Converter::new()
        .remove_boundaries(&[Boundary::DigitLower, Boundary::LowerDigit])
        .to_case(Case::Snake);
    converter.convert(name)
}

/// Detects non-snake_cased identifiers.
#[derive(Debug, Clone, Copy)]
pub struct SnakeCaseRule;

impl Rule for SnakeCaseRule {
    fn id(&self) -> &'static str {
        ID
    }

    fn description(&self) -> &'static str {
        "Ensures that tasks, workflows, and variables use snake_case."
    }

    fn explanation(&self) -> &'static str {
        "Workflow, task, and variable names should be in snake case. Maintaining a consistent \
         naming convention makes the code easier to read and understand."
    }

    fn tags(&self) -> TagSet {
        TagSet::new(&[Tag::Naming, Tag::Style, Tag::Clarity])
    }

    fn visitor(&self) -> Box<dyn Visitor<State = Diagnostics>> {
        Box::new(SnakeCaseVisitor)
    }
}

/// Implements the visitor for the snake case rule.
struct SnakeCaseVisitor;

impl Visitor for SnakeCaseVisitor {
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

        let name = task.name();
        let properly_cased_name = convert_to_snake_case(name.as_str());
        if name.as_str() != properly_cased_name {
            let span = name.span();
            let warning = snake_case(Context::Task, name.as_str(), &properly_cased_name, span);
            state.add(warning);
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

        let name = workflow.name();
        let properly_cased_name = convert_to_snake_case(name.as_str());
        if name.as_str() != properly_cased_name {
            let span = name.span();
            let warning = snake_case(Context::Workflow, name.as_str(), &properly_cased_name, span);
            state.add(warning);
        }
    }

    fn bound_decl(&mut self, state: &mut Self::State, reason: VisitReason, decl: &BoundDecl) {
        if reason == VisitReason::Exit {
            return;
        }

        let name = decl.name();
        let properly_cased_name = convert_to_snake_case(name.as_str());
        if name.as_str() != properly_cased_name {
            let span = name.span();
            let warning = snake_case(Context::Variable, name.as_str(), &properly_cased_name, span);
            state.add(warning);
        }
    }

    fn unbound_decl(&mut self, state: &mut Self::State, reason: VisitReason, decl: &UnboundDecl) {
        if reason == VisitReason::Exit {
            return;
        }

        let name = decl.name();
        let properly_cased_name = convert_to_snake_case(name.as_str());
        if name.as_str() != properly_cased_name {
            let span = name.span();
            let warning = snake_case(Context::Variable, name.as_str(), &properly_cased_name, span);
            state.add(warning);
        }
    }
}
