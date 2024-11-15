//! Module for the WDL standard library implementation.

use std::borrow::Cow;
use std::collections::HashMap;
use std::fs;
use std::io::BufRead;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::LazyLock;

use anyhow::Context;
use indexmap::IndexMap;
use itertools::EitherOrBoth;
use itertools::Itertools;
use regex::Regex;
use util::TsvHeader;
use util::is_ident;
use wdl_analysis::stdlib::Binding;
use wdl_analysis::stdlib::STDLIB as ANALYSIS_STDLIB;
use wdl_analysis::types::Optional;
use wdl_analysis::types::PrimitiveTypeKind;
use wdl_analysis::types::Type;
use wdl_analysis::types::TypeEq;
use wdl_ast::Diagnostic;
use wdl_ast::Span;

use crate::Array;
use crate::Coercible;
use crate::CompoundValue;
use crate::EvaluationContext;
use crate::PrimitiveValue;
use crate::StorageUnit;
use crate::Value;
use crate::diagnostics::array_path_not_relative;
use crate::diagnostics::function_call_failed;
use crate::diagnostics::invalid_glob_pattern;
use crate::diagnostics::invalid_regex;
use crate::diagnostics::invalid_storage_unit;
use crate::diagnostics::path_not_relative;

mod util;

/// Represents a function call argument.
pub struct CallArgument {
    /// The value of the argument.
    value: Value,
    /// The span of the expression of the argument.
    span: Span,
}

impl CallArgument {
    /// Constructs a new call argument given its value and span.
    pub const fn new(value: Value, span: Span) -> Self {
        Self { value, span }
    }

    /// Constructs a `None` call argument.
    pub const fn none() -> Self {
        Self {
            value: Value::None,
            span: Span::new(0, 0),
        }
    }
}

/// Represents function call context.
pub struct CallContext<'a> {
    /// The evaluation context for the call.
    context: &'a dyn EvaluationContext,
    /// The call site span.
    call_site: Span,
    /// The arguments to the call.
    arguments: &'a [CallArgument],
    /// The return type.
    return_type: Type,
}

impl<'a> CallContext<'a> {
    /// Constructs a new call context given the call arguments.
    pub fn new(
        context: &'a dyn EvaluationContext,
        call_site: Span,
        arguments: &'a [CallArgument],
        return_type: Type,
    ) -> Self {
        Self {
            context,
            call_site,
            arguments,
            return_type,
        }
    }

    /// Gets the current working directory for the call.
    pub fn cwd(&self) -> &Path {
        self.context.cwd()
    }

    /// Gets the temp directory for the call.
    pub fn tmp(&self) -> &Path {
        self.context.tmp()
    }

    /// Gets the stdout value for the call.
    pub fn stdout(&self) -> Option<Value> {
        self.context.stdout()
    }

    /// Gets the stderr value for the call.
    pub fn stderr(&self) -> Option<Value> {
        self.context.stderr()
    }

    /// Coerces an argument to the given type.
    ///
    /// # Panics
    ///
    /// Panics if the given index is out of range or if the value fails to
    /// coerce to the given type.
    #[inline]
    fn coerce_argument(&self, index: usize, ty: impl Into<Type>) -> Value {
        self.arguments[index]
            .value
            .coerce(self.context.types(), ty.into())
            .expect("value should coerce")
    }

    /// Checks to see if the calculated return type equals the given type.
    ///
    /// This is only used in assertions made by the function implementations.
    #[cfg(debug_assertions)]
    fn return_type_eq(&self, ty: impl Into<Type>) -> bool {
        self.return_type.type_eq(self.context.types(), &ty.into())
    }
}

/// Rounds a floating point number down to the next lower integer.
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#floor
pub fn floor(context: CallContext<'_>) -> Result<Value, Diagnostic> {
    debug_assert_eq!(context.arguments.len(), 1);
    debug_assert!(context.return_type_eq(PrimitiveTypeKind::Integer));

    let arg = context
        .coerce_argument(0, PrimitiveTypeKind::Float)
        .unwrap_float();
    Ok((arg.floor() as i64).into())
}

/// Rounds a floating point number up to the next higher integer.
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#ceil
pub fn ceil(context: CallContext<'_>) -> Result<Value, Diagnostic> {
    debug_assert_eq!(context.arguments.len(), 1);
    debug_assert!(context.return_type_eq(PrimitiveTypeKind::Integer));

    let arg = context
        .coerce_argument(0, PrimitiveTypeKind::Float)
        .unwrap_float();
    Ok((arg.ceil() as i64).into())
}

/// Rounds a floating point number to the nearest integer based on standard
/// rounding rules ("round half up").
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#round
pub fn round(context: CallContext<'_>) -> Result<Value, Diagnostic> {
    debug_assert_eq!(context.arguments.len(), 1);
    debug_assert!(context.return_type_eq(PrimitiveTypeKind::Integer));

    let arg = context
        .coerce_argument(0, PrimitiveTypeKind::Float)
        .unwrap_float();
    Ok((arg.round() as i64).into())
}

/// Returns the smaller of two integer values.
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#min
pub fn int_min(context: CallContext<'_>) -> Result<Value, Diagnostic> {
    debug_assert_eq!(context.arguments.len(), 2);
    debug_assert!(context.return_type_eq(PrimitiveTypeKind::Integer));

    let first = context
        .coerce_argument(0, PrimitiveTypeKind::Integer)
        .unwrap_integer();
    let second = context
        .coerce_argument(1, PrimitiveTypeKind::Integer)
        .unwrap_integer();
    Ok(first.min(second).into())
}

/// Returns the smaller of two float values.
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#min
pub fn float_min(context: CallContext<'_>) -> Result<Value, Diagnostic> {
    debug_assert_eq!(context.arguments.len(), 2);
    debug_assert!(context.return_type_eq(PrimitiveTypeKind::Float));

    let first = context
        .coerce_argument(0, PrimitiveTypeKind::Float)
        .unwrap_float();
    let second = context
        .coerce_argument(1, PrimitiveTypeKind::Float)
        .unwrap_float();
    Ok(first.min(second).into())
}

/// Returns the larger of two integer values.
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#max
pub fn int_max(context: CallContext<'_>) -> Result<Value, Diagnostic> {
    debug_assert_eq!(context.arguments.len(), 2);
    debug_assert!(context.return_type_eq(PrimitiveTypeKind::Integer));

    let first = context
        .coerce_argument(0, PrimitiveTypeKind::Integer)
        .unwrap_integer();
    let second = context
        .coerce_argument(1, PrimitiveTypeKind::Integer)
        .unwrap_integer();
    Ok(first.max(second).into())
}

/// Returns the larger of two float values.
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#max
pub fn float_max(context: CallContext<'_>) -> Result<Value, Diagnostic> {
    debug_assert_eq!(context.arguments.len(), 2);
    debug_assert!(context.return_type_eq(PrimitiveTypeKind::Float));

    let first = context
        .coerce_argument(0, PrimitiveTypeKind::Float)
        .unwrap_float();
    let second = context
        .coerce_argument(1, PrimitiveTypeKind::Float)
        .unwrap_float();
    Ok(first.max(second).into())
}

/// Given two String parameters `input` and `pattern`, searches for the
/// occurrence of `pattern` within `input` and returns the first match or `None`
/// if there are no matches.
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#-find
pub fn find(context: CallContext<'_>) -> Result<Value, Diagnostic> {
    debug_assert_eq!(context.arguments.len(), 2);
    debug_assert!(context.return_type_eq(Type::from(PrimitiveTypeKind::String).optional()));

    let input = context
        .coerce_argument(0, PrimitiveTypeKind::String)
        .unwrap_string();
    let pattern = context
        .coerce_argument(1, PrimitiveTypeKind::String)
        .unwrap_string();

    let regex =
        Regex::new(pattern.as_str()).map_err(|e| invalid_regex(&e, context.arguments[1].span))?;

    match regex.find(input.as_str()) {
        Some(m) => Ok(PrimitiveValue::new_string(m.as_str()).into()),
        None => Ok(Value::None),
    }
}

/// Given two String parameters `input` and `pattern`, tests whether `pattern`
/// matches `input` at least once.
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#-matches
pub fn matches(context: CallContext<'_>) -> Result<Value, Diagnostic> {
    debug_assert_eq!(context.arguments.len(), 2);
    debug_assert!(context.return_type_eq(PrimitiveTypeKind::Boolean));

    let input = context
        .coerce_argument(0, PrimitiveTypeKind::String)
        .unwrap_string();
    let pattern = context
        .coerce_argument(1, PrimitiveTypeKind::String)
        .unwrap_string();

    let regex =
        Regex::new(pattern.as_str()).map_err(|e| invalid_regex(&e, context.arguments[1].span))?;
    Ok(regex.is_match(input.as_str()).into())
}

/// Given three String parameters `input`, `pattern`, and `replace`, this
/// function replaces all non-overlapping occurrences of `pattern` in `input`
/// with `replace`.
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#sub
pub fn sub(context: CallContext<'_>) -> Result<Value, Diagnostic> {
    debug_assert_eq!(context.arguments.len(), 3);
    debug_assert!(context.return_type_eq(PrimitiveTypeKind::String));

    let input = context
        .coerce_argument(0, PrimitiveTypeKind::String)
        .unwrap_string();
    let pattern = context
        .coerce_argument(1, PrimitiveTypeKind::String)
        .unwrap_string();
    let replacement = context
        .coerce_argument(2, PrimitiveTypeKind::String)
        .unwrap_string();

    let regex =
        Regex::new(pattern.as_str()).map_err(|e| invalid_regex(&e, context.arguments[1].span))?;
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
pub fn basename(context: CallContext<'_>) -> Result<Value, Diagnostic> {
    debug_assert!(!context.arguments.is_empty() && context.arguments.len() < 3);
    debug_assert!(context.return_type_eq(PrimitiveTypeKind::String));

    let path = context
        .coerce_argument(0, PrimitiveTypeKind::String)
        .unwrap_string();

    match Path::new(path.as_str()).file_name() {
        Some(base) => {
            let base = base.to_str().expect("should be UTF-8");
            let base = if context.arguments.len() == 2 {
                base.strip_suffix(
                    context
                        .coerce_argument(1, PrimitiveTypeKind::String)
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
pub fn join_paths_simple(context: CallContext<'_>) -> Result<Value, Diagnostic> {
    debug_assert!(context.arguments.len() == 2);
    debug_assert!(context.return_type_eq(PrimitiveTypeKind::File));

    let first = context
        .coerce_argument(0, PrimitiveTypeKind::File)
        .unwrap_file();

    let second = context
        .coerce_argument(1, PrimitiveTypeKind::String)
        .unwrap_string();

    let second = Path::new(second.as_str());
    if !second.is_relative() {
        return Err(path_not_relative(context.arguments[1].span));
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
pub fn join_paths(context: CallContext<'_>) -> Result<Value, Diagnostic> {
    debug_assert!(!context.arguments.is_empty() && context.arguments.len() < 3);
    debug_assert!(context.return_type_eq(PrimitiveTypeKind::File));

    // Handle being provided one or two arguments
    let (first, array, skip, array_span) = if context.arguments.len() == 1 {
        let array = context
            .coerce_argument(0, ANALYSIS_STDLIB.array_string_non_empty_type())
            .unwrap_array();

        (
            array.elements()[0].clone().unwrap_string(),
            array,
            true,
            context.arguments[0].span,
        )
    } else {
        let first = context
            .coerce_argument(0, PrimitiveTypeKind::File)
            .unwrap_file();

        let array = context
            .coerce_argument(1, ANALYSIS_STDLIB.array_string_non_empty_type())
            .unwrap_array();

        (first, array, false, context.arguments[1].span)
    };

    let mut path = PathBuf::from(Arc::unwrap_or_clone(first));

    for (i, element) in array
        .elements()
        .iter()
        .enumerate()
        .skip(if skip { 1 } else { 0 })
    {
        let next = element.as_string().expect("element should be string");

        let next = Path::new(next.as_str());
        if !next.is_relative() {
            return Err(array_path_not_relative(i, array_span));
        }

        path.push(next);
    }

    Ok(PrimitiveValue::new_file(
        path.into_os_string()
            .into_string()
            .expect("should be UTF-8"),
    )
    .into())
}

/// Returns the Bash expansion of the glob string relative to the task's
/// execution directory, and in the same order.
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#glob
pub fn glob(context: CallContext<'_>) -> Result<Value, Diagnostic> {
    debug_assert!(context.arguments.len() == 1);
    debug_assert!(context.return_type_eq(ANALYSIS_STDLIB.array_file_type()));

    let path = context
        .coerce_argument(0, PrimitiveTypeKind::String)
        .unwrap_string();

    let mut elements: Vec<Value> = Vec::new();
    for path in glob::glob(&context.cwd().join(path.as_str()).to_string_lossy())
        .map_err(|e| invalid_glob_pattern(&e, context.arguments[0].span))?
    {
        let path = path.map_err(|e| function_call_failed("glob", &e, context.call_site))?;

        // Filter out directories (only files are returned from WDL's `glob` function)
        if path.is_dir() {
            continue;
        }

        // Strip the CWD prefix if there is one
        let path = match path.strip_prefix(context.cwd()) {
            Ok(path) => {
                // Create a string from the stripped path
                path.to_str()
                    .ok_or_else(|| {
                        function_call_failed(
                            "glob",
                            format!(
                                "path `{path}` cannot be represented as UTF-8",
                                path = path.display()
                            ),
                            context.call_site,
                        )
                    })?
                    .to_string()
            }
            Err(_) => {
                // Convert the path directly to a string
                path.into_os_string().into_string().map_err(|path| {
                    function_call_failed(
                        "glob",
                        format!(
                            "path `{path}` cannot be represented as UTF-8",
                            path = Path::new(&path).display()
                        ),
                        context.call_site,
                    )
                })?
            }
        };

        elements.push(PrimitiveValue::new_file(path).into());
    }

    Ok(Array::new_unchecked(context.return_type, Arc::new(elements.into_boxed_slice())).into())
}

/// Determines the size of a file, directory, or the sum total sizes of the
/// files/directories contained within a compound value. The files may be
/// optional values; None values have a size of 0.0. By default, the size is
/// returned in bytes unless the optional second argument is specified with a
/// unit.
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#glob
pub fn size(context: CallContext<'_>) -> Result<Value, Diagnostic> {
    debug_assert!(!context.arguments.is_empty() && context.arguments.len() < 3);
    debug_assert!(context.return_type_eq(PrimitiveTypeKind::Float));

    let unit = if context.arguments.len() == 2 {
        let unit = context
            .coerce_argument(1, PrimitiveTypeKind::String)
            .unwrap_string();

        unit.parse()
            .map_err(|_| invalid_storage_unit(&unit, context.arguments[1].span))?
    } else {
        StorageUnit::default()
    };

    // If the first argument is a string, we need to check if it's a file or
    // directory and treat it as such.
    let value = if let Some(s) = context.arguments[0].value.as_string() {
        let path = context.cwd().join(s.as_str());
        let metadata = path
            .metadata()
            .with_context(|| {
                format!(
                    "failed to read metadata for file `{path}`",
                    path = path.display()
                )
            })
            .map_err(|e| function_call_failed("size", format!("{e:?}"), context.call_site))?;
        if metadata.is_dir() {
            PrimitiveValue::Directory(s.clone()).into()
        } else {
            PrimitiveValue::File(s.clone()).into()
        }
    } else {
        context.arguments[0].value.clone()
    };

    util::calculate_disk_size(&value, unit, context.cwd())
        .map_err(|e| function_call_failed("size", format!("{e:?}"), context.call_site))
        .map(Into::into)
}

/// Returns the value of the executed command's standard output (stdout) as a
/// File.
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#stdout
pub fn stdout(context: CallContext<'_>) -> Result<Value, Diagnostic> {
    debug_assert!(context.arguments.is_empty());
    debug_assert!(context.return_type_eq(PrimitiveTypeKind::File));

    match context.stdout() {
        Some(stdout) => {
            debug_assert!(
                stdout.as_file().is_some(),
                "expected the value to be a file"
            );
            Ok(stdout)
        }
        None => Err(function_call_failed(
            "stdout",
            "function may only be called in a task output section",
            context.call_site,
        )),
    }
}

/// Returns the value of the executed command's standard error (stderr) as a
/// File
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#stderr
pub fn stderr(context: CallContext<'_>) -> Result<Value, Diagnostic> {
    debug_assert!(context.arguments.is_empty());
    debug_assert!(context.return_type_eq(PrimitiveTypeKind::File));

    match context.stderr() {
        Some(stderr) => {
            debug_assert!(
                stderr.as_file().is_some(),
                "expected the value to be a file"
            );
            Ok(stderr)
        }
        None => Err(function_call_failed(
            "stderr",
            "function may only be called in a task output section",
            context.call_site,
        )),
    }
}

/// Reads an entire file as a String, with any trailing end-of-line characters
/// (\r and \n) stripped off.
///
/// If the file is empty, an empty string is returned.
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#read_string
pub fn read_string(context: CallContext<'_>) -> Result<Value, Diagnostic> {
    debug_assert!(context.arguments.len() == 1);
    debug_assert!(context.return_type_eq(PrimitiveTypeKind::String));

    let path = context.cwd().join(
        context
            .coerce_argument(0, PrimitiveTypeKind::File)
            .unwrap_file()
            .as_str(),
    );
    let mut contents = fs::read_to_string(&path)
        .with_context(|| format!("failed to read file `{path}`", path = path.display()))
        .map_err(|e| function_call_failed("read_string", format!("{e:?}"), context.call_site))?;

    let trimmed = contents.trim_end_matches(['\r', '\n']);
    contents.truncate(trimmed.len());
    Ok(PrimitiveValue::new_string(contents).into())
}

/// Reads a file that contains a single line containing only an integer and
/// (optional) whitespace.
///
/// If the line contains a valid integer, that value is returned as an Int. If
/// the file is empty or does not contain a single integer, an error is raised.
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#read_int
pub fn read_int(context: CallContext<'_>) -> Result<Value, Diagnostic> {
    debug_assert!(context.arguments.len() == 1);
    debug_assert!(context.return_type_eq(PrimitiveTypeKind::Integer));

    let path = context.cwd().join(
        context
            .coerce_argument(0, PrimitiveTypeKind::File)
            .unwrap_file()
            .as_str(),
    );
    let contents = fs::read_to_string(&path)
        .with_context(|| format!("failed to read file `{path}`", path = path.display()))
        .map_err(|e| function_call_failed("read_int", format!("{e:?}"), context.call_site))?;

    Ok(contents
        .trim()
        .parse::<i64>()
        .map_err(|_| {
            function_call_failed(
                "read_int",
                format!(
                    "file `{path}` does not contain a single integer value",
                    path = path.display()
                ),
                context.call_site,
            )
        })?
        .into())
}

/// Reads a file that contains only a float value and (optional) whitespace.
///
/// If the line contains a valid floating point number, that value is returned
/// as a Float. If the file is empty or does not contain a single float, an
/// error is raised.
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#read_float
pub fn read_float(context: CallContext<'_>) -> Result<Value, Diagnostic> {
    debug_assert!(context.arguments.len() == 1);
    debug_assert!(context.return_type_eq(PrimitiveTypeKind::Float));

    let path = context.cwd().join(
        context
            .coerce_argument(0, PrimitiveTypeKind::File)
            .unwrap_file()
            .as_str(),
    );
    let contents = fs::read_to_string(&path)
        .with_context(|| format!("failed to read file `{path}`", path = path.display()))
        .map_err(|e| function_call_failed("read_float", format!("{e:?}"), context.call_site))?;

    Ok(contents
        .trim()
        .parse::<f64>()
        .map_err(|_| {
            function_call_failed(
                "read_float",
                format!(
                    "file `{path}` does not contain a single float value",
                    path = path.display()
                ),
                context.call_site,
            )
        })?
        .into())
}

/// Reads a file that contains a single line containing only a boolean value and
/// (optional) whitespace.
///
/// If the non-whitespace content of the line is "true" or "false", that value
/// is returned as a Boolean. If the file is empty or does not contain a single
/// boolean, an error is raised. The comparison is case-insensitive.
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#read_boolean
pub fn read_boolean(context: CallContext<'_>) -> Result<Value, Diagnostic> {
    debug_assert!(context.arguments.len() == 1);
    debug_assert!(context.return_type_eq(PrimitiveTypeKind::Boolean));

    let path = context.cwd().join(
        context
            .coerce_argument(0, PrimitiveTypeKind::File)
            .unwrap_file()
            .as_str(),
    );
    let mut contents = fs::read_to_string(&path)
        .with_context(|| format!("failed to read file `{path}`", path = path.display()))
        .map_err(|e| function_call_failed("read_boolean", format!("{e:?}"), context.call_site))?;

    contents.make_ascii_lowercase();

    Ok(contents
        .trim()
        .parse::<bool>()
        .map_err(|_| {
            function_call_failed(
                "read_boolean",
                format!(
                    "file `{path}` does not contain a single boolean value",
                    path = path.display()
                ),
                context.call_site,
            )
        })?
        .into())
}

/// Reads each line of a file as a String, and returns all lines in the file as
/// an Array[String].
///
/// Trailing end-of-line characters (\r and \n) are removed from each line.
///
/// The order of the lines in the returned Array[String] is the order in which
/// the lines appear in the file.
///
/// If the file is empty, an empty array is returned.
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#read_lines
pub fn read_lines(context: CallContext<'_>) -> Result<Value, Diagnostic> {
    debug_assert!(context.arguments.len() == 1);
    debug_assert!(context.return_type_eq(ANALYSIS_STDLIB.array_string_type()));

    let path = context.cwd().join(
        context
            .coerce_argument(0, PrimitiveTypeKind::File)
            .unwrap_file()
            .as_str(),
    );

    let file = fs::File::open(&path)
        .with_context(|| format!("failed to open file `{path}`", path = path.display()))
        .map_err(|e| function_call_failed("read_lines", format!("{e:?}"), context.call_site))?;

    let elements = BufReader::new(file)
        .lines()
        .map(|line| {
            let mut line = line
                .with_context(|| format!("failed to read file `{path}`", path = path.display()))
                .map_err(|e| {
                    function_call_failed("read_lines", format!("{e:?}"), context.call_site)
                })?;

            let trimmed = line.trim_end_matches(['\r', '\n']);
            line.truncate(trimmed.len());
            Ok(PrimitiveValue::new_string(line).into())
        })
        .collect::<Result<Vec<Value>, _>>()?;

    Ok(Array::new_unchecked(context.return_type, Arc::new(elements.into_boxed_slice())).into())
}

/// Writes a file with one line for each element in a Array[String].
///
/// All lines are terminated by the newline (\n) character (following the POSIX
/// standard).
///
/// If the Array is empty, an empty file is written.
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#write_lines
pub fn write_lines(context: CallContext<'_>) -> Result<Value, Diagnostic> {
    debug_assert!(context.arguments.len() == 1);
    debug_assert!(context.return_type_eq(PrimitiveTypeKind::File));

    // Helper for handling errors while writing to the file.
    let write_error = |e: std::io::Error| {
        function_call_failed(
            "write_lines",
            format!("failed to write to temporary file: {e}"),
            context.call_site,
        )
    };

    let lines = context
        .coerce_argument(0, ANALYSIS_STDLIB.array_string_type())
        .unwrap_array();

    // Create a temporary file that will be persisted after writing the lines
    let mut file = tempfile::NamedTempFile::new_in(context.tmp()).map_err(|e| {
        function_call_failed(
            "write_lines",
            format!("failed to create temporary file: {e}"),
            context.call_site,
        )
    })?;

    // Write the lines
    let mut writer = BufWriter::new(file.as_file_mut());
    for line in lines.elements() {
        writer
            .write(line.as_string().unwrap().as_str().as_bytes())
            .map_err(write_error)?;
        writer.write(b"\n").map_err(write_error)?;
    }

    // Consume the writer, flushing the buffer to disk.
    writer
        .into_inner()
        .map_err(|e| write_error(e.into_error()))?;

    let (_, path) = file.keep().map_err(|e| {
        function_call_failed(
            "write_lines",
            format!("failed to keep temporary file: {e}"),
            context.call_site,
        )
    })?;

    Ok(
        PrimitiveValue::new_file(path.into_os_string().into_string().map_err(|path| {
            function_call_failed(
                "write_lines",
                format!(
                    "path `{path}` cannot be represented as UTF-8",
                    path = Path::new(&path).display()
                ),
                context.call_site,
            )
        })?)
        .into(),
    )
}

/// Reads a tab-separated value (TSV) file as an Array[Array[String]]
/// representing a table of values.
///
/// Trailing end-of-line characters (\r and \n) are removed from each line.
///
/// `Array[Array[String]] read_tsv(File, [false])``: Returns each row of the
/// table as an Array[String]. There is no requirement that the rows of the
/// table are all the same length.
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#read_tsv
pub fn read_tsv_simple(context: CallContext<'_>) -> Result<Value, Diagnostic> {
    debug_assert!(context.arguments.len() == 1);
    debug_assert!(context.return_type_eq(ANALYSIS_STDLIB.array_array_string_type()));

    let path = context.cwd().join(
        context
            .coerce_argument(0, PrimitiveTypeKind::File)
            .unwrap_file()
            .as_str(),
    );

    let file = fs::File::open(&path)
        .with_context(|| format!("failed to open file `{path}`", path = path.display()))
        .map_err(|e| function_call_failed("read_tsv", format!("{e:?}"), context.call_site))?;

    let mut rows: Vec<Value> = Vec::new();
    for line in BufReader::new(file).lines() {
        let line = line
            .with_context(|| format!("failed to read file `{path}`", path = path.display()))
            .map_err(|e| function_call_failed("read_tsv", format!("{e:?}"), context.call_site))?;
        let values = line
            .trim_end_matches(['\r', '\n'])
            .split('\t')
            .map(|s| PrimitiveValue::new_string(s).into())
            .collect::<Vec<Value>>()
            .into_boxed_slice();
        rows.push(
            Array::new_unchecked(ANALYSIS_STDLIB.array_string_type(), Arc::new(values)).into(),
        );
    }

    Ok(Array::new_unchecked(
        ANALYSIS_STDLIB.array_array_string_type(),
        Arc::new(rows.into_boxed_slice()),
    )
    .into())
}

/// Reads a tab-separated value (TSV) file as an Array[Array[String]]
/// representing a table of values.
///
/// Trailing end-of-line characters (\r and \n) are removed from each line.
///
/// `Array[Object] read_tsv(File, true)``: The second parameter must be true and
/// specifies that the TSV file contains a header line. Each row is returned as
/// an Object with its keys determined by the header (the first line in the
/// file) and its values as Strings. All rows in the file must be the same
/// length and the field names in the header row must be valid Object field
/// names, or an error is raised.
///
/// `Array[Object] read_tsv(File, Boolean, Array[String])``: The second
/// parameter specifies whether the TSV file contains a header line, and the
/// third parameter is an array of field names that is used to specify the field
/// names to use for the returned Objects. If the second parameter is true, the
/// specified field names override those in the file's header (i.e., the header
/// line is ignored).
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#read_tsv
pub fn read_tsv(context: CallContext<'_>) -> Result<Value, Diagnostic> {
    debug_assert!(context.arguments.len() >= 2 && context.arguments.len() <= 3);
    debug_assert!(context.return_type_eq(ANALYSIS_STDLIB.array_object_type()));

    let path = context.cwd().join(
        context
            .coerce_argument(0, PrimitiveTypeKind::File)
            .unwrap_file()
            .as_str(),
    );

    let file = fs::File::open(&path)
        .with_context(|| format!("failed to open file `{path}`", path = path.display()))
        .map_err(|e| function_call_failed("read_tsv", format!("{e:?}"), context.call_site))?;

    let mut lines = BufReader::new(file).lines();

    // Read the file header if there is one; ignore it if the header was directly
    // specified.
    let file_has_header = context
        .coerce_argument(1, PrimitiveTypeKind::Boolean)
        .unwrap_boolean();
    let header = if context.arguments.len() == 3 {
        if file_has_header {
            lines.next();
        }

        TsvHeader::Specified(
            context
                .coerce_argument(2, ANALYSIS_STDLIB.array_string_type())
                .unwrap_array(),
        )
    } else if !file_has_header {
        return Err(function_call_failed(
            "read_tsv",
            "argument specifying presence of a file header must be `true`",
            context.arguments[1].span,
        ));
    } else {
        TsvHeader::File(
            lines
                .next()
                .unwrap_or_else(|| Ok(String::default()))
                .with_context(|| format!("failed to read file `{path}`", path = path.display()))
                .map_err(|e| {
                    function_call_failed("read_tsv", format!("{e:?}"), context.call_site)
                })?,
        )
    };

    if let Some(invalid) = header.columns().find(|c| !is_ident(c)) {
        return Err(function_call_failed(
            "read_tsv",
            format!("column name `{invalid}` is not a valid WDL object field name"),
            context.call_site,
        ));
    }

    let mut rows: Vec<Value> = Vec::new();
    for (index, line) in lines.enumerate() {
        let line = line
            .with_context(|| format!("failed to read file `{path}`", path = path.display()))
            .map_err(|e| function_call_failed("read_tsv", format!("{e:?}"), context.call_site))?;

        let members = header
            .columns()
            .zip_longest(line.trim_end_matches(['\r', '\n']).split('\t'))
            .map(|e| match e {
                EitherOrBoth::Both(c, v) => {
                    Ok((c.to_string(), PrimitiveValue::new_string(v).into()))
                }
                _ => Err(function_call_failed(
                    "read_tsv",
                    format!(
                        "line {index} in file `{path}` does not have the expected number of \
                         columns",
                        index = index + 1 + if file_has_header { 1 } else { 0 },
                        path = path.display()
                    ),
                    context.call_site,
                )),
            })
            .collect::<Result<IndexMap<_, _>, _>>()?;

        rows.push(CompoundValue::Object(members.into()).into());
    }

    Ok(Array::new_unchecked(
        ANALYSIS_STDLIB.array_object_type(),
        Arc::new(rows.into_boxed_slice()),
    )
    .into())
}

/// Represents a WDL function implementation callback.
type Callback = fn(context: CallContext<'_>) -> Result<Value, Diagnostic>;

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

    /// Calls the function given the binding and call context.
    #[inline]
    pub fn call(
        &self,
        binding: Binding<'_>,
        context: CallContext<'_>,
    ) -> Result<Value, Diagnostic> {
        (self.signatures[binding.index()].callback)(context)
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
    let mut functions = HashMap::with_capacity(ANALYSIS_STDLIB.functions().len());
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
    assert!(
        functions
            .insert(
                "glob",
                Function::new(const { &[Signature::new("(String) -> Array[File]", glob)] })
            )
            .is_none()
    );
    assert!(
        functions
            .insert(
                "size",
                Function::new(
                    const {
                        &[
                            Signature::new("(None, <String>) -> Float", size),
                            Signature::new("(File?, <String>) -> Float", size),
                            Signature::new("(String?, <String>) -> Float", size),
                            Signature::new("(Directory?, <String>) -> Float", size),
                            Signature::new(
                                "(X, <String>) -> Float where `X`: any compound type that \
                                 recursively contains a `File` or `Directory`",
                                size,
                            ),
                        ]
                    }
                )
            )
            .is_none()
    );
    assert!(
        functions
            .insert(
                "stdout",
                Function::new(const { &[Signature::new("() -> File", stdout)] })
            )
            .is_none()
    );
    assert!(
        functions
            .insert(
                "stderr",
                Function::new(const { &[Signature::new("() -> File", stderr)] })
            )
            .is_none()
    );
    assert!(
        functions
            .insert(
                "read_string",
                Function::new(const { &[Signature::new("(File) -> String", read_string)] })
            )
            .is_none()
    );
    assert!(
        functions
            .insert(
                "read_int",
                Function::new(const { &[Signature::new("(File) -> Int", read_int)] })
            )
            .is_none()
    );
    assert!(
        functions
            .insert(
                "read_float",
                Function::new(const { &[Signature::new("(File) -> Float", read_float)] })
            )
            .is_none()
    );
    assert!(
        functions
            .insert(
                "read_boolean",
                Function::new(const { &[Signature::new("(File) -> Boolean", read_boolean)] })
            )
            .is_none()
    );
    assert!(
        functions
            .insert(
                "read_lines",
                Function::new(const { &[Signature::new("(File) -> Array[String]", read_lines)] })
            )
            .is_none()
    );
    assert!(
        functions
            .insert(
                "write_lines",
                Function::new(const { &[Signature::new("(Array[String]) -> File", write_lines)] })
            )
            .is_none()
    );
    assert!(
        functions
            .insert(
                "read_tsv",
                Function::new(
                    const {
                        &[
                            Signature::new("(File) -> Array[Array[String]]", read_tsv_simple),
                            Signature::new("(File, Boolean) -> Array[Object]", read_tsv),
                            Signature::new(
                                "(File, Boolean, Array[String]) -> Array[Object]",
                                read_tsv,
                            ),
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
    use std::fs;

    use pretty_assertions::assert_eq;
    use wdl_analysis::stdlib::TypeParameters;
    use wdl_ast::version::V1;

    use super::*;
    use crate::v1::test::TestEnv;
    use crate::v1::test::eval_v1_expr;
    use crate::v1::test::eval_v1_expr_with_stdio;

    /// A test to verify that the STDLIB function types from `wdl-analysis`
    /// aligns with the STDLIB implementation from `wdl-engine`.
    #[test]
    fn verify_stdlib() {
        for (name, func) in ANALYSIS_STDLIB.functions() {
            match STDLIB.functions.get(name) {
                Some(imp) => match func {
                    wdl_analysis::stdlib::Function::Monomorphic(f) => {
                        assert_eq!(
                            imp.signatures.len(),
                            1,
                            "signature count mismatch for function `{name}`"
                        );
                        assert_eq!(
                            f.signature()
                                .display(
                                    ANALYSIS_STDLIB.types(),
                                    &TypeParameters::new(f.signature().type_parameters())
                                )
                                .to_string(),
                            imp.signatures[0].display,
                            "signature mismatch for function `{name}`"
                        );
                    }
                    wdl_analysis::stdlib::Function::Polymorphic(f) => {
                        assert_eq!(
                            imp.signatures.len(),
                            f.signatures().len(),
                            "signature count mismatch for function `{name}`"
                        );
                        for (i, sig) in f.signatures().iter().enumerate() {
                            assert_eq!(
                                sig.display(
                                    ANALYSIS_STDLIB.types(),
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
        let mut env = TestEnv::default();
        let value = eval_v1_expr(&mut env, V1::Zero, "floor(10.5)").unwrap();
        assert_eq!(value.unwrap_integer(), 10);

        let value = eval_v1_expr(&mut env, V1::Zero, "floor(10)").unwrap();
        assert_eq!(value.unwrap_integer(), 10);

        let value = eval_v1_expr(&mut env, V1::Zero, "floor(9.9999)").unwrap();
        assert_eq!(value.unwrap_integer(), 9);

        let value = eval_v1_expr(&mut env, V1::Zero, "floor(0)").unwrap();
        assert_eq!(value.unwrap_integer(), 0);

        let value = eval_v1_expr(&mut env, V1::Zero, "floor(-5.1)").unwrap();
        assert_eq!(value.unwrap_integer(), -6);
    }

    #[test]
    fn ceil() {
        let mut env = TestEnv::default();
        let value = eval_v1_expr(&mut env, V1::Zero, "ceil(10.5)").unwrap();
        assert_eq!(value.unwrap_integer(), 11);

        let value = eval_v1_expr(&mut env, V1::Zero, "ceil(10)").unwrap();
        assert_eq!(value.unwrap_integer(), 10);

        let value = eval_v1_expr(&mut env, V1::Zero, "ceil(9.9999)").unwrap();
        assert_eq!(value.unwrap_integer(), 10);

        let value = eval_v1_expr(&mut env, V1::Zero, "ceil(0)").unwrap();
        assert_eq!(value.unwrap_integer(), 0);

        let value = eval_v1_expr(&mut env, V1::Zero, "ceil(-5.1)").unwrap();
        assert_eq!(value.unwrap_integer(), -5);
    }

    #[test]
    fn round() {
        let mut env = TestEnv::default();
        let value = eval_v1_expr(&mut env, V1::Zero, "round(10.5)").unwrap();
        assert_eq!(value.unwrap_integer(), 11);

        let value = eval_v1_expr(&mut env, V1::Zero, "round(10.3)").unwrap();
        assert_eq!(value.unwrap_integer(), 10);

        let value = eval_v1_expr(&mut env, V1::Zero, "round(10)").unwrap();
        assert_eq!(value.unwrap_integer(), 10);

        let value = eval_v1_expr(&mut env, V1::Zero, "round(9.9999)").unwrap();
        assert_eq!(value.unwrap_integer(), 10);

        let value = eval_v1_expr(&mut env, V1::Zero, "round(9.12345)").unwrap();
        assert_eq!(value.unwrap_integer(), 9);

        let value = eval_v1_expr(&mut env, V1::Zero, "round(0)").unwrap();
        assert_eq!(value.unwrap_integer(), 0);

        let value = eval_v1_expr(&mut env, V1::Zero, "round(-5.1)").unwrap();
        assert_eq!(value.unwrap_integer(), -5);

        let value = eval_v1_expr(&mut env, V1::Zero, "round(-5.5)").unwrap();
        assert_eq!(value.unwrap_integer(), -6);
    }

    #[test]
    fn min() {
        let mut env = TestEnv::default();
        let value = eval_v1_expr(&mut env, V1::One, "min(7, 42)").unwrap();
        assert_eq!(value.unwrap_integer(), 7);

        let value = eval_v1_expr(&mut env, V1::One, "min(42, 7)").unwrap();
        assert_eq!(value.unwrap_integer(), 7);

        let value = eval_v1_expr(&mut env, V1::One, "min(-42, 7)").unwrap();
        assert_eq!(value.unwrap_integer(), -42);

        let value = eval_v1_expr(&mut env, V1::One, "min(0, -42)").unwrap();
        assert_eq!(value.unwrap_integer(), -42);

        let value = eval_v1_expr(&mut env, V1::One, "min(0, 42)").unwrap();
        assert_eq!(value.unwrap_integer(), 0);

        let value = eval_v1_expr(&mut env, V1::One, "min(7.0, 42)").unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 7.0);

        let value = eval_v1_expr(&mut env, V1::One, "min(42.0, 7)").unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 7.0);

        let value = eval_v1_expr(&mut env, V1::One, "min(-42.0, 7)").unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), -42.0);

        let value = eval_v1_expr(&mut env, V1::One, "min(0.0, -42)").unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), -42.0);

        let value = eval_v1_expr(&mut env, V1::One, "min(0.0, 42)").unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), -0.0);

        let value = eval_v1_expr(&mut env, V1::One, "min(7, 42.0)").unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 7.0);

        let value = eval_v1_expr(&mut env, V1::One, "min(42, 7.0)").unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 7.0);

        let value = eval_v1_expr(&mut env, V1::One, "min(-42, 7.0)").unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), -42.0);

        let value = eval_v1_expr(&mut env, V1::One, "min(0, -42.0)").unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), -42.0);

        let value = eval_v1_expr(&mut env, V1::One, "min(0, 42.0)").unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), -0.0);

        let value = eval_v1_expr(&mut env, V1::One, "min(7.0, 42.0)").unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 7.0);

        let value = eval_v1_expr(&mut env, V1::One, "min(42.0, 7.0)").unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 7.0);

        let value = eval_v1_expr(&mut env, V1::One, "min(-42.0, 7.0)").unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), -42.0);

        let value = eval_v1_expr(&mut env, V1::One, "min(0.0, -42.0)").unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), -42.0);

        let value = eval_v1_expr(&mut env, V1::One, "min(0.0, 42.0)").unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), -0.0);

        let value = eval_v1_expr(
            &mut env,
            V1::One,
            "min(12345, min(-100, min(54321, 1234.5678)))",
        )
        .unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), -100.0);
    }

    #[test]
    fn max() {
        let mut env = TestEnv::default();
        let value = eval_v1_expr(&mut env, V1::One, "max(7, 42)").unwrap();
        assert_eq!(value.unwrap_integer(), 42);

        let value = eval_v1_expr(&mut env, V1::One, "max(42, 7)").unwrap();
        assert_eq!(value.unwrap_integer(), 42);

        let value = eval_v1_expr(&mut env, V1::One, "max(-42, 7)").unwrap();
        assert_eq!(value.unwrap_integer(), 7);

        let value = eval_v1_expr(&mut env, V1::One, "max(0, -42)").unwrap();
        assert_eq!(value.unwrap_integer(), 0);

        let value = eval_v1_expr(&mut env, V1::One, "max(0, 42)").unwrap();
        assert_eq!(value.unwrap_integer(), 42);

        let value = eval_v1_expr(&mut env, V1::One, "max(7.0, 42)").unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 42.0);

        let value = eval_v1_expr(&mut env, V1::One, "max(42.0, 7)").unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 42.0);

        let value = eval_v1_expr(&mut env, V1::One, "max(-42.0, 7)").unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 7.0);

        let value = eval_v1_expr(&mut env, V1::One, "max(0.0, -42)").unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 0.0);

        let value = eval_v1_expr(&mut env, V1::One, "max(0.0, 42)").unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 42.0);

        let value = eval_v1_expr(&mut env, V1::One, "max(7, 42.0)").unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 42.0);

        let value = eval_v1_expr(&mut env, V1::One, "max(42, 7.0)").unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 42.0);

        let value = eval_v1_expr(&mut env, V1::One, "max(-42, 7.0)").unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 7.0);

        let value = eval_v1_expr(&mut env, V1::One, "max(0, -42.0)").unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 0.0);

        let value = eval_v1_expr(&mut env, V1::One, "max(0, 42.0)").unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 42.0);

        let value = eval_v1_expr(&mut env, V1::One, "max(7.0, 42.0)").unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 42.0);

        let value = eval_v1_expr(&mut env, V1::One, "max(42.0, 7.0)").unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 42.0);

        let value = eval_v1_expr(&mut env, V1::One, "max(-42.0, 7.0)").unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 7.0);

        let value = eval_v1_expr(&mut env, V1::One, "max(0.0, -42.0)").unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 0.0);

        let value = eval_v1_expr(&mut env, V1::One, "max(0.0, 42.0)").unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 42.0);

        let value = eval_v1_expr(
            &mut env,
            V1::One,
            "max(12345, max(-100, max(54321, 1234.5678)))",
        )
        .unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 54321.0);
    }

    #[test]
    fn find() {
        let mut env = TestEnv::default();
        let diagnostic = eval_v1_expr(&mut env, V1::Two, "find('foo bar baz', '?')").unwrap_err();
        assert_eq!(
            diagnostic.message(),
            "regex parse error:\n    ?\n    ^\nerror: repetition operator missing expression"
        );

        let value = eval_v1_expr(&mut env, V1::Two, "find('hello world', 'e..o')").unwrap();
        assert_eq!(value.unwrap_string().as_str(), "ello");

        let value = eval_v1_expr(&mut env, V1::Two, "find('hello world', 'goodbye')").unwrap();
        assert!(value.is_none());

        let value = eval_v1_expr(&mut env, V1::Two, "find('hello\tBob', '\\t')").unwrap();
        assert_eq!(value.unwrap_string().as_str(), "\t");
    }

    #[test]
    fn matches() {
        let mut env = TestEnv::default();
        let diagnostic =
            eval_v1_expr(&mut env, V1::Two, "matches('foo bar baz', '?')").unwrap_err();
        assert_eq!(
            diagnostic.message(),
            "regex parse error:\n    ?\n    ^\nerror: repetition operator missing expression"
        );

        let value = eval_v1_expr(&mut env, V1::Two, "matches('hello world', 'e..o')").unwrap();
        assert!(value.unwrap_boolean());

        let value = eval_v1_expr(&mut env, V1::Two, "matches('hello world', 'goodbye')").unwrap();
        assert!(!value.unwrap_boolean());

        let value = eval_v1_expr(&mut env, V1::Two, "matches('hello\tBob', '\\t')").unwrap();
        assert!(value.unwrap_boolean());
    }

    #[test]
    fn sub() {
        let mut env = TestEnv::default();
        let diagnostic =
            eval_v1_expr(&mut env, V1::Two, "sub('foo bar baz', '?', 'nope')").unwrap_err();
        assert_eq!(
            diagnostic.message(),
            "regex parse error:\n    ?\n    ^\nerror: repetition operator missing expression"
        );

        let value =
            eval_v1_expr(&mut env, V1::Two, "sub('hello world', 'e..o', 'ey there')").unwrap();
        assert_eq!(value.unwrap_string().as_str(), "hey there world");

        let value =
            eval_v1_expr(&mut env, V1::Two, "sub('hello world', 'goodbye', 'nope')").unwrap();
        assert_eq!(value.unwrap_string().as_str(), "hello world");

        let value = eval_v1_expr(&mut env, V1::Two, "sub('hello\tBob', '\\t', ' ')").unwrap();
        assert_eq!(value.unwrap_string().as_str(), "hello Bob");
    }

    #[test]
    fn basename() {
        let mut env = TestEnv::default();
        let value = eval_v1_expr(&mut env, V1::Two, "basename('/path/to/file.txt')").unwrap();
        assert_eq!(value.unwrap_string().as_str(), "file.txt");

        let value =
            eval_v1_expr(&mut env, V1::Two, "basename('/path/to/file.txt', '.txt')").unwrap();
        assert_eq!(value.unwrap_string().as_str(), "file");

        let value = eval_v1_expr(&mut env, V1::Two, "basename('/path/to/dir')").unwrap();
        assert_eq!(value.unwrap_string().as_str(), "dir");

        let value = eval_v1_expr(&mut env, V1::Two, "basename('file.txt')").unwrap();
        assert_eq!(value.unwrap_string().as_str(), "file.txt");

        let value = eval_v1_expr(&mut env, V1::Two, "basename('file.txt', '.txt')").unwrap();
        assert_eq!(value.unwrap_string().as_str(), "file");
    }

    #[test]
    fn join_paths() {
        let mut env = TestEnv::default();
        let value = eval_v1_expr(&mut env, V1::Two, "join_paths('/usr', ['bin', 'echo'])").unwrap();
        assert_eq!(value.unwrap_file().as_str(), "/usr/bin/echo");

        let value = eval_v1_expr(&mut env, V1::Two, "join_paths(['/usr', 'bin', 'echo'])").unwrap();
        assert_eq!(value.unwrap_file().as_str(), "/usr/bin/echo");

        let value = eval_v1_expr(&mut env, V1::Two, "join_paths('mydir', 'mydata.txt')").unwrap();
        assert_eq!(value.unwrap_file().as_str(), "mydir/mydata.txt");

        let value = eval_v1_expr(&mut env, V1::Two, "join_paths('/usr', 'bin/echo')").unwrap();
        assert_eq!(value.unwrap_file().as_str(), "/usr/bin/echo");

        #[cfg(unix)]
        {
            let diagnostic =
                eval_v1_expr(&mut env, V1::Two, "join_paths('/usr', '/bin/echo')").unwrap_err();
            assert_eq!(
                diagnostic.message(),
                "path is required to be a relative path, but an absolute path was provided"
            );

            let diagnostic = eval_v1_expr(
                &mut env,
                V1::Two,
                "join_paths('/usr', ['foo', '/bin/echo'])",
            )
            .unwrap_err();
            assert_eq!(
                diagnostic.message(),
                "index 1 of the array is required to be a relative path, but an absolute path was \
                 provided"
            );

            let diagnostic = eval_v1_expr(
                &mut env,
                V1::Two,
                "join_paths(['/usr', 'foo', '/bin/echo'])",
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
            let diagnostic =
                eval_v1_expr(&mut env, V1::Two, "join_paths('C:\\usr', 'C:\\bin\\echo')")
                    .unwrap_err();
            assert_eq!(
                diagnostic.message(),
                "path is required to be a relative path, but an absolute path was provided"
            );

            let diagnostic = eval_v1_expr(
                &mut env,
                V1::Two,
                "join_paths('C:\\usr', ['foo', 'C:\\bin\\echo'])",
            )
            .unwrap_err();
            assert_eq!(
                diagnostic.message(),
                "index 1 of the array is required to be a relative path, but an absolute path was \
                 provided"
            );

            let diagnostic = eval_v1_expr(
                &mut env,
                V1::Two,
                "join_paths(['C:\\usr', 'foo', 'C:\\bin\\echo'])",
            )
            .unwrap_err();
            assert_eq!(
                diagnostic.message(),
                "index 2 of the array is required to be a relative path, but an absolute path was \
                 provided"
            );
        }
    }

    #[test]
    fn glob() {
        let mut env = TestEnv::default();
        let diagnostic = eval_v1_expr(&mut env, V1::Two, "glob('invalid***')").unwrap_err();
        assert_eq!(
            diagnostic.message(),
            "invalid glob pattern specified: wildcards are either regular `*` or recursive `**`"
        );

        env.write_file("qux", "qux");
        env.write_file("baz", "baz");
        env.write_file("foo", "foo");
        env.write_file("bar", "bar");
        fs::create_dir_all(env.cwd().join("nested")).expect("failed to create directory");
        env.write_file("nested/bar", "bar");
        env.write_file("nested/baz", "baz");

        let value = eval_v1_expr(&mut env, V1::Two, "glob('jam')").unwrap();
        let elements: Vec<_> = value
            .as_array()
            .unwrap()
            .elements()
            .iter()
            .map(|v| v.as_file().unwrap().as_str())
            .collect();
        assert!(elements.is_empty());

        let value = eval_v1_expr(&mut env, V1::Two, "glob('*')").unwrap();
        let elements: Vec<_> = value
            .as_array()
            .unwrap()
            .elements()
            .iter()
            .map(|v| v.as_file().unwrap().as_str())
            .collect();
        assert_eq!(elements, ["bar", "baz", "foo", "qux"]);

        let value = eval_v1_expr(&mut env, V1::Two, "glob('ba?')").unwrap();
        let elements: Vec<_> = value
            .as_array()
            .unwrap()
            .elements()
            .iter()
            .map(|v| v.as_file().unwrap().as_str())
            .collect();
        assert_eq!(elements, ["bar", "baz"]);

        let value = eval_v1_expr(&mut env, V1::Two, "glob('b*')").unwrap();
        let elements: Vec<_> = value
            .as_array()
            .unwrap()
            .elements()
            .iter()
            .map(|v| v.as_file().unwrap().as_str())
            .collect();
        assert_eq!(elements, ["bar", "baz"]);

        let value = eval_v1_expr(&mut env, V1::Two, "glob('**/b*')").unwrap();
        let elements: Vec<_> = value
            .as_array()
            .unwrap()
            .elements()
            .iter()
            .map(|v| v.as_file().unwrap().as_str())
            .collect();
        assert_eq!(elements, ["bar", "baz", "nested/bar", "nested/baz"]);
    }

    #[test]
    fn size() {
        let mut env = TestEnv::default();

        // 10 byte file
        env.write_file("foo", "0123456789");
        // 20 byte file
        env.write_file("bar", "01234567890123456789");
        // 30 byte file
        env.write_file("baz", "012345678901234567890123456789");

        env.insert_name(
            "file",
            PrimitiveValue::new_file(env.cwd().join("bar").to_str().expect("should be UTF-8")),
        );
        env.insert_name(
            "dir",
            PrimitiveValue::new_directory(env.cwd().to_str().expect("should be UTF-8")),
        );

        let diagnostic = eval_v1_expr(&mut env, V1::Two, "size('foo', 'invalid')").unwrap_err();
        assert_eq!(
            diagnostic.message(),
            "invalid storage unit `invalid`; supported units are `B`, `KB`, `K`, `MB`, `M`, `GB`, \
             `G`, `TB`, `T`, `KiB`, `Ki`, `MiB`, `Mi`, `GiB`, `Gi`, `TiB`, and `Ti`"
        );

        let diagnostic =
            eval_v1_expr(&mut env, V1::Two, "size('does-not-exist', 'B')").unwrap_err();
        assert!(
            diagnostic
                .message()
                .starts_with("call to function `size` failed: failed to read metadata for file")
        );

        let source = format!("size('{path}', 'B')", path = env.cwd().display());
        let value = eval_v1_expr(&mut env, V1::Two, &source).unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 60.0);

        for (expected, unit) in [
            (10.0, "B"),
            (0.01, "K"),
            (0.01, "KB"),
            (0.00001, "M"),
            (0.00001, "MB"),
            (0.00000001, "G"),
            (0.00000001, "GB"),
            (0.00000000001, "T"),
            (0.00000000001, "TB"),
            (0.009765625, "Ki"),
            (0.009765625, "KiB"),
            (0.0000095367431640625, "Mi"),
            (0.0000095367431640625, "MiB"),
            (0.000000009313225746154785, "Gi"),
            (0.000000009313225746154785, "GiB"),
            (0.000000000009094947017729282, "Ti"),
            (0.000000000009094947017729282, "TiB"),
        ] {
            let value = eval_v1_expr(&mut env, V1::Two, &format!("size('foo', '{unit}')")).unwrap();
            approx::assert_relative_eq!(value.unwrap_float(), expected);
        }

        let value = eval_v1_expr(&mut env, V1::Two, "size(None, 'B')").unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 0.0);

        let value = eval_v1_expr(&mut env, V1::Two, "size(file, 'B')").unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 20.0);

        let value = eval_v1_expr(&mut env, V1::Two, "size(dir, 'B')").unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 60.0);

        let value = eval_v1_expr(&mut env, V1::Two, "size((dir, dir), 'B')").unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 120.0);

        let value = eval_v1_expr(&mut env, V1::Two, "size([file, file, file], 'B')").unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 60.0);

        let value = eval_v1_expr(
            &mut env,
            V1::Two,
            "size({ 'a': file, 'b': file, 'c': file }, 'B')",
        )
        .unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 60.0);

        let value = eval_v1_expr(
            &mut env,
            V1::Two,
            "size(object { a: file, b: file, c: file }, 'B')",
        )
        .unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 60.0);
    }

    #[test]
    fn stdout() {
        let mut env = TestEnv::default();
        let diagnostic = eval_v1_expr(&mut env, V1::Two, "stdout()").unwrap_err();
        assert_eq!(
            diagnostic.message(),
            "call to function `stdout` failed: function may only be called in a task output \
             section"
        );

        let value = eval_v1_expr_with_stdio(
            &mut env,
            V1::Zero,
            "stdout()",
            PrimitiveValue::new_file("stdout.txt"),
            PrimitiveValue::new_file("stderr.txt"),
        )
        .unwrap();
        assert_eq!(value.unwrap_file().as_str(), "stdout.txt");
    }

    #[test]
    fn stderr() {
        let mut env = TestEnv::default();
        let diagnostic = eval_v1_expr(&mut env, V1::Two, "stderr()").unwrap_err();
        assert_eq!(
            diagnostic.message(),
            "call to function `stderr` failed: function may only be called in a task output \
             section"
        );

        let value = eval_v1_expr_with_stdio(
            &mut env,
            V1::Zero,
            "stderr()",
            PrimitiveValue::new_file("stdout.txt"),
            PrimitiveValue::new_file("stderr.txt"),
        )
        .unwrap();
        assert_eq!(value.unwrap_file().as_str(), "stderr.txt");
    }

    #[test]
    fn read_string() {
        let mut env = TestEnv::default();
        env.write_file("foo", "hello\nworld!\n\r\n");
        env.insert_name(
            "file",
            PrimitiveValue::new_file(env.cwd().join("foo").to_str().expect("should be UTF-8")),
        );

        let diagnostic =
            eval_v1_expr(&mut env, V1::Two, "read_string('does-not-exist')").unwrap_err();
        assert!(
            diagnostic
                .message()
                .starts_with("call to function `read_string` failed: failed to read file")
        );

        let value = eval_v1_expr(&mut env, V1::Two, "read_string('foo')").unwrap();
        assert_eq!(value.unwrap_string().as_str(), "hello\nworld!");

        let value = eval_v1_expr(&mut env, V1::Two, "read_string(file)").unwrap();
        assert_eq!(value.unwrap_string().as_str(), "hello\nworld!");
    }

    #[test]
    fn read_int() {
        let mut env = TestEnv::default();
        env.write_file("foo", "12345 hello world!");
        env.write_file("bar", "\n\t\t12345   \n");
        env.insert_name("file", PrimitiveValue::new_file("bar"));

        let diagnostic = eval_v1_expr(&mut env, V1::Two, "read_int('does-not-exist')").unwrap_err();
        assert!(
            diagnostic
                .message()
                .starts_with("call to function `read_int` failed: failed to read file")
        );

        let diagnostic = eval_v1_expr(&mut env, V1::Two, "read_int('foo')").unwrap_err();
        assert!(
            diagnostic
                .message()
                .contains("does not contain a single integer value")
        );

        let value = eval_v1_expr(&mut env, V1::Two, "read_int('bar')").unwrap();
        assert_eq!(value.unwrap_integer(), 12345);

        let value = eval_v1_expr(&mut env, V1::Two, "read_int(file)").unwrap();
        assert_eq!(value.unwrap_integer(), 12345);
    }

    #[test]
    fn read_float() {
        let mut env = TestEnv::default();
        env.write_file("foo", "12345.6789 hello world!");
        env.write_file("bar", "\n\t\t12345.6789   \n");
        env.insert_name("file", PrimitiveValue::new_file("bar"));

        let diagnostic =
            eval_v1_expr(&mut env, V1::Two, "read_float('does-not-exist')").unwrap_err();
        assert!(
            diagnostic
                .message()
                .starts_with("call to function `read_float` failed: failed to read file")
        );

        let diagnostic = eval_v1_expr(&mut env, V1::Two, "read_float('foo')").unwrap_err();
        assert!(
            diagnostic
                .message()
                .contains("does not contain a single float value")
        );

        let value = eval_v1_expr(&mut env, V1::Two, "read_float('bar')").unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 12345.6789);

        let value = eval_v1_expr(&mut env, V1::Two, "read_float(file)").unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 12345.6789);
    }

    #[test]
    fn read_boolean() {
        let mut env = TestEnv::default();
        env.write_file("foo", "true false hello world!");
        env.write_file("bar", "\n\t\tTrUe   \n");
        env.write_file("baz", "\n\t\tfalse   \n");
        env.insert_name("t", PrimitiveValue::new_file("bar"));
        env.insert_name("f", PrimitiveValue::new_file("baz"));

        let diagnostic =
            eval_v1_expr(&mut env, V1::Two, "read_boolean('does-not-exist')").unwrap_err();
        assert!(
            diagnostic
                .message()
                .starts_with("call to function `read_boolean` failed: failed to read file")
        );

        let diagnostic = eval_v1_expr(&mut env, V1::Two, "read_boolean('foo')").unwrap_err();
        assert!(
            diagnostic
                .message()
                .contains("does not contain a single boolean value")
        );

        let value = eval_v1_expr(&mut env, V1::Two, "read_boolean('bar')").unwrap();
        assert!(value.unwrap_boolean());

        let value = eval_v1_expr(&mut env, V1::Two, "read_boolean(t)").unwrap();
        assert!(value.unwrap_boolean());

        let value = eval_v1_expr(&mut env, V1::Two, "read_boolean('baz')").unwrap();
        assert!(!value.unwrap_boolean());

        let value = eval_v1_expr(&mut env, V1::Two, "read_boolean(f)").unwrap();
        assert!(!value.unwrap_boolean());
    }

    #[test]
    fn read_lines() {
        let mut env = TestEnv::default();
        env.write_file("foo", "\nhello!\nworld!\n\r\nhi!\r\nthere!");
        env.write_file("empty", "");
        env.insert_name("file", PrimitiveValue::new_file("foo"));

        let diagnostic =
            eval_v1_expr(&mut env, V1::Two, "read_lines('does-not-exist')").unwrap_err();
        assert!(
            diagnostic
                .message()
                .starts_with("call to function `read_lines` failed: failed to open file")
        );

        let value = eval_v1_expr(&mut env, V1::Two, "read_lines('foo')").unwrap();
        let elements: Vec<_> = value
            .as_array()
            .unwrap()
            .elements()
            .iter()
            .map(|v| v.as_string().unwrap().as_str())
            .collect();
        assert_eq!(elements, ["", "hello!", "world!", "", "hi!", "there!"]);

        let value = eval_v1_expr(&mut env, V1::Two, "read_lines(file)").unwrap();
        let elements: Vec<_> = value
            .as_array()
            .unwrap()
            .elements()
            .iter()
            .map(|v| v.as_string().unwrap().as_str())
            .collect();
        assert_eq!(elements, ["", "hello!", "world!", "", "hi!", "there!"]);

        let value = eval_v1_expr(&mut env, V1::Two, "read_lines('empty')").unwrap();
        assert!(value.unwrap_array().elements().is_empty());
    }

    #[test]
    fn write_lines() {
        let mut env = TestEnv::default();

        let value = eval_v1_expr(&mut env, V1::Two, "write_lines([])").unwrap();
        assert!(
            value
                .as_file()
                .expect("should be file")
                .as_str()
                .starts_with(env.tmp().to_str().expect("should be UTF-8")),
            "file should be in temp directory"
        );
        assert_eq!(
            fs::read_to_string(value.unwrap_file().as_str()).expect("failed to read file"),
            "",
        );

        let value = eval_v1_expr(
            &mut env,
            V1::Two,
            "write_lines(['hello', 'world', '!\n', '!'])",
        )
        .unwrap();
        assert!(
            value
                .as_file()
                .expect("should be file")
                .as_str()
                .starts_with(env.tmp().to_str().expect("should be UTF-8")),
            "file should be in temp directory"
        );
        assert_eq!(
            fs::read_to_string(value.unwrap_file().as_str()).expect("failed to read file"),
            "hello\nworld\n!\n\n!\n"
        );
    }

    #[test]
    fn read_tsv() {
        let mut env = TestEnv::default();
        env.write_file(
            "foo.tsv",
            "row1_1\trow1_2\trow1_3\nrow2_1\trow2_2\trow2_3\trow2_4\nrow3_1\trow3_2\n",
        );
        env.write_file(
            "bar.tsv",
            "foo\tbar\tbaz\nrow1_1\trow1_2\trow1_3\nrow2_1\trow2_2\trow2_3\nrow3_1\trow3_2\trow3_3",
        );
        env.write_file(
            "bar_invalid.tsv",
            "foo\tbar\tbaz\nnrow1_1\trow1_2\trow1_3\nrow2_1\trow2_3\nrow3_1\trow3_2\trow3_3",
        );
        env.write_file(
            "baz.tsv",
            "row1_1\trow1_2\trow1_3\nrow2_1\trow2_2\trow2_3\nrow3_1\trow3_2\trow3_3",
        );
        env.write_file("empty.tsv", "");
        env.write_file("invalid_name.tsv", "invalid-name\nfoo");

        let diagnostic = eval_v1_expr(&mut env, V1::Two, "read_tsv('unknown.tsv')").unwrap_err();
        assert!(
            diagnostic
                .message()
                .contains("call to function `read_tsv` failed: failed to open file")
        );

        let value = eval_v1_expr(&mut env, V1::Two, "read_tsv('empty.tsv')").unwrap();
        assert!(value.unwrap_array().elements().is_empty());

        let diagnostic = eval_v1_expr(&mut env, V1::Two, "read_tsv('foo.tsv', false)").unwrap_err();
        assert_eq!(
            diagnostic.message(),
            "call to function `read_tsv` failed: argument specifying presence of a file header \
             must be `true`"
        );

        let value = eval_v1_expr(&mut env, V1::Two, "read_tsv('foo.tsv')").unwrap();
        let elements = value
            .as_array()
            .unwrap()
            .elements()
            .iter()
            .map(|v| {
                v.as_array()
                    .unwrap()
                    .elements()
                    .iter()
                    .map(|v| v.as_string().unwrap().as_str())
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(elements, [
            Vec::from_iter(["row1_1", "row1_2", "row1_3"]),
            Vec::from_iter(["row2_1", "row2_2", "row2_3", "row2_4"]),
            Vec::from_iter(["row3_1", "row3_2"])
        ]);

        let value = eval_v1_expr(&mut env, V1::Two, "read_tsv('bar.tsv', true)").unwrap();
        let elements = value
            .as_array()
            .unwrap()
            .elements()
            .iter()
            .map(|v| {
                v.as_object()
                    .unwrap()
                    .members()
                    .iter()
                    .map(|(k, v)| (k.as_str(), v.as_string().unwrap().as_str()))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(elements, [
            Vec::from_iter([("foo", "row1_1"), ("bar", "row1_2"), ("baz", "row1_3")]),
            Vec::from_iter([("foo", "row2_1"), ("bar", "row2_2"), ("baz", "row2_3")]),
            Vec::from_iter([("foo", "row3_1"), ("bar", "row3_2"), ("baz", "row3_3")]),
        ]);

        let value = eval_v1_expr(
            &mut env,
            V1::Two,
            "read_tsv('bar.tsv', true, ['qux', 'jam', 'cakes'])",
        )
        .unwrap();
        let elements = value
            .as_array()
            .unwrap()
            .elements()
            .iter()
            .map(|v| {
                v.as_object()
                    .unwrap()
                    .members()
                    .iter()
                    .map(|(k, v)| (k.as_str(), v.as_string().unwrap().as_str()))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(elements, [
            Vec::from_iter([("qux", "row1_1"), ("jam", "row1_2"), ("cakes", "row1_3")]),
            Vec::from_iter([("qux", "row2_1"), ("jam", "row2_2"), ("cakes", "row2_3")]),
            Vec::from_iter([("qux", "row3_1"), ("jam", "row3_2"), ("cakes", "row3_3")]),
        ]);

        let diagnostic =
            eval_v1_expr(&mut env, V1::Two, "read_tsv('bar.tsv', true, ['nope'])").unwrap_err();
        assert!(
            diagnostic
                .message()
                .contains("call to function `read_tsv` failed: line 2 in file")
        );
        assert!(
            diagnostic
                .message()
                .contains("does not have the expected number of column")
        );

        let diagnostic =
            eval_v1_expr(&mut env, V1::Two, "read_tsv('bar_invalid.tsv', true)").unwrap_err();
        assert!(
            diagnostic
                .message()
                .contains("call to function `read_tsv` failed: line 3 in file")
        );
        assert!(
            diagnostic
                .message()
                .contains("does not have the expected number of column")
        );

        let value = eval_v1_expr(
            &mut env,
            V1::Two,
            "read_tsv('baz.tsv', false, ['foo', 'bar', 'baz'])",
        )
        .unwrap();
        let elements = value
            .as_array()
            .unwrap()
            .elements()
            .iter()
            .map(|v| {
                v.as_object()
                    .unwrap()
                    .members()
                    .iter()
                    .map(|(k, v)| (k.as_str(), v.as_string().unwrap().as_str()))
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();
        assert_eq!(elements, [
            Vec::from_iter([("foo", "row1_1"), ("bar", "row1_2"), ("baz", "row1_3")]),
            Vec::from_iter([("foo", "row2_1"), ("bar", "row2_2"), ("baz", "row2_3")]),
            Vec::from_iter([("foo", "row3_1"), ("bar", "row3_2"), ("baz", "row3_3")]),
        ]);

        let diagnostic = eval_v1_expr(
            &mut env,
            V1::Two,
            "read_tsv('bar_invalid.tsv', true, ['not-valid'])",
        )
        .unwrap_err();
        assert_eq!(
            diagnostic.message(),
            "call to function `read_tsv` failed: column name `not-valid` is not a valid WDL \
             object field name"
        );

        let diagnostic = eval_v1_expr(
            &mut env,
            V1::Two,
            "read_tsv('bar_invalid.tsv', true, ['not-valid'])",
        )
        .unwrap_err();
        assert_eq!(
            diagnostic.message(),
            "call to function `read_tsv` failed: column name `not-valid` is not a valid WDL \
             object field name"
        );

        let diagnostic =
            eval_v1_expr(&mut env, V1::Two, "read_tsv('invalid_name.tsv', true)").unwrap_err();
        assert_eq!(
            diagnostic.message(),
            "call to function `read_tsv` failed: column name `invalid-name` is not a valid WDL \
             object field name"
        );
    }
}
