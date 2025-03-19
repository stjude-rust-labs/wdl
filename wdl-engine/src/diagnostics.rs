//! Implementation of evaluation diagnostics.

use std::fmt;

use wdl_analysis::types::CallKind;
use wdl_analysis::types::Type;
use wdl_ast::AstToken;
use wdl_ast::Diagnostic;
use wdl_ast::Ident;
use wdl_ast::Span;
use wdl_ast::TreeToken;

/// Creates an "integer not in range" diagnostic.
pub fn integer_not_in_range(span: Span) -> Diagnostic {
    Diagnostic::error(format!(
        "literal integer exceeds the range for a 64-bit signed integer ({min}..={max})",
        min = i64::MIN,
        max = i64::MAX,
    ))
    .with_label("this literal integer is not in range", span)
}

/// Creates an "integer negation not in range" diagnostic.
pub fn integer_negation_not_in_range(value: i64, span: Span) -> Diagnostic {
    Diagnostic::error(format!(
        "negation of integer value {value} exceeds the range for a 64-bit signed integer \
         ({min}..={max})",
        min = i64::MIN,
        max = i64::MAX,
    ))
    .with_highlight(span)
}

/// Creates a "float not in range" diagnostic.
pub fn float_not_in_range(span: Span) -> Diagnostic {
    Diagnostic::error(format!(
        "literal float exceeds the range for a 64-bit float ({min:+e}..={max:+e})",
        min = f64::MIN,
        max = f64::MAX,
    ))
    .with_label("this literal float is not in range", span)
}

/// Creates a "numeric overflow" diagnostic.
pub fn numeric_overflow(span: Span) -> Diagnostic {
    Diagnostic::error("evaluation of arithmetic expression resulted in overflow")
        .with_highlight(span)
}

/// Creates a "division by zero" diagnostic.
pub fn division_by_zero(span: Span, divisor_span: Span) -> Diagnostic {
    Diagnostic::error("attempt to divide by zero")
        .with_highlight(span)
        .with_label("this expression evaluated to zero", divisor_span)
}

/// Creates a "exponent not in range" diagnostic.
pub fn exponent_not_in_range(span: Span) -> Diagnostic {
    Diagnostic::error(format!(
        "exponent exceeds acceptable range ({min}..={max})",
        min = u32::MIN,
        max = u32::MAX,
    ))
    .with_label("this value exceeds the range for an exponent", span)
}

/// Creates a "cannot call" diagnostic.
pub fn cannot_call<T: TreeToken>(target: &Ident<T>) -> Diagnostic {
    Diagnostic::error(format!(
        "function `{target}` can only be called from task outputs",
        target = target.text()
    ))
    .with_highlight(target.span())
}

/// Creates a "call failed" diagnostic.
pub fn call_failed(
    kind: CallKind,
    target: &str,
    document: &str,
    span: Span,
    e: anyhow::Error,
) -> Diagnostic {
    let e = e.context(format!("call to {kind} `{target}` in `{document}` failed"));

    Diagnostic::error(format!("{e:#}")).with_highlight(span)
}

/// Creates a "runtime type mismatch" diagnostic.
pub fn runtime_type_mismatch(
    e: anyhow::Error,
    expected: &Type,
    expected_span: Span,
    actual: &Type,
    actual_span: Span,
) -> Diagnostic {
    let e = e.context(format!(
        "type mismatch: expected type `{expected}`, but found type `{actual}`"
    ));

    Diagnostic::error(format!("{e:#}"))
        .with_label(format!("this is type `{actual}`"), actual_span)
        .with_label(format!("this expects type `{expected}`"), expected_span)
}

/// Creates an "if conditional mismatch" diagnostic.
pub fn if_conditional_mismatch(e: anyhow::Error, actual: &Type, actual_span: Span) -> Diagnostic {
    let e = e.context(format!(
        "type mismatch: expected `if` conditional expression to be type `Boolean`, but found type \
         `{actual}`"
    ));

    Diagnostic::error(format!("{e:#}")).with_label(format!("this is type `{actual}`"), actual_span)
}

/// Creates an "array index out of range" diagnostic.
pub fn array_index_out_of_range(
    index: i64,
    count: usize,
    span: Span,
    target_span: Span,
) -> Diagnostic {
    Diagnostic::error(format!("array index {index} is out of range"))
        .with_highlight(span)
        .with_label(
            if count == 0 {
                "this array is empty".to_string()
            } else {
                format!(
                    "this array has only {count} element{s}",
                    s = if count == 1 { "" } else { "s" }
                )
            },
            target_span,
        )
}

/// Creates a "map key not found" diagnostic.
pub fn map_key_not_found(span: Span) -> Diagnostic {
    Diagnostic::error("the map does not contain an entry for the specified key")
        .with_highlight(span)
}

/// Creates a "not an object member" diagnostic.
pub fn not_an_object_member<T: TreeToken>(member: &Ident<T>) -> Diagnostic {
    Diagnostic::error(format!(
        "object does not have a member named `{member}`",
        member = member.text()
    ))
    .with_highlight(member.span())
}

/// Creates an "exponentiation requirement" diagnostic.
pub fn exponentiation_requirement(span: Span) -> Diagnostic {
    Diagnostic::error("use of the exponentiation operator requires WDL version 1.2")
        .with_highlight(span)
}

/// Creates a "multi-line string requirement" diagnostic.
pub fn multiline_string_requirement(span: Span) -> Diagnostic {
    Diagnostic::error("use of multi-line strings requires WDL version 1.2").with_highlight(span)
}

/// Creates an "invalid regular expression" diagnostic.
pub fn invalid_regex(error: &regex::Error, span: Span) -> Diagnostic {
    Diagnostic::error(error.to_string()).with_highlight(span)
}

/// Creates a "path not relative" diagnostic.
pub fn path_not_relative(span: Span) -> Diagnostic {
    Diagnostic::error("path is required to be a relative path, but an absolute path was provided")
        .with_highlight(span)
}

/// Creates an "array path not relative" diagnostic.
pub fn array_path_not_relative(index: usize, span: Span) -> Diagnostic {
    Diagnostic::error(format!(
        "index {index} of the array is required to be a relative path, but an absolute path was \
         provided"
    ))
    .with_highlight(span)
}

/// Creates an "invalid glob pattern" diagnostic.
pub fn invalid_glob_pattern(error: &glob::PatternError, span: Span) -> Diagnostic {
    Diagnostic::error(format!(
        "invalid glob pattern specified: {error}",
        error = error.msg
    ))
    .with_highlight(span)
}

/// Creates an "invalid storage unit" diagnostic.
pub fn invalid_storage_unit(unit: &str, span: Span) -> Diagnostic {
    Diagnostic::error(format!(
        "invalid storage unit `{unit}`; supported units are `B`, `KB`, `K`, `MB`, `M`, `GB`, `G`, \
         `TB`, `T`, `KiB`, `Ki`, `MiB`, `Mi`, `GiB`, `Gi`, `TiB`, and `Ti`",
    ))
    .with_highlight(span)
}

/// Creates a "function call failed" diagnostic.
pub fn function_call_failed(name: &str, error: impl fmt::Display, span: Span) -> Diagnostic {
    Diagnostic::error(format!("call to function `{name}` failed: {error}")).with_highlight(span)
}

/// Creates an "output evaluation failed" diagnostic.
pub fn output_evaluation_failed(
    e: anyhow::Error,
    name: &str,
    task: bool,
    output: &str,
    span: Span,
) -> Diagnostic {
    let e = e.context(format!(
        "failed to evaluate output `{output}` for {kind} `{name}`",
        kind = if task { "task" } else { "workflow" }
    ));

    Diagnostic::error(format!("{e:#}")).with_highlight(span)
}
