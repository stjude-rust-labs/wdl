//! Implements the `write_json` function from the WDL standard library.

use std::io::BufWriter;
use std::path::Path;

use tempfile::NamedTempFile;
use wdl_analysis::types::PrimitiveTypeKind;
use wdl_ast::Diagnostic;

use super::CallContext;
use super::Function;
use super::Signature;
use crate::PrimitiveValue;
use crate::Value;
use crate::diagnostics::function_call_failed;

/// Writes a JSON file with the serialized form of a WDL value.
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#write_json
fn write_json(context: CallContext<'_>) -> Result<Value, Diagnostic> {
    debug_assert!(context.arguments.len() == 1);
    debug_assert!(context.return_type_eq(PrimitiveTypeKind::File));

    // Helper for handling errors while writing to the file.
    let write_error = |e: std::io::Error| {
        function_call_failed(
            "write_json",
            format!("failed to write to temporary file: {e}"),
            context.call_site,
        )
    };

    // Create a temporary file that will be persisted after writing the lines
    let mut file = NamedTempFile::new_in(context.tmp()).map_err(|e| {
        function_call_failed(
            "write_json",
            format!("failed to create temporary file: {e}"),
            context.call_site,
        )
    })?;

    // Serialize the value
    let mut writer = BufWriter::new(file.as_file_mut());
    let mut serializer = serde_json::Serializer::pretty(&mut writer);
    context.arguments[0]
        .value
        .serialize(context.types(), &mut serializer)
        .map_err(|e| {
            function_call_failed(
                "write_json",
                format!("failed to serialize value: {e}"),
                context.call_site,
            )
        })?;

    // Consume the writer, flushing the buffer to disk.
    writer
        .into_inner()
        .map_err(|e| write_error(e.into_error()))?;

    let (_, path) = file.keep().map_err(|e| {
        function_call_failed(
            "write_json",
            format!("failed to keep temporary file: {e}"),
            context.call_site,
        )
    })?;

    Ok(
        PrimitiveValue::new_file(path.into_os_string().into_string().map_err(|path| {
            function_call_failed(
                "write_json",
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

/// Gets the function describing `write_json`.
pub const fn descriptor() -> Function {
    Function::new(
        const {
            &[Signature::new(
                "(X) -> File where `X`: any JSON-serializable type",
                write_json,
            )]
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

    use crate::PrimitiveValue;
    use crate::Value;
    use crate::v1::test::TestEnv;
    use crate::v1::test::eval_v1_expr;

    fn assert_file_in_temp(env: &TestEnv, value: &Value) {
        assert!(
            value
                .as_file()
                .expect("should be file")
                .as_str()
                .starts_with(env.tmp().to_str().expect("should be UTF-8")),
            "file should be in temp directory"
        );
    }

    #[test]
    fn write_json() {
        let mut env = TestEnv::default();

        let ty = env.types_mut().add_struct(StructType::new("Foo", [
            ("foo", PrimitiveTypeKind::Integer),
            ("bar", PrimitiveTypeKind::String),
            ("baz", PrimitiveTypeKind::Float),
        ]));
        env.insert_struct("Foo", ty);
        env.insert_name("foo", PrimitiveValue::new_file("foo"));
        env.insert_name("bar", PrimitiveValue::new_file("bar"));

        let value = eval_v1_expr(&mut env, V1::Two, "write_json(None)").unwrap();
        assert_file_in_temp(&env, &value);
        assert_eq!(
            fs::read_to_string(value.unwrap_file().as_str()).expect("failed to read file"),
            "null",
        );

        let value = eval_v1_expr(&mut env, V1::Two, "write_json(true)").unwrap();
        assert_file_in_temp(&env, &value);
        assert_eq!(
            fs::read_to_string(value.unwrap_file().as_str()).expect("failed to read file"),
            "true",
        );

        let value = eval_v1_expr(&mut env, V1::Two, "write_json(false)").unwrap();
        assert_file_in_temp(&env, &value);
        assert_eq!(
            fs::read_to_string(value.unwrap_file().as_str()).expect("failed to read file"),
            "false",
        );

        let value = eval_v1_expr(&mut env, V1::Two, "write_json(12345)").unwrap();
        assert_file_in_temp(&env, &value);
        assert_eq!(
            fs::read_to_string(value.unwrap_file().as_str()).expect("failed to read file"),
            "12345",
        );

        let value = eval_v1_expr(&mut env, V1::Two, "write_json(12345.6789)").unwrap();
        assert_file_in_temp(&env, &value);
        assert_eq!(
            fs::read_to_string(value.unwrap_file().as_str()).expect("failed to read file"),
            "12345.6789",
        );

        let value = eval_v1_expr(&mut env, V1::Two, "write_json('hello world!')").unwrap();
        assert_file_in_temp(&env, &value);
        assert_eq!(
            fs::read_to_string(value.unwrap_file().as_str()).expect("failed to read file"),
            r#""hello world!""#,
        );

        let value = eval_v1_expr(&mut env, V1::Two, "write_json(foo)").unwrap();
        assert_file_in_temp(&env, &value);
        assert_eq!(
            fs::read_to_string(value.unwrap_file().as_str()).expect("failed to read file"),
            r#""foo""#,
        );

        let value = eval_v1_expr(&mut env, V1::Two, "write_json(bar)").unwrap();
        assert_file_in_temp(&env, &value);
        assert_eq!(
            fs::read_to_string(value.unwrap_file().as_str()).expect("failed to read file"),
            r#""bar""#,
        );

        let value = eval_v1_expr(&mut env, V1::Two, "write_json([])").unwrap();
        assert_file_in_temp(&env, &value);
        assert_eq!(
            fs::read_to_string(value.unwrap_file().as_str()).expect("failed to read file"),
            "[]",
        );

        let value = eval_v1_expr(&mut env, V1::Two, "write_json([1, 2, 3])").unwrap();
        assert_file_in_temp(&env, &value);
        assert_eq!(
            fs::read_to_string(value.unwrap_file().as_str()).expect("failed to read file"),
            "[\n  1,\n  2,\n  3\n]",
        );

        let value = eval_v1_expr(&mut env, V1::Two, "write_json({})").unwrap();
        assert_file_in_temp(&env, &value);
        assert_eq!(
            fs::read_to_string(value.unwrap_file().as_str()).expect("failed to read file"),
            "{}",
        );

        let value = eval_v1_expr(
            &mut env,
            V1::Two,
            "write_json({'foo': 'bar', 'baz': 'qux'})",
        )
        .unwrap();
        assert_file_in_temp(&env, &value);
        assert_eq!(
            fs::read_to_string(value.unwrap_file().as_str()).expect("failed to read file"),
            "{\n  \"foo\": \"bar\",\n  \"baz\": \"qux\"\n}",
        );

        let value = eval_v1_expr(
            &mut env,
            V1::Two,
            "write_json(object { foo: 1, bar: 'baz', baz: 1.9 })",
        )
        .unwrap();
        assert_file_in_temp(&env, &value);
        assert_eq!(
            fs::read_to_string(value.unwrap_file().as_str()).expect("failed to read file"),
            "{\n  \"foo\": 1,\n  \"bar\": \"baz\",\n  \"baz\": 1.9\n}",
        );

        let value = eval_v1_expr(
            &mut env,
            V1::Two,
            "write_json(Foo { foo: 1, bar: 'baz', baz: 1.9 })",
        )
        .unwrap();
        assert_file_in_temp(&env, &value);
        assert_eq!(
            fs::read_to_string(value.unwrap_file().as_str()).expect("failed to read file"),
            "{\n  \"foo\": 1,\n  \"bar\": \"baz\",\n  \"baz\": 1.9\n}",
        );
    }
}