//! A lint rule for detecting redundant "= None" in optional inputs.

use wdl_ast::v1::{SectionParent, InputDeclaration};
use wdl_ast::{Diagnostic, Diagnostics, SyntaxElement, Span};
use crate::Rule;

const ID: &str = "RedundantNone";

/// Creates a diagnostic warning when an optional input is assigned `= None`.
fn redundant_none_warning(span: Span) -> Diagnostic {
    Diagnostic::warning("Optional inputs do not need to be explicitly assigned to `None`.")
        .with_rule(ID)
        .with_label(
            format!("Remove `= None` from this optional input as it's redundant."),
            span,
        )
}

/// Lint rule struct for detecting redundant `= None`.
#[derive(Default, Debug, Clone, Copy)]
pub struct RedundantNoneRule;

impl Rule for RedundantNoneRule {
    fn id(&self) -> &'static str {
        ID
    }

    fn description(&self) -> &'static str {
        "Detects and warns when an optional input is redundantly assigned `= None`."
    }

    fn explanation(&self) -> &'static str {
        "`String? foo = None` is unnecessary because `String? foo` already means it can be None."
    }

    fn check_input_declaration(
        &mut self,
        parent: &SectionParent,
        input: &InputDeclaration,
        diagnostics: &mut Diagnostics,
    ) {
        if input.is_optional() && input.has_default_none() {
            diagnostics.add(redundant_none_warning(input.span()));
        }
    }
}
