//! Implements the `size` function from the WDL standard library.

use std::borrow::Cow;
use std::path::Path;
use std::path::PathBuf;

use anyhow::Context;
use anyhow::Result;
use anyhow::bail;
use futures::FutureExt;
use futures::future::BoxFuture;
use tokio::fs;
use url::Url;
use wdl_analysis::types::PrimitiveType;
use wdl_ast::Diagnostic;

use super::CallContext;
use super::Callback;
use super::Function;
use super::Signature;
use crate::CompoundValue;
use crate::PrimitiveValue;
use crate::StorageUnit;
use crate::Value;
use crate::diagnostics::function_call_failed;

/// The name of the function defined in this file for use in diagnostics.
const FUNCTION_NAME: &str = "size";

/// Converts a string to a file path.
///
/// This conversion supports `file://` schemed URLs.
fn to_file_path(cwd: &Path, path: &str) -> Result<PathBuf> {
    // Do not attempt to parse absolute Windows paths (and by extension, we do not
    // support single-character schemed URLs)
    if path.get(1..2) != Some(":") {
        if let Ok(url) = path.parse::<Url>() {
            if url.scheme() != "file" {
                bail!("path `{path}` cannot be sized: only `file` scheme URLs are supported");
            }

            match url.to_file_path() {
                Ok(path) => Ok(path),
                Err(_) => {
                    bail!("path `{path}` cannot be represented as a local file path");
                }
            }
        } else {
            Ok(cwd.join(path))
        }
    } else {
        Ok(cwd.join(path))
    }
}

/// Determines the size of a file, directory, or the sum total sizes of the
/// files/directories contained within a compound value. The files may be
/// optional values; None values have a size of 0.0. By default, the size is
/// returned in bytes unless the optional second argument is specified with a
/// unit.
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#glob
fn size(context: CallContext<'_>) -> BoxFuture<'_, Result<Value, Diagnostic>> {
    async move {
        debug_assert!(!context.arguments.is_empty() && context.arguments.len() < 3);
        debug_assert!(context.return_type_eq(PrimitiveType::Float));

        let unit = if context.arguments.len() == 2 {
            let unit = context
                .coerce_argument(1, PrimitiveType::String)
                .unwrap_string();

            unit.parse().map_err(|_| {
                function_call_failed(
                    FUNCTION_NAME,
                    format!(
                        "invalid storage unit `{unit}`: supported units are `B`, `KB`, `K`, `MB`, \
                         `M`, `GB`, `G`, `TB`, `T`, `KiB`, `Ki`, `MiB`, `Mi`, `GiB`, `Gi`, `TiB`, \
                         and `Ti`",
                    ),
                    context.arguments[1].span,
                )
            })?
        } else {
            StorageUnit::default()
        };

        // If the first argument is a string, we need to check if it's a file or
        // directory and treat it as such.
        let value = match context.arguments[0].value.as_string() {
            Some(s) => {
                let path = to_file_path(context.work_dir(), s).map_err(|e| {
                    function_call_failed(FUNCTION_NAME, format!("{e:?}"), context.call_site)
                })?;
                let metadata = fs::metadata(&path)
                    .await
                    .with_context(|| {
                        format!(
                            "failed to read metadata for file `{path}`",
                            path = path.display()
                        )
                    })
                    .map_err(|e| {
                        function_call_failed(FUNCTION_NAME, format!("{e:?}"), context.call_site)
                    })?;
                if metadata.is_dir() {
                    PrimitiveValue::Directory(s.clone()).into()
                } else {
                    PrimitiveValue::File(s.clone()).into()
                }
            }
            _ => context.arguments[0].value.clone(),
        };

        calculate_disk_size(&value, unit, context.work_dir())
            .await
            .map_err(|e| function_call_failed(FUNCTION_NAME, format!("{e:?}"), context.call_site))
            .map(Into::into)
    }
    .boxed()
}

/// Used to calculate the disk size of a value.
///
/// The value may be a file or a directory or a compound type containing files
/// or directories.
///
/// The size of a directory is based on the sum of the files contained in the
/// directory.
fn calculate_disk_size<'a>(
    value: &'a Value,
    unit: StorageUnit,
    cwd: &'a Path,
) -> BoxFuture<'a, Result<f64>> {
    async move {
        match value {
            Value::None => Ok(0.0),
            Value::Primitive(v) => primitive_disk_size(v, unit, cwd).await,
            Value::Compound(v) => compound_disk_size(v, unit, cwd).await,
            Value::Task(_) => bail!("the size of a task variable cannot be calculated"),
            Value::Hints(_) => bail!("the size of a hints value cannot be calculated"),
            Value::Input(_) => bail!("the size of an input value cannot be calculated"),
            Value::Output(_) => bail!("the size of an output value cannot be calculated"),
            Value::Call(_) => bail!("the size of a call value cannot be calculated"),
        }
    }
    .boxed()
}

/// Calculates the disk size of the given primitive value in the given unit.
async fn primitive_disk_size(value: &PrimitiveValue, unit: StorageUnit, cwd: &Path) -> Result<f64> {
    match value {
        PrimitiveValue::File(path) => {
            let path = to_file_path(cwd, path)?;
            let metadata = fs::metadata(&path).await.with_context(|| {
                format!(
                    "failed to read metadata for file `{path}`",
                    path = path.display()
                )
            })?;

            if !metadata.is_file() {
                bail!("path `{path}` is not a file", path = path.display());
            }

            Ok(unit.units(metadata.len()))
        }
        PrimitiveValue::Directory(path) => {
            let path = to_file_path(cwd, path)?;
            calculate_directory_size(&path, unit).await
        }
        _ => Ok(0.0),
    }
}

/// Calculates the disk size for a compound value in the given unit.
async fn compound_disk_size(value: &CompoundValue, unit: StorageUnit, cwd: &Path) -> Result<f64> {
    match value {
        CompoundValue::Pair(pair) => Ok(calculate_disk_size(pair.left(), unit, cwd).await?
            + calculate_disk_size(pair.right(), unit, cwd).await?),
        CompoundValue::Array(array) => {
            let mut size = 0.0;
            for e in array.as_slice() {
                size += calculate_disk_size(e, unit, cwd).await?;
            }

            Ok(size)
        }
        CompoundValue::Map(map) => {
            let mut size = 0.0;
            for (k, v) in map.iter() {
                size += match k {
                    Some(k) => primitive_disk_size(k, unit, cwd).await?,
                    None => 0.0,
                } + calculate_disk_size(v, unit, cwd).await?;
            }

            Ok(size)
        }
        CompoundValue::Object(object) => {
            let mut size = 0.0;
            for (_, v) in object.iter() {
                size += calculate_disk_size(v, unit, cwd).await?;
            }

            Ok(size)
        }
        CompoundValue::Struct(s) => {
            let mut size = 0.0;
            for (_, v) in s.iter() {
                size += calculate_disk_size(v, unit, cwd).await?;
            }

            Ok(size)
        }
    }
}

/// Calculates the size of the given directory in the given unit.
async fn calculate_directory_size(path: &Path, unit: StorageUnit) -> Result<f64> {
    // Don't follow symlinks as a security measure
    let metadata = fs::symlink_metadata(&path).await.with_context(|| {
        format!(
            "failed to read metadata for directory `{path}`",
            path = path.display()
        )
    })?;

    if !metadata.is_dir() {
        bail!("path `{path}` is not a directory", path = path.display());
    }

    // Create a queue for processing directories
    let mut queue: Vec<Cow<'_, Path>> = Vec::new();
    queue.push(path.into());

    // Process each directory in the queue, adding the sizes of its files
    let mut size = 0.0;
    while let Some(path) = queue.pop() {
        let mut dir = fs::read_dir(&path).await.with_context(|| {
            format!(
                "failed to read entry of directory `{path}`",
                path = path.display()
            )
        })?;

        while let Some(entry) = dir.next_entry().await.with_context(|| {
            format!(
                "failed to read entry of directory `{path}`",
                path = path.display()
            )
        })? {
            // Note: `DirEntry::metadata` doesn't follow symlinks
            let metadata = entry.metadata().await.with_context(|| {
                format!(
                    "failed to read metadata for file `{path}`",
                    path = entry.path().display()
                )
            })?;
            if metadata.is_dir() {
                queue.push(entry.path().into());
            } else {
                size += unit.units(metadata.len());
            }
        }
    }

    Ok(size)
}

/// Gets the function describing `size`.
pub const fn descriptor() -> Function {
    Function::new(
        const {
            &[
                Signature::new("(None, <String>) -> Float", Callback::Async(size)),
                Signature::new("(File?, <String>) -> Float", Callback::Async(size)),
                Signature::new("(String?, <String>) -> Float", Callback::Async(size)),
                Signature::new("(Directory?, <String>) -> Float", Callback::Async(size)),
                Signature::new(
                    "(X, <String>) -> Float where `X`: any compound type that recursively \
                     contains a `File` or `Directory`",
                    Callback::Async(size),
                ),
            ]
        },
    )
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use url::Url;
    use wdl_ast::version::V1;

    use crate::PrimitiveValue;
    use crate::v1::test::TestEnv;
    use crate::v1::test::eval_v1_expr;

    #[tokio::test]
    async fn size() {
        let mut env = TestEnv::default();

        // Create a URL for the working directory and force it to end with `/`
        let mut work_dir_url = Url::from_file_path(env.work_dir()).expect("should convert");
        if let Ok(mut segments) = work_dir_url.path_segments_mut() {
            segments.pop_if_empty();
            segments.push("");
        }

        // 10 byte file
        env.write_file("foo", "0123456789");
        // 20 byte file
        env.write_file("bar", "01234567890123456789");
        // 30 byte file
        env.write_file("baz", "012345678901234567890123456789");

        env.insert_name(
            "file",
            PrimitiveValue::new_file(
                env.work_dir()
                    .join("bar")
                    .to_str()
                    .expect("should be UTF-8"),
            ),
        );
        env.insert_name(
            "dir",
            PrimitiveValue::new_directory(env.work_dir().to_str().expect("should be UTF-8")),
        );

        let diagnostic = eval_v1_expr(&env, V1::Two, "size('foo', 'invalid')")
            .await
            .unwrap_err();
        assert_eq!(
            diagnostic.message(),
            "call to function `size` failed: invalid storage unit `invalid`: supported units are \
             `B`, `KB`, `K`, `MB`, `M`, `GB`, `G`, `TB`, `T`, `KiB`, `Ki`, `MiB`, `Mi`, `GiB`, \
             `Gi`, `TiB`, and `Ti`"
        );

        let diagnostic = eval_v1_expr(&env, V1::Two, "size('https://example.com/foo')")
            .await
            .unwrap_err();
        assert_eq!(
            diagnostic.message(),
            "call to function `size` failed: path `https://example.com/foo` cannot be sized: only \
             `file` scheme URLs are supported"
        );

        let diagnostic = eval_v1_expr(&env, V1::Two, "size('does-not-exist', 'B')")
            .await
            .unwrap_err();
        assert!(
            diagnostic
                .message()
                .starts_with("call to function `size` failed: failed to read metadata for file")
        );

        let source = format!("size('{path}', 'B')", path = env.work_dir().display());
        let value = eval_v1_expr(&env, V1::Two, &source).await.unwrap();
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
            let value = eval_v1_expr(&env, V1::Two, &format!("size('foo', '{unit}')"))
                .await
                .unwrap();
            approx::assert_relative_eq!(value.unwrap_float(), expected);

            let value = eval_v1_expr(
                &env,
                V1::Two,
                &format!(
                    "size('{url}', '{unit}')",
                    url = work_dir_url.join("foo").unwrap().as_str()
                ),
            )
            .await
            .unwrap();
            approx::assert_relative_eq!(value.unwrap_float(), expected);
        }

        let value = eval_v1_expr(&env, V1::Two, "size(None, 'B')")
            .await
            .unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 0.0);

        let value = eval_v1_expr(&env, V1::Two, "size(file, 'B')")
            .await
            .unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 20.0);

        let value = eval_v1_expr(&env, V1::Two, "size(dir, 'B')").await.unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 60.0);

        let value = eval_v1_expr(&env, V1::Two, "size((dir, dir), 'B')")
            .await
            .unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 120.0);

        let value = eval_v1_expr(&env, V1::Two, "size([file, file, file], 'B')")
            .await
            .unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 60.0);

        let value = eval_v1_expr(
            &env,
            V1::Two,
            "size({ 'a': file, 'b': file, 'c': file }, 'B')",
        )
        .await
        .unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 60.0);

        let value = eval_v1_expr(
            &env,
            V1::Two,
            "size(object { a: file, b: file, c: file }, 'B')",
        )
        .await
        .unwrap();
        approx::assert_relative_eq!(value.unwrap_float(), 60.0);
    }
}
