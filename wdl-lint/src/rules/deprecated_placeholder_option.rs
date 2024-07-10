//! A lint rule for flagging placeholder options as deprecated.

use wdl_ast::span_of;
use wdl_ast::v1::Placeholder;
use wdl_ast::v1::PlaceholderOption;
use wdl_ast::version::V1;
use wdl_ast::AstToken as _;
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
pub struct DeprecatedPlaceholderOptionRule {
    /// The detected version of the current document.
    version: Option<SupportedVersion>,
}

impl Rule for DeprecatedPlaceholderOptionRule {
    fn id(&self) -> &'static str {
        ID
    }

    fn description(&self) -> &'static str {
        "Ensures that deprecated expression placeholder options not used."
    }

    fn explanation(&self) -> &'static str {
        "Expression placeholder were deprecated in WDL v1.1. and will be removed in the \
         next major WDL version.

         - `sep` placeholder options should be replaced by the `sep()` standard library \
           function.
         - `true/false` placeholder options should be replaced with `if`/`else` \
            statements.
         - `default` placeholder options should be replaced by the `select_first()` \
            standard library function.

         This rule only evaluates for WDL V1 documents with a version of v1.1 or later, \
         as this reflects when the deprecation was introduced."
    }

    fn tags(&self) -> TagSet {
        TagSet::new(&[Tag::Deprecated])
    }
}

impl Visitor for DeprecatedPlaceholderOptionRule {
    type State = Diagnostics;

    fn document(&mut self, _: &mut Self::State, reason: VisitReason, document: &Document) {
        if reason == VisitReason::Exit {
            return;
        }

        // Reset the visitor upon document entry.
        *self = Default::default();

        // NOTE: this rule is dependent on the version of the WDL document.
        self.version = document
            .version_statement()
            .and_then(|s| s.version().as_str().parse().ok());
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

        let version = match self
            .version
            .expect("document version should be known at this point")
        {
            SupportedVersion::V1(v) => v,
            // NOTE: this rule is not supported for non-V1 WDL documents.
            _ => return,
        };

        if version < V1::One {
            // NOTE: this rule does not support any WDL V1 documents before
            // WDL v1.1.
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
