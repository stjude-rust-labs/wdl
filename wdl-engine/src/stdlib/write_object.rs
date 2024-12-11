//! Implements the `write_object` function from the WDL standard library.

use std::io::BufWriter;
use std::io::Write;
use std::path::Path;

use tempfile::NamedTempFile;
use wdl_analysis::types::PrimitiveTypeKind;
use wdl_analysis::types::Type;
use wdl_ast::Diagnostic;

use super::CallContext;
use super::Function;
use super::Signature;
use crate::PrimitiveValue;
use crate::Value;
use crate::diagnostics::function_call_failed;
use crate::stdlib::write_tsv::write_tsv_value;

/// Writes a tab-separated value (TSV) file with the contents of a Object or
/// Struct.
///
/// The file contains two tab-delimited lines.
///
/// The first line is the names of the members, and the second line is the
/// corresponding values.
///
/// Each line is terminated by the newline (\n) character. The ordering of the
/// columns is unspecified.
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#write_object
fn write_object(context: CallContext<'_>) -> Result<Value, Diagnostic> {
    debug_assert!(context.arguments.len() == 1);
    debug_assert!(context.return_type_eq(PrimitiveTypeKind::File));

    // Helper for handling errors while writing to the file.
    let write_error = |e: std::io::Error| {
        function_call_failed(
            "write_object",
            format!("failed to write to temporary file: {e}"),
            context.call_site,
        )
    };

    let object = context.coerce_argument(0, Type::Object).unwrap_object();

    // Create a temporary file that will be persisted after writing the map
    let mut file = NamedTempFile::with_prefix_in("tmp", context.temp_dir()).map_err(|e| {
        function_call_failed(
            "write_object",
            format!("failed to create temporary file: {e}"),
            context.call_site,
        )
    })?;

    let mut writer = BufWriter::new(file.as_file_mut());
    if !object.is_empty() {
        // Write the header first
        for (i, key) in object.keys().enumerate() {
            if i > 0 {
                writer.write(b"\t").map_err(write_error)?;
            }

            writer.write(key.as_bytes()).map_err(write_error)?;
        }

        writeln!(&mut writer).map_err(write_error)?;

        for (i, (key, value)) in object.iter().enumerate() {
            if i > 0 {
                writer.write(b"\t").map_err(write_error)?;
            }

            match value {
                Value::None => {}
                Value::Primitive(v) => {
                    if !write_tsv_value(&mut writer, v).map_err(write_error)? {
                        return Err(function_call_failed(
                            "write_object",
                            format!("member `{key}` contains a tab character"),
                            context.call_site,
                        ));
                    }
                }
                _ => {
                    return Err(function_call_failed(
                        "write_object",
                        format!("member `{key}` is not a primitive value"),
                        context.call_site,
                    ));
                }
            }
        }

        writeln!(&mut writer).map_err(write_error)?;
    }

    // Consume the writer, flushing the buffer to disk.
    writer
        .into_inner()
        .map_err(|e| write_error(e.into_error()))?;

    let (_, path) = file.keep().map_err(|e| {
        function_call_failed(
            "write_object",
            format!("failed to keep temporary file: {e}"),
            context.call_site,
        )
    })?;

    Ok(
        PrimitiveValue::new_file(path.into_os_string().into_string().map_err(|path| {
            function_call_failed(
                "write_object",
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

/// Gets the function describing `write_object`.
pub const fn descriptor() -> Function {
    Function::new(
        const {
            &[
                Signature::new("(Object) -> File", write_object),
                Signature::new(
                    "(S) -> File where `S`: any structure containing only primitive types",
                    write_object,
                ),
            ]
        },
    )
}

#[cfg(test)]
mod test {
    use std::fs;

    use pretty_assertions::assert_eq;
    use wdl_analysis::types::PrimitiveTypeKind;
    use wdl_analysis::types::StructType;
    use wdl_ast::version::V1;

    use crate::v1::test::TestEnv;
    use crate::v1::test::eval_v1_expr;

    #[test]
    fn write_object() {
        let mut env = TestEnv::default();

        let ty = env.types_mut().add_struct(StructType::new("Foo", [
            ("foo", PrimitiveTypeKind::Integer),
            ("bar", PrimitiveTypeKind::String),
            ("baz", PrimitiveTypeKind::Boolean),
        ]));

        env.insert_struct("Foo", ty);

        let value = eval_v1_expr(&mut env, V1::Two, "write_object(object {})").unwrap();
        assert!(
            value
                .as_file()
                .expect("should be file")
                .as_str()
                .starts_with(env.temp_dir().to_str().expect("should be UTF-8")),
            "file should be in temp directory"
        );
        assert_eq!(
            fs::read_to_string(value.unwrap_file().as_str()).expect("failed to read file"),
            "",
        );

        let value = eval_v1_expr(
            &mut env,
            V1::Two,
            "write_object(object { foo: 'bar', bar: 1, baz: 3.5 })",
        )
        .unwrap();
        assert!(
            value
                .as_file()
                .expect("should be file")
                .as_str()
                .starts_with(env.temp_dir().to_str().expect("should be UTF-8")),
            "file should be in temp directory"
        );
        assert_eq!(
            fs::read_to_string(value.unwrap_file().as_str()).expect("failed to read file"),
            "foo\tbar\tbaz\nbar\t1\t3.500000\n",
        );

        let value = eval_v1_expr(
            &mut env,
            V1::Two,
            "write_object(Foo { foo: 1, bar: 'foo', baz: true })",
        )
        .unwrap();
        assert!(
            value
                .as_file()
                .expect("should be file")
                .as_str()
                .starts_with(env.temp_dir().to_str().expect("should be UTF-8")),
            "file should be in temp directory"
        );
        assert_eq!(
            fs::read_to_string(value.unwrap_file().as_str()).expect("failed to read file"),
            "foo\tbar\tbaz\n1\tfoo\ttrue\n",
        );

        let value = eval_v1_expr(
            &mut env,
            V1::Two,
            "write_object(object { foo: 1, bar: None, baz: true })",
        )
        .unwrap();
        assert!(
            value
                .as_file()
                .expect("should be file")
                .as_str()
                .starts_with(env.temp_dir().to_str().expect("should be UTF-8")),
            "file should be in temp directory"
        );
        assert_eq!(
            fs::read_to_string(value.unwrap_file().as_str()).expect("failed to read file"),
            "foo\tbar\tbaz\n1\t\ttrue\n",
        );

        let diagnostic =
            eval_v1_expr(&mut env, V1::Two, "write_object(object { foo: [] })").unwrap_err();
        assert_eq!(
            diagnostic.message(),
            "call to function `write_object` failed: member `foo` is not a primitive value"
        );

        let diagnostic =
            eval_v1_expr(&mut env, V1::Two, "write_object(object { foo: '\tbar' })").unwrap_err();
        assert_eq!(
            diagnostic.message(),
            "call to function `write_object` failed: member `foo` contains a tab character"
        );
    }
}
