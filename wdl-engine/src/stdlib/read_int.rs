//! Implements the `read_int` function from the WDL standard library.

use std::fs;

use anyhow::Context;
use wdl_analysis::types::PrimitiveTypeKind;
use wdl_ast::Diagnostic;

use super::CallContext;
use super::Function;
use super::Signature;
use crate::Value;
use crate::diagnostics::function_call_failed;

/// Reads a file that contains a single line containing only an integer and
/// (optional) whitespace.
///
/// If the line contains a valid integer, that value is returned as an Int. If
/// the file is empty or does not contain a single integer, an error is raised.
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#read_int
fn read_int(context: CallContext<'_>) -> Result<Value, Diagnostic> {
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

/// Gets the function describing `read_int`.
pub const fn descriptor() -> Function {
    Function::new(const { &[Signature::new("(File) -> Int", read_int)] })
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use wdl_ast::version::V1;

    use crate::PrimitiveValue;
    use crate::v1::test::TestEnv;
    use crate::v1::test::eval_v1_expr;

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
}
