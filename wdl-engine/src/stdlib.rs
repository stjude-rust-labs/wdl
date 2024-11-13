//! Module for the WDL standard library implementation.

use std::borrow::Cow;
use std::collections::HashMap;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::LazyLock;

use regex::Regex;
use wdl_analysis::stdlib::Binding;
use wdl_analysis::types::Optional;
use wdl_analysis::types::PrimitiveTypeKind;
use wdl_analysis::types::Type;
use wdl_analysis::types::TypeEq;
use wdl_analysis::types::Types;
use wdl_ast::Diagnostic;
use wdl_ast::Span;

use crate::Coercible;
use crate::PrimitiveValue;
use crate::Value;
use crate::diagnostics::array_path_not_relative;
use crate::diagnostics::invalid_regex;
use crate::diagnostics::path_not_relative;

/// Rounds a floating point number down to the next lower integer.
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#floor
pub fn floor(
    types: &Types,
    arguments: &[(Value, Span)],
    return_type: Type,
) -> Result<Value, Diagnostic> {
    debug_assert_eq!(arguments.len(), 1);
    debug_assert!(return_type.type_eq(types, &PrimitiveTypeKind::Integer.into()));

    let arg = arguments[0]
        .0
        .coerce(types, PrimitiveTypeKind::Float.into())
        .expect("value should coerce to float")
        .unwrap_float();
    Ok((arg.floor() as i64).into())
}

/// Rounds a floating point number up to the next higher integer.
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#ceil
pub fn ceil(
    types: &Types,
    arguments: &[(Value, Span)],
    return_type: Type,
) -> Result<Value, Diagnostic> {
    debug_assert_eq!(arguments.len(), 1);
    debug_assert!(return_type.type_eq(types, &PrimitiveTypeKind::Integer.into()));

    let arg = arguments[0]
        .0
        .coerce(types, PrimitiveTypeKind::Float.into())
        .expect("value should coerce to float")
        .unwrap_float();
    Ok((arg.ceil() as i64).into())
}

/// Rounds a floating point number to the nearest integer based on standard
/// rounding rules ("round half up").
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#round
pub fn round(
    types: &Types,
    arguments: &[(Value, Span)],
    return_type: Type,
) -> Result<Value, Diagnostic> {
    debug_assert_eq!(arguments.len(), 1);
    debug_assert!(return_type.type_eq(types, &PrimitiveTypeKind::Integer.into()));

    let arg = arguments[0]
        .0
        .coerce(types, PrimitiveTypeKind::Float.into())
        .expect("value should coerce to float")
        .unwrap_float();
    Ok((arg.round() as i64).into())
}

/// Returns the smaller of two integer values.
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#min
pub fn int_min(
    types: &Types,
    arguments: &[(Value, Span)],
    return_type: Type,
) -> Result<Value, Diagnostic> {
    debug_assert_eq!(arguments.len(), 2);
    debug_assert!(return_type.type_eq(types, &PrimitiveTypeKind::Integer.into()));

    let first = arguments[0]
        .0
        .coerce(types, PrimitiveTypeKind::Integer.into())
        .expect("value should coerce to integer")
        .unwrap_integer();
    let second = arguments[1]
        .0
        .coerce(types, PrimitiveTypeKind::Integer.into())
        .expect("value should coerce to integer")
        .unwrap_integer();
    Ok(first.min(second).into())
}

/// Returns the smaller of two float values.
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#min
pub fn float_min(
    types: &Types,
    arguments: &[(Value, Span)],
    return_type: Type,
) -> Result<Value, Diagnostic> {
    debug_assert_eq!(arguments.len(), 2);
    debug_assert!(return_type.type_eq(types, &PrimitiveTypeKind::Float.into()));

    let first = arguments[0]
        .0
        .coerce(types, PrimitiveTypeKind::Float.into())
        .expect("value should coerce to float")
        .unwrap_float();
    let second = arguments[1]
        .0
        .coerce(types, PrimitiveTypeKind::Float.into())
        .expect("value should coerce to float")
        .unwrap_float();
    Ok(first.min(second).into())
}

/// Returns the larger of two integer values.
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#max
pub fn int_max(
    types: &Types,
    arguments: &[(Value, Span)],
    return_type: Type,
) -> Result<Value, Diagnostic> {
    debug_assert_eq!(arguments.len(), 2);
    debug_assert!(return_type.type_eq(types, &PrimitiveTypeKind::Integer.into()));

    let first = arguments[0]
        .0
        .coerce(types, PrimitiveTypeKind::Integer.into())
        .expect("value should coerce to integer")
        .unwrap_integer();
    let second = arguments[1]
        .0
        .coerce(types, PrimitiveTypeKind::Integer.into())
        .expect("value should coerce to integer")
        .unwrap_integer();
    Ok(first.max(second).into())
}

/// Returns the larger of two float values.
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#max
pub fn float_max(
    types: &Types,
    arguments: &[(Value, Span)],
    return_type: Type,
) -> Result<Value, Diagnostic> {
    debug_assert_eq!(arguments.len(), 2);
    debug_assert!(return_type.type_eq(types, &PrimitiveTypeKind::Float.into()));

    let first = arguments[0]
        .0
        .coerce(types, PrimitiveTypeKind::Float.into())
        .expect("value should coerce to float")
        .unwrap_float();
    let second = arguments[1]
        .0
        .coerce(types, PrimitiveTypeKind::Float.into())
        .expect("value should coerce to float")
        .unwrap_float();
    Ok(first.max(second).into())
}

/// Given two String parameters `input` and `pattern`, searches for the
/// occurrence of `pattern` within `input` and returns the first match or `None`
/// if there are no matches.
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#-find
pub fn find(
    types: &Types,
    arguments: &[(Value, Span)],
    return_type: Type,
) -> Result<Value, Diagnostic> {
    debug_assert_eq!(arguments.len(), 2);
    debug_assert!(return_type.type_eq(types, &Type::from(PrimitiveTypeKind::String).optional()));

    let input = arguments[0]
        .0
        .coerce(types, PrimitiveTypeKind::String.into())
        .expect("value should coerce to string")
        .unwrap_string();
    let pattern = arguments[1]
        .0
        .coerce(types, PrimitiveTypeKind::String.into())
        .expect("value should coerce to string")
        .unwrap_string();

    let regex = Regex::new(pattern.as_str()).map_err(|e| invalid_regex(&e, arguments[1].1))?;

    match regex.find(input.as_str()) {
        Some(m) => Ok(PrimitiveValue::new_string(m.as_str()).into()),
        None => Ok(Value::None),
    }
}

/// Given two String parameters `input` and `pattern`, tests whether `pattern`
/// matches `input` at least once.
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#-matches
pub fn matches(
    types: &Types,
    arguments: &[(Value, Span)],
    return_type: Type,
) -> Result<Value, Diagnostic> {
    debug_assert_eq!(arguments.len(), 2);
    debug_assert!(return_type.type_eq(types, &PrimitiveTypeKind::Boolean.into()));

    let input = arguments[0]
        .0
        .coerce(types, PrimitiveTypeKind::String.into())
        .expect("value should coerce to string")
        .unwrap_string();
    let pattern = arguments[1]
        .0
        .coerce(types, PrimitiveTypeKind::String.into())
        .expect("value should coerce to string")
        .unwrap_string();

    let regex = Regex::new(pattern.as_str()).map_err(|e| invalid_regex(&e, arguments[1].1))?;
    Ok(regex.is_match(input.as_str()).into())
}

/// Given three String parameters `input`, `pattern`, and `replace`, this
/// function replaces all non-overlapping occurrences of `pattern` in `input`
/// with `replace`.
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#sub
pub fn sub(
    types: &Types,
    arguments: &[(Value, Span)],
    return_type: Type,
) -> Result<Value, Diagnostic> {
    debug_assert_eq!(arguments.len(), 3);
    debug_assert!(return_type.type_eq(types, &PrimitiveTypeKind::String.into()));

    let input = arguments[0]
        .0
        .coerce(types, PrimitiveTypeKind::String.into())
        .expect("value should coerce to string")
        .unwrap_string();
    let pattern = arguments[1]
        .0
        .coerce(types, PrimitiveTypeKind::String.into())
        .expect("value should coerce to string")
        .unwrap_string();
    let replacement = arguments[2]
        .0
        .coerce(types, PrimitiveTypeKind::String.into())
        .expect("value should coerce to string")
        .unwrap_string();

    let regex = Regex::new(pattern.as_str()).map_err(|e| invalid_regex(&e, arguments[1].1))?;
    match regex.replace(input.as_str(), replacement.as_str()) {
        Cow::Borrowed(_) => {
            // No replacements, just return the input
            Ok(PrimitiveValue::String(input).into())
        }
        Cow::Owned(s) => {
            // A replacement occurred, allocate a new string
            Ok(PrimitiveValue::new_string(s).into())
        }
    }
}

/// Returns the "basename" of a file or directory - the name after the last
/// directory separator in the path.
///
/// The optional second parameter specifies a literal suffix to remove from the
/// file name. If the file name does not end with the specified suffix then it
/// is ignored.
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#basename
pub fn basename(
    types: &Types,
    arguments: &[(Value, Span)],
    return_type: Type,
) -> Result<Value, Diagnostic> {
    debug_assert!(!arguments.is_empty() && arguments.len() < 3);
    debug_assert!(return_type.type_eq(types, &PrimitiveTypeKind::String.into()));

    let path = arguments[0]
        .0
        .coerce(types, PrimitiveTypeKind::String.into())
        .expect("value should coerce to string")
        .unwrap_string();

    match Path::new(path.as_str()).file_name() {
        Some(base) => {
            let base = base.to_str().expect("should be UTF-8");
            let base = if arguments.len() == 2 {
                base.strip_suffix(
                    arguments[1]
                        .0
                        .coerce(types, PrimitiveTypeKind::String.into())
                        .expect("value should coerce to string")
                        .unwrap_string()
                        .as_str(),
                )
                .unwrap_or(base)
            } else {
                base
            };

            Ok(PrimitiveValue::new_string(base).into())
        }
        None => Ok(PrimitiveValue::String(path).into()),
    }
}

/// Joins together two or more paths into an absolute path in the host
/// filesystem.
///
/// `File join_paths(File, String)`: Joins together exactly two paths. The first
/// path may be either absolute or relative and must specify a directory; the
/// second path is relative to the first path and may specify a file or
/// directory.
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#-join_paths
pub fn join_paths_simple(
    types: &Types,
    arguments: &[(Value, Span)],
    return_type: Type,
) -> Result<Value, Diagnostic> {
    debug_assert!(arguments.len() == 2);
    debug_assert!(return_type.type_eq(types, &PrimitiveTypeKind::File.into()));

    let first = arguments[0]
        .0
        .coerce(types, PrimitiveTypeKind::File.into())
        .expect("value should coerce to a file")
        .unwrap_file();

    let second = arguments[1]
        .0
        .coerce(types, PrimitiveTypeKind::String.into())
        .expect("value should coerce to a string")
        .unwrap_string();

    let second = Path::new(second.as_str());
    if !second.is_relative() {
        return Err(path_not_relative(arguments[1].1));
    }

    let mut path = PathBuf::from(Arc::unwrap_or_clone(first));
    path.push(second);

    Ok(PrimitiveValue::new_file(
        path.into_os_string()
            .into_string()
            .expect("should be UTF-8"),
    )
    .into())
}

/// Joins together two or more paths into an absolute path in the host
/// filesystem.
///
/// `File join_paths(File, Array[String]+)`: Joins together any number of
/// relative paths with a base path. The first argument may be either an
/// absolute or a relative path and must specify a directory. The paths in the
/// second array argument must all be relative. The last element may specify a
/// file or directory; all other elements must specify a directory.
///
/// `File join_paths(Array[String]+)`: Joins together any number of paths. The
/// array must not be empty. The first element of the array may be either
/// absolute or relative; subsequent path(s) must be relative. The last element
/// may specify a file or directory; all other elements must specify a
/// directory.
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#-join_paths
pub fn join_paths(
    types: &Types,
    arguments: &[(Value, Span)],
    return_type: Type,
) -> Result<Value, Diagnostic> {
    debug_assert!(!arguments.is_empty() && arguments.len() < 3);
    debug_assert!(return_type.type_eq(types, &PrimitiveTypeKind::File.into()));

    // Handle being provided one or two arguments
    let (first, array, skip, array_span) = if arguments.len() == 1 {
        let array = arguments[0]
            .0
            .coerce(
                types,
                wdl_analysis::stdlib::STDLIB.array_string_non_empty_type(),
            )
            .expect("value should coerce to Array[String]+")
            .unwrap_array();

        (
            array.elements()[0].clone().unwrap_string(),
            array,
            true,
            arguments[0].1,
        )
    } else {
        let first = arguments[0]
            .0
            .coerce(types, PrimitiveTypeKind::File.into())
            .expect("value should coerce to a file")
            .unwrap_file();

        let array = arguments[1]
            .0
            .coerce(
                types,
                wdl_analysis::stdlib::STDLIB.array_string_non_empty_type(),
            )
            .expect("value should coerce to a Array[String]+")
            .unwrap_array();

        (first, array, false, arguments[1].1)
    };

    let mut path = PathBuf::from(Arc::unwrap_or_clone(first));

    for (i, second) in array
        .elements()
        .iter()
        .enumerate()
        .skip(if skip { 1 } else { 0 })
    {
        let second = second
            .coerce(types, PrimitiveTypeKind::String.into())
            .expect("value should coerce to a string")
            .unwrap_string();

        let second = Path::new(second.as_str());
        if !second.is_relative() {
            return Err(array_path_not_relative(i, array_span));
        }

        path.push(second);
    }

    Ok(PrimitiveValue::new_file(
        path.into_os_string()
            .into_string()
            .expect("should be UTF-8"),
    )
    .into())
}

/// Represents a WDL function implementation callback.
type Callback = fn(&Types, &[(Value, Span)], Type) -> Result<Value, Diagnostic>;

/// Represents an implementation signature for a WDL standard library function.
#[derive(Debug, Clone, Copy)]
pub struct Signature {
    /// The display string of the signature.
    ///
    /// This is only used for unit tests.
    #[allow(unused)]
    display: &'static str,
    /// The implementation callback of the signature.
    callback: Callback,
}

impl Signature {
    /// Constructs a new signature given its display and callback.
    const fn new(display: &'static str, callback: Callback) -> Self {
        Self { display, callback }
    }
}

/// Represents a standard library function.
#[derive(Debug, Clone, Copy)]
pub struct Function {
    /// The signatures of the function.
    signatures: &'static [Signature],
}

impl Function {
    /// Constructs a new function given its signatures.
    const fn new(signatures: &'static [Signature]) -> Self {
        Self { signatures }
    }

    /// Calls the function given the binding and the function's arguments.
    #[inline]
    pub fn call(
        &self,
        binding: Binding<'_>,
        types: &Types,
        arguments: &[(Value, Span)],
    ) -> Result<Value, Diagnostic> {
        (self.signatures[binding.index()].callback)(types, arguments, binding.return_type())
    }
}

/// Represents the WDL standard library.
#[derive(Debug)]
pub struct StandardLibrary {
    /// The implementation functions for the standard library.
    functions: HashMap<&'static str, Function>,
}

impl StandardLibrary {
    /// Gets a function from the standard library.
    ///
    /// Returns `None` if the function isn't in the WDL standard library.
    #[inline]
    pub fn get(&self, name: &str) -> Option<Function> {
        self.functions.get(name).copied()
    }
}

/// Represents the mapping between function name and overload index to the
/// implementation callback.
pub static STDLIB: LazyLock<StandardLibrary> = LazyLock::new(|| {
    let mut functions = HashMap::with_capacity(wdl_analysis::stdlib::STDLIB.functions().len());
    assert!(
        functions
            .insert(
                "floor",
                Function::new(const { &[Signature::new("(Float) -> Int", floor)] })
            )
            .is_none()
    );
    assert!(
        functions
            .insert(
                "ceil",
                Function::new(const { &[Signature::new("(Float) -> Int", ceil)] })
            )
            .is_none()
    );
    assert!(
        functions
            .insert(
                "round",
                Function::new(const { &[Signature::new("(Float) -> Int", round)] })
            )
            .is_none()
    );
    assert!(
        functions
            .insert(
                "min",
                Function::new(
                    const {
                        &[
                            Signature::new("(Int, Int) -> Int", int_min),
                            Signature::new("(Int, Float) -> Float", float_min),
                            Signature::new("(Float, Int) -> Float", float_min),
                            Signature::new("(Float, Float) -> Float", float_min),
                        ]
                    }
                )
            )
            .is_none()
    );
    assert!(
        functions
            .insert(
                "max",
                Function::new(
                    const {
                        &[
                            Signature::new("(Int, Int) -> Int", int_max),
                            Signature::new("(Int, Float) -> Float", float_max),
                            Signature::new("(Float, Int) -> Float", float_max),
                            Signature::new("(Float, Float) -> Float", float_max),
                        ]
                    }
                )
            )
            .is_none()
    );
    assert!(
        functions
            .insert(
                "find",
                Function::new(const { &[Signature::new("(String, String) -> String?", find)] })
            )
            .is_none()
    );
    assert!(
        functions
            .insert(
                "matches",
                Function::new(const { &[Signature::new("(String, String) -> Boolean", matches)] })
            )
            .is_none()
    );
    assert!(
        functions
            .insert(
                "sub",
                Function::new(
                    const { &[Signature::new("(String, String, String) -> String", sub)] }
                )
            )
            .is_none()
    );
    assert!(
        functions
            .insert(
                "basename",
                Function::new(
                    const {
                        &[
                            Signature::new("(File, <String>) -> String", basename),
                            Signature::new("(String, <String>) -> String", basename),
                            Signature::new("(Directory, <String>) -> String", basename),
                        ]
                    }
                )
            )
            .is_none()
    );
    assert!(
        functions
            .insert(
                "join_paths",
                Function::new(
                    const {
                        &[
                            Signature::new("(File, String) -> File", join_paths_simple),
                            Signature::new("(File, Array[String]+) -> File", join_paths),
                            Signature::new("(Array[String]+) -> File", join_paths),
                        ]
                    }
                )
            )
            .is_none()
    );

    StandardLibrary { functions }
});

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use wdl_analysis::stdlib::TypeParameters;
    use wdl_ast::version::V1;

    use super::*;
    use crate::Scope;
    use crate::ScopeRef;
    use crate::v1::test::eval_v1_expr;

    /// A test to verify that the STDLIB function types from `wdl-analysis`
    /// aligns with the STDLIB implementation from `wdl-engine`.
    #[test]
    fn verify_stdlib() {
        for (name, func) in wdl_analysis::stdlib::STDLIB.functions() {
            match STDLIB.functions.get(name) {
                Some(imp) => match func {
                    wdl_analysis::stdlib::Function::Monomorphic(f) => {
                        assert_eq!(
                            imp.signatures.len(),
                            1,
                            "signature mismatch for function `{name}`"
                        );
                        assert_eq!(
                            f.signature()
                                .display(
                                    wdl_analysis::stdlib::STDLIB.types(),
                                    &TypeParameters::new(f.signature().type_parameters())
                                )
                                .to_string(),
                            imp.signatures[0].display,
                            "signature mismatch for function `{name}`"
                        );
                    }
                    wdl_analysis::stdlib::Function::Polymorphic(f) => {
                        assert_eq!(imp.signatures.len(), f.signatures().len());
                        for (i, sig) in f.signatures().iter().enumerate() {
                            assert_eq!(
                                sig.display(
                                    wdl_analysis::stdlib::STDLIB.types(),
                                    &TypeParameters::new(sig.type_parameters())
                                )
                                .to_string(),
                                imp.signatures[i].display,
                                "signature mismatch for function `{name}` (index {i})"
                            );
                        }
                    }
                },
                None => {
                    // TODO: make this a failure in the future once the entire STDLIB is implemented
                    continue;
                }
            }
        }
    }

    #[test]
    fn floor() {
        let scopes = &[Scope::new(None)];
        let scope = ScopeRef::new(scopes, 0);

        let mut types = Types::default();
        let value = eval_v1_expr(V1::Zero, "floor(10.5)", &mut types, scope).unwrap();
        assert_eq!(value.unwrap_integer(), 10);

        let value = eval_v1_expr(V1::Zero, "floor(10)", &mut types, scope).unwrap();
        assert_eq!(value.unwrap_integer(), 10);

        let value = eval_v1_expr(V1::Zero, "floor(9.9999)", &mut types, scope).unwrap();
        assert_eq!(value.unwrap_integer(), 9);

        let value = eval_v1_expr(V1::Zero, "floor(0)", &mut types, scope).unwrap();
        assert_eq!(value.unwrap_integer(), 0);

        let value = eval_v1_expr(V1::Zero, "floor(-5.1)", &mut types, scope).unwrap();
        assert_eq!(value.unwrap_integer(), -6);
    }

    #[test]
    fn ceil() {
        let scopes = &[Scope::new(None)];
        let scope = ScopeRef::new(scopes, 0);

        let mut types = Types::default();
        let value = eval_v1_expr(V1::Zero, "ceil(10.5)", &mut types, scope).unwrap();
        assert_eq!(value.unwrap_integer(), 11);

        let value = eval_v1_expr(V1::Zero, "ceil(10)", &mut types, scope).unwrap();
        assert_eq!(value.unwrap_integer(), 10);

        let value = eval_v1_expr(V1::Zero, "ceil(9.9999)", &mut types, scope).unwrap();
        assert_eq!(value.unwrap_integer(), 10);

        let value = eval_v1_expr(V1::Zero, "ceil(0)", &mut types, scope).unwrap();
        assert_eq!(value.unwrap_integer(), 0);

        let value = eval_v1_expr(V1::Zero, "ceil(-5.1)", &mut types, scope).unwrap();
        assert_eq!(value.unwrap_integer(), -5);
    }

    #[test]
    fn round() {
        let scopes = &[Scope::new(None)];
        let scope = ScopeRef::new(scopes, 0);

        let mut types = Types::default();
        let value = eval_v1_expr(V1::Zero, "round(10.5)", &mut types, scope).unwrap();
        assert_eq!(value.unwrap_integer(), 11);

        let value = eval_v1_expr(V1::Zero, "round(10.3)", &mut types, scope).unwrap();
        assert_eq!(value.unwrap_integer(), 10);

        let value = eval_v1_expr(V1::Zero, "round(10)", &mut types, scope).unwrap();
        assert_eq!(value.unwrap_integer(), 10);

        let value = eval_v1_expr(V1::Zero, "round(9.9999)", &mut types, scope).unwrap();
        assert_eq!(value.unwrap_integer(), 10);

        let value = eval_v1_expr(V1::Zero, "round(9.12345)", &mut types, scope).unwrap();
        assert_eq!(value.unwrap_integer(), 9);

        let value = eval_v1_expr(V1::Zero, "round(0)", &mut types, scope).unwrap();
        assert_eq!(value.unwrap_integer(), 0);

        let value = eval_v1_expr(V1::Zero, "round(-5.1)", &mut types, scope).unwrap();
        assert_eq!(value.unwrap_integer(), -5);

        let value = eval_v1_expr(V1::Zero, "round(-5.5)", &mut types, scope).unwrap();
        assert_eq!(value.unwrap_integer(), -6);
    }

    #[test]
    fn min() {
        let scopes = &[Scope::new(None)];
        let scope = ScopeRef::new(scopes, 0);

        let mut types = Types::default();
        let value = eval_v1_expr(V1::One, "min(7, 42)", &mut types, scope).unwrap();
        assert_eq!(value.unwrap_integer(), 7);

        let value = eval_v1_expr(V1::One, "min(42, 7)", &mut types, scope).unwrap();
        assert_eq!(value.unwrap_integer(), 7);

        let value = eval_v1_expr(V1::One, "min(-42, 7)", &mut types, scope).unwrap();
        assert_eq!(value.unwrap_integer(), -42);

        let value = eval_v1_expr(V1::One, "min(0, -42)", &mut types, scope).unwrap();
        assert_eq!(value.unwrap_integer(), -42);

        let value = eval_v1_expr(V1::One, "min(0, 42)", &mut types, scope).unwrap();
        assert_eq!(value.unwrap_integer(), 0);

        let value = eval_v1_expr(V1::One, "min(7.0, 42)", &mut types, scope).unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 7.0);

        let value = eval_v1_expr(V1::One, "min(42.0, 7)", &mut types, scope).unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 7.0);

        let value = eval_v1_expr(V1::One, "min(-42.0, 7)", &mut types, scope).unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), -42.0);

        let value = eval_v1_expr(V1::One, "min(0.0, -42)", &mut types, scope).unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), -42.0);

        let value = eval_v1_expr(V1::One, "min(0.0, 42)", &mut types, scope).unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), -0.0);

        let value = eval_v1_expr(V1::One, "min(7, 42.0)", &mut types, scope).unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 7.0);

        let value = eval_v1_expr(V1::One, "min(42, 7.0)", &mut types, scope).unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 7.0);

        let value = eval_v1_expr(V1::One, "min(-42, 7.0)", &mut types, scope).unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), -42.0);

        let value = eval_v1_expr(V1::One, "min(0, -42.0)", &mut types, scope).unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), -42.0);

        let value = eval_v1_expr(V1::One, "min(0, 42.0)", &mut types, scope).unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), -0.0);

        let value = eval_v1_expr(V1::One, "min(7.0, 42.0)", &mut types, scope).unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 7.0);

        let value = eval_v1_expr(V1::One, "min(42.0, 7.0)", &mut types, scope).unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 7.0);

        let value = eval_v1_expr(V1::One, "min(-42.0, 7.0)", &mut types, scope).unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), -42.0);

        let value = eval_v1_expr(V1::One, "min(0.0, -42.0)", &mut types, scope).unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), -42.0);

        let value = eval_v1_expr(V1::One, "min(0.0, 42.0)", &mut types, scope).unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), -0.0);

        let value = eval_v1_expr(
            V1::One,
            "min(12345, min(-100, min(54321, 1234.5678)))",
            &mut types,
            scope,
        )
        .unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), -100.0);
    }

    #[test]
    fn max() {
        let scopes = &[Scope::new(None)];
        let scope = ScopeRef::new(scopes, 0);

        let mut types = Types::default();
        let value = eval_v1_expr(V1::One, "max(7, 42)", &mut types, scope).unwrap();
        assert_eq!(value.unwrap_integer(), 42);

        let value = eval_v1_expr(V1::One, "max(42, 7)", &mut types, scope).unwrap();
        assert_eq!(value.unwrap_integer(), 42);

        let value = eval_v1_expr(V1::One, "max(-42, 7)", &mut types, scope).unwrap();
        assert_eq!(value.unwrap_integer(), 7);

        let value = eval_v1_expr(V1::One, "max(0, -42)", &mut types, scope).unwrap();
        assert_eq!(value.unwrap_integer(), 0);

        let value = eval_v1_expr(V1::One, "max(0, 42)", &mut types, scope).unwrap();
        assert_eq!(value.unwrap_integer(), 42);

        let value = eval_v1_expr(V1::One, "max(7.0, 42)", &mut types, scope).unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 42.0);

        let value = eval_v1_expr(V1::One, "max(42.0, 7)", &mut types, scope).unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 42.0);

        let value = eval_v1_expr(V1::One, "max(-42.0, 7)", &mut types, scope).unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 7.0);

        let value = eval_v1_expr(V1::One, "max(0.0, -42)", &mut types, scope).unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 0.0);

        let value = eval_v1_expr(V1::One, "max(0.0, 42)", &mut types, scope).unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 42.0);

        let value = eval_v1_expr(V1::One, "max(7, 42.0)", &mut types, scope).unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 42.0);

        let value = eval_v1_expr(V1::One, "max(42, 7.0)", &mut types, scope).unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 42.0);

        let value = eval_v1_expr(V1::One, "max(-42, 7.0)", &mut types, scope).unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 7.0);

        let value = eval_v1_expr(V1::One, "max(0, -42.0)", &mut types, scope).unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 0.0);

        let value = eval_v1_expr(V1::One, "max(0, 42.0)", &mut types, scope).unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 42.0);

        let value = eval_v1_expr(V1::One, "max(7.0, 42.0)", &mut types, scope).unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 42.0);

        let value = eval_v1_expr(V1::One, "max(42.0, 7.0)", &mut types, scope).unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 42.0);

        let value = eval_v1_expr(V1::One, "max(-42.0, 7.0)", &mut types, scope).unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 7.0);

        let value = eval_v1_expr(V1::One, "max(0.0, -42.0)", &mut types, scope).unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 0.0);

        let value = eval_v1_expr(V1::One, "max(0.0, 42.0)", &mut types, scope).unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 42.0);

        let value = eval_v1_expr(
            V1::One,
            "max(12345, max(-100, max(54321, 1234.5678)))",
            &mut types,
            scope,
        )
        .unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 54321.0);
    }

    #[test]
    fn find() {
        let scopes = &[Scope::new(None)];
        let scope = ScopeRef::new(scopes, 0);

        let mut types = Types::default();
        let diagnostic =
            eval_v1_expr(V1::Two, "find('foo bar baz', '?')", &mut types, scope).unwrap_err();
        assert_eq!(
            diagnostic.message(),
            "regex parse error:\n    ?\n    ^\nerror: repetition operator missing expression"
        );

        let value =
            eval_v1_expr(V1::Two, "find('hello world', 'e..o')", &mut types, scope).unwrap();
        assert_eq!(value.unwrap_string().as_str(), "ello");

        let value =
            eval_v1_expr(V1::Two, "find('hello world', 'goodbye')", &mut types, scope).unwrap();
        assert!(value.is_none());

        let value = eval_v1_expr(V1::Two, "find('hello\tBob', '\\t')", &mut types, scope).unwrap();
        assert_eq!(value.unwrap_string().as_str(), "\t");
    }

    #[test]
    fn matches() {
        let scopes = &[Scope::new(None)];
        let scope = ScopeRef::new(scopes, 0);

        let mut types = Types::default();
        let diagnostic =
            eval_v1_expr(V1::Two, "matches('foo bar baz', '?')", &mut types, scope).unwrap_err();
        assert_eq!(
            diagnostic.message(),
            "regex parse error:\n    ?\n    ^\nerror: repetition operator missing expression"
        );

        let value =
            eval_v1_expr(V1::Two, "matches('hello world', 'e..o')", &mut types, scope).unwrap();
        assert!(value.unwrap_boolean());

        let value = eval_v1_expr(
            V1::Two,
            "matches('hello world', 'goodbye')",
            &mut types,
            scope,
        )
        .unwrap();
        assert!(!value.unwrap_boolean());

        let value =
            eval_v1_expr(V1::Two, "matches('hello\tBob', '\\t')", &mut types, scope).unwrap();
        assert!(value.unwrap_boolean());
    }

    #[test]
    fn sub() {
        let scopes = &[Scope::new(None)];
        let scope = ScopeRef::new(scopes, 0);

        let mut types = Types::default();
        let diagnostic = eval_v1_expr(
            V1::Two,
            "sub('foo bar baz', '?', 'nope')",
            &mut types,
            scope,
        )
        .unwrap_err();
        assert_eq!(
            diagnostic.message(),
            "regex parse error:\n    ?\n    ^\nerror: repetition operator missing expression"
        );

        let value = eval_v1_expr(
            V1::Two,
            "sub('hello world', 'e..o', 'ey there')",
            &mut types,
            scope,
        )
        .unwrap();
        assert_eq!(value.unwrap_string().as_str(), "hey there world");

        let value = eval_v1_expr(
            V1::Two,
            "sub('hello world', 'goodbye', 'nope')",
            &mut types,
            scope,
        )
        .unwrap();
        assert_eq!(value.unwrap_string().as_str(), "hello world");

        let value =
            eval_v1_expr(V1::Two, "sub('hello\tBob', '\\t', ' ')", &mut types, scope).unwrap();
        assert_eq!(value.unwrap_string().as_str(), "hello Bob");
    }

    #[test]
    fn basename() {
        let scopes = &[Scope::new(None)];
        let scope = ScopeRef::new(scopes, 0);

        let mut types = Types::default();
        let value =
            eval_v1_expr(V1::Two, "basename('/path/to/file.txt')", &mut types, scope).unwrap();
        assert_eq!(value.unwrap_string().as_str(), "file.txt");

        let value = eval_v1_expr(
            V1::Two,
            "basename('/path/to/file.txt', '.txt')",
            &mut types,
            scope,
        )
        .unwrap();
        assert_eq!(value.unwrap_string().as_str(), "file");

        let value = eval_v1_expr(V1::Two, "basename('/path/to/dir')", &mut types, scope).unwrap();
        assert_eq!(value.unwrap_string().as_str(), "dir");

        let value = eval_v1_expr(V1::Two, "basename('file.txt')", &mut types, scope).unwrap();
        assert_eq!(value.unwrap_string().as_str(), "file.txt");

        let value =
            eval_v1_expr(V1::Two, "basename('file.txt', '.txt')", &mut types, scope).unwrap();
        assert_eq!(value.unwrap_string().as_str(), "file");
    }

    #[test]
    fn join_paths() {
        let scopes = &[Scope::new(None)];
        let scope = ScopeRef::new(scopes, 0);

        let mut types = Types::default();
        let value = eval_v1_expr(
            V1::Two,
            "join_paths('/usr', ['bin', 'echo'])",
            &mut types,
            scope,
        )
        .unwrap();
        assert_eq!(value.unwrap_file().as_str(), "/usr/bin/echo");

        let value = eval_v1_expr(
            V1::Two,
            "join_paths(['/usr', 'bin', 'echo'])",
            &mut types,
            scope,
        )
        .unwrap();
        assert_eq!(value.unwrap_file().as_str(), "/usr/bin/echo");

        let value = eval_v1_expr(
            V1::Two,
            "join_paths('mydir', 'mydata.txt')",
            &mut types,
            scope,
        )
        .unwrap();
        assert_eq!(value.unwrap_file().as_str(), "mydir/mydata.txt");

        let value =
            eval_v1_expr(V1::Two, "join_paths('/usr', 'bin/echo')", &mut types, scope).unwrap();
        assert_eq!(value.unwrap_file().as_str(), "/usr/bin/echo");

        #[cfg(unix)]
        {
            let diagnostic = eval_v1_expr(
                V1::Two,
                "join_paths('/usr', '/bin/echo')",
                &mut types,
                scope,
            )
            .unwrap_err();
            assert_eq!(
                diagnostic.message(),
                "path is required to be a relative path, but an absolute path was provided"
            );

            let diagnostic = eval_v1_expr(
                V1::Two,
                "join_paths('/usr', ['foo', '/bin/echo'])",
                &mut types,
                scope,
            )
            .unwrap_err();
            assert_eq!(
                diagnostic.message(),
                "index 1 of the array is required to be a relative path, but an absolute path was \
                 provided"
            );

            let diagnostic = eval_v1_expr(
                V1::Two,
                "join_paths(['/usr', 'foo', '/bin/echo'])",
                &mut types,
                scope,
            )
            .unwrap_err();
            assert_eq!(
                diagnostic.message(),
                "index 2 of the array is required to be a relative path, but an absolute path was \
                 provided"
            );
        }

        #[cfg(windows)]
        {
            let diagnostic = eval_v1_expr(
                V1::Two,
                "join_paths('C:\\usr', 'C:\\bin\\echo')",
                &mut types,
                scope,
            )
            .unwrap_err();
            assert_eq!(
                diagnostic.message(),
                "path is required to be a relative path, but an absolute path was provided"
            );

            let diagnostic = eval_v1_expr(
                V1::Two,
                "join_paths('C:\\usr', ['foo', 'C:\\bin\\echo'])",
                &mut types,
                scope,
            )
            .unwrap_err();
            assert_eq!(
                diagnostic.message(),
                "index 1 of the array is required to be a relative path, but an absolute path was \
                 provided"
            );

            let diagnostic = eval_v1_expr(
                V1::Two,
                "join_paths(['C:\\usr', 'foo', 'C:\\bin\\echo'])",
                &mut types,
                scope,
            )
            .unwrap_err();
            assert_eq!(
                diagnostic.message(),
                "index 2 of the array is required to be a relative path, but an absolute path was \
                 provided"
            );
        }
    }
}
