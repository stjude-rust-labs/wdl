//! Validation of unique names in a V1 AST.

use std::collections::HashMap;
use std::fmt;

use wdl_grammar::Diagnostic;
use wdl_grammar::Span;

use crate::v1::BoundDecl;
use crate::v1::ImportStatement;
use crate::v1::ScatterStatement;
use crate::v1::StructDefinition;
use crate::v1::TaskDefinition;
use crate::v1::UnboundDecl;
use crate::v1::Visitor;
use crate::v1::WorkflowDefinition;
use crate::AstToken;
use crate::Diagnostics;
use crate::Ident;
use crate::VisitReason;

/// Represents context about a unique name validation error.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Context {
    /// The error is a task name.
    Task,
    /// The error is a struct name.
    Struct,
    /// The error is a struct member name.
    StructMember,
    /// The error is a declaration name.
    Declaration,
}

impl fmt::Display for Context {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Task => write!(f, "task"),
            Self::Struct => write!(f, "struct"),
            Self::StructMember => write!(f, "struct member"),
            Self::Declaration => write!(f, "declaration"),
        }
    }
}

/// Creates a "name conflict" diagnostic
fn name_conflict(context: Context, name: Ident, first: Span) -> Diagnostic {
    Diagnostic::error(format!(
        "conflicting {context} name `{name}`",
        name = name.as_str(),
    ))
    .with_label(
        format!("this conflicts with a previous {context} of the same name"),
        name.span(),
    )
    .with_label(format!("first {context} with this name is here"), first)
}

/// A visitor of unique names within an AST.
///
/// Ensures that the following names are unique:
///
/// * Task names.
/// * Struct names from struct declarations and import aliases.
/// * Struct member names.
/// * Declarations and scatter variable names.
///
/// Note that it does not check for duplicate workflow names as it's already a
/// validation error to have more than one workflow in a document.
#[derive(Debug, Default)]
pub struct UniqueNamesVisitor {
    /// A map of task names to the span of the first name.
    tasks: HashMap<String, Span>,
    /// A map of struct names to the span of the first name.
    structs: HashMap<String, Span>,
    /// A map of decl names to span of the first name and whether or not a
    /// scatter variable introduced the name.
    ///
    /// This map is cleared upon entry to a workflow, task, or struct.
    decls: HashMap<String, (Span, bool)>,
    /// Whether or not we're inside a struct definition.
    inside_struct: bool,
}

impl Visitor for UniqueNamesVisitor {
    type State = Diagnostics;

    fn import_statement(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        stmt: &ImportStatement,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        for alias in stmt.aliases() {
            let (_, name) = alias.names();
            if let Some(first) = self.structs.get(name.as_str()) {
                state.add(name_conflict(Context::Struct, name, *first));
            } else {
                self.structs.insert(name.as_str().to_string(), name.span());
            }
        }
    }

    fn workflow_definition(
        &mut self,
        _: &mut Self::State,
        reason: VisitReason,
        _: &WorkflowDefinition,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        self.decls.clear();
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

        self.decls.clear();

        let name = task.name();
        if let Some(first) = self.tasks.get(name.as_str()) {
            state.add(name_conflict(Context::Task, name, *first));
        } else {
            self.tasks.insert(name.as_str().to_string(), name.span());
        }
    }

    fn struct_definition(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        def: &StructDefinition,
    ) {
        if reason == VisitReason::Exit {
            self.inside_struct = false;
            return;
        }

        self.inside_struct = true;
        self.decls.clear();

        let name = def.name();
        if let Some(first) = self.structs.get(name.as_str()) {
            state.add(name_conflict(Context::Struct, name, *first));
        } else {
            self.structs.insert(name.as_str().to_string(), name.span());
        }
    }

    fn bound_decl(&mut self, state: &mut Self::State, reason: VisitReason, decl: &BoundDecl) {
        if reason == VisitReason::Exit {
            return;
        }

        let name = decl.name();
        let span = name.span();
        if let Some((first, scatter)) = self.decls.get_mut(name.as_str()) {
            state.add(name_conflict(Context::Declaration, name, *first));

            // If the name came from a scatter variable, "promote" this declaration as the
            // source of any additional conflicts.
            if *scatter {
                *first = span;
                *scatter = false;
            }
        } else {
            self.decls.insert(name.as_str().to_string(), (span, false));
        }
    }

    fn unbound_decl(&mut self, state: &mut Self::State, reason: VisitReason, decl: &UnboundDecl) {
        if reason == VisitReason::Exit {
            return;
        }

        let name = decl.name();
        let span = name.span();
        if let Some((first, scatter)) = self.decls.get_mut(name.as_str()) {
            state.add(name_conflict(
                if self.inside_struct {
                    Context::StructMember
                } else {
                    Context::Declaration
                },
                name,
                *first,
            ));

            // If the name came from a scatter variable, "promote" this declaration as the
            // source of any additional conflicts.
            if *scatter {
                *first = span;
                *scatter = false;
            }
        } else {
            self.decls.insert(name.as_str().to_string(), (span, false));
        }
    }

    fn scatter_statement(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        stmt: &ScatterStatement,
    ) {
        let name = stmt.variable();
        if reason == VisitReason::Exit {
            // Check to see if this scatter statement introduced the name
            // If so, remove it from the set
            if name.span() == self.decls[name.as_str()].0 {
                self.decls.remove(name.as_str());
            }

            return;
        }

        if let Some((first, _)) = self.decls.get(name.as_str()) {
            state.add(name_conflict(Context::Declaration, name, *first));
        } else {
            self.decls
                .insert(name.as_str().to_string(), (name.span(), true));
        }
    }
}
