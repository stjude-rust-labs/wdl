//! A lint rule for ensuring no curly commands are used.

use wdl_ast::experimental::v1::TaskDefinition;
use wdl_ast::experimental::v1::Visitor;
use wdl_ast::experimental::AstToken;
use wdl_ast::experimental::Diagnostic;
use wdl_ast::experimental::Diagnostics;
use wdl_ast::experimental::Span;
use wdl_ast::experimental::VisitReason;

use super::Rule;
use crate::Tag;
use crate::TagSet;

/// The identifier for the no curly commands rule.
const ID: &str = "NoCurlyCommands";

/// Creates a "curly commands" diagnostic.
fn curly_commands(task: &str, span: Span) -> Diagnostic {
    Diagnostic::warning(format!(
        "task `{task}` uses curly braces in command section"
    ))
    .with_rule(ID)
    .with_label("this task uses curly braces in the command section", span)
    .with_fix("remove curly braces from the command section")
}

/// Detects curly command section for tasks.
#[derive(Debug, Clone, Copy)]
pub struct NoCurlyCommandsRule;

impl Rule for NoCurlyCommandsRule {
    fn id(&self) -> &'static str {
        ID
    }

    fn description(&self) -> &'static str {
        "Ensures that tasks use heredoc syntax in command sections."
    }

    fn explanation(&self) -> &'static str {
        "Curly command blocks are no longer considered idiomatic WDL. Idiomatic WDL code uses \
         heredoc command blocks instead. This is because curly command blocks create ambiguity \
         with Bash syntax."
    }

    fn tags(&self) -> TagSet {
        TagSet::new(&[Tag::Clarity])
    }

    fn visitor(&self) -> Box<dyn Visitor<State = Diagnostics>> {
        Box::new(NoCurlyCommandsVisitor)
    }
}

/// Implements the visitor for the no curly commands rule.
struct NoCurlyCommandsVisitor;

impl Visitor for NoCurlyCommandsVisitor {
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

        for command in task.commands() {
            if !command.is_heredoc() {
                let name = task.name();
                state.add(curly_commands(name.as_str(), name.span()))
            }
        }
    }
}
