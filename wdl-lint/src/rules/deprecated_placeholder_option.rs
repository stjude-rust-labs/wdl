//! A lint rule for flagging placeholder options as deprecated.

use wdl_ast::span_of;
use wdl_ast::v1::Placeholder;
use wdl_ast::v1::PlaceholderOption;
use wdl_ast::Diagnostic;
use wdl_ast::Diagnostics;
use wdl_ast::Document;
use wdl_ast::Span;
use wdl_ast::VisitReason;
use wdl_ast::Visitor;

use crate::Rule;
use crate::Tag;
use crate::TagSet;

/// The identifier for the deprecated placeholder option rule.
const ID: &str = "DeprecatedPlaceholderOption";

/// Creates a diagnostic for the use of the deprecated `default` placeholder
/// option.
fn deprecated_default_placeholder_option(span: Span) -> Diagnostic {
    Diagnostic::warning(String::from(
        "use of the deprecated `default` placeholder option",
    ))
    .with_rule(ID)
    .with_highlight(span)
    .with_fix(
        "replace the `default` placeholder option with a call to the `select_first()` standard \
         library function",
    )
}

/// Creates a diagnostic for the use of the deprecated `sep` placeholder option.
fn deprecated_sep_placeholder_option(span: Span) -> Diagnostic {
    Diagnostic::warning(String::from(
        "use of the deprecated `sep` placeholder option",
    ))
    .with_rule(ID)
    .with_highlight(span)
    .with_fix(
        "replace the `sep` placeholder option with a call to the `sep()` standard library function",
    )
}

/// Creates a diagnostic for the use of the deprecated `true`/`false`
/// placeholder option.
fn deprecated_true_false_placeholder_option(span: Span) -> Diagnostic {
    Diagnostic::warning(String::from(
        "use of the deprecated `true`/`false` placeholder option",
    ))
    .with_rule(ID)
    .with_highlight(span)
    .with_fix("replace the `true`/`false` placeholder option with an `if`/`else` statement")
}

/// Detects the use of a deprecated placeholder option.
#[derive(Debug, Default, Clone, Copy)]
pub struct DeprecatedPlaceholderOptionRule;

impl Rule for DeprecatedPlaceholderOptionRule {
    fn id(&self) -> &'static str {
        ID
    }

    fn description(&self) -> &'static str {
        "Ensures that deprecated expression placeholder options not used."
    }

    fn explanation(&self) -> &'static str {
        "Expression placeholder options are deprecated and will be removed in the next major WDL \
         release. `sep` placeholder options, `true/false` placeholder options, and `default` \
         placeholder options should be replaced by the `sep()` standard library function, \
         `if`/`else` statements, and the `select_first()` 
         standard library function respectively."
    }

    fn tags(&self) -> TagSet {
        TagSet::new(&[Tag::Deprecated])
    }
}

impl Visitor for DeprecatedPlaceholderOptionRule {
    type State = Diagnostics;

    fn document(&mut self, _: &mut Self::State, reason: VisitReason, _: &Document) {
        if reason == VisitReason::Exit {
            return;
        }

        // Reset the visitor upon document entry
        *self = Default::default();
    }

    fn placeholder(
        &mut self,
        state: &mut Self::State,
        reason: VisitReason,
        placeholder: &Placeholder,
    ) {
        if reason == VisitReason::Exit {
            return;
        }

        for option in placeholder.options() {
            match option {
                PlaceholderOption::Sep(option) => {
                    state.add(deprecated_sep_placeholder_option(span_of(&option)));
                }
                PlaceholderOption::Default(option) => {
                    state.add(deprecated_default_placeholder_option(span_of(&option)));
                }
                PlaceholderOption::TrueFalse(option) => {
                    state.add(deprecated_true_false_placeholder_option(span_of(&option)))
                }
            }
        }
    }
}
