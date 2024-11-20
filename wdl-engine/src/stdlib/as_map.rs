//! Implements the `as_map` function from the WDL standard library.

use std::sync::Arc;

use wdl_ast::Diagnostic;

use super::CallContext;
use super::Function;
use super::Signature;
use crate::Map;
use crate::Value;

/// Converts an Array of Pairs into a Map in which the left elements of the
/// Pairs are the keys and the right elements the values.
///
/// All the keys must be unique, or an error is raised.
///
/// The order of the key/value pairs in the output Map is the same as the order
/// of the Pairs in the Array.
///
/// https://github.com/openwdl/wdl/blob/wdl-1.2/SPEC.md#as_map
fn as_map(context: CallContext<'_>) -> Result<Value, Diagnostic> {
    debug_assert_eq!(context.arguments.len(), 1);
    debug_assert!(
        context
            .types()
            .type_definition(
                context
                    .return_type
                    .as_compound()
                    .expect("type should be compound")
                    .definition()
            )
            .as_map()
            .is_some(),
        "type should be an map"
    );

    let array = context.arguments[0]
        .value
        .as_array()
        .expect("argument should be an array");

    let elements = array
        .elements()
        .iter()
        .map(|e| {
            let pair = e.as_pair().expect("element should be a pair");
            let key = match pair.left() {
                Value::Primitive(v) => v.clone(),
                _ => unreachable!("expected a primitive type for the left value"),
            };
            (key, pair.right().clone())
        })
        .collect();

    Ok(Map::new_unchecked(context.return_type, Arc::new(elements)).into())
}

/// Gets the function describing `as_map`.
pub const fn descriptor() -> Function {
    Function::new(
        const {
            &[Signature::new(
                "(Array[Pair[K, V]]) -> Map[K, V] where `K`: any required primitive type",
                as_map,
            )]
        },
    )
}

#[cfg(test)]
mod test {
    use pretty_assertions::assert_eq;
    use wdl_ast::version::V1;

    use crate::v1::test::TestEnv;
    use crate::v1::test::eval_v1_expr;

    #[test]
    fn as_map() {
        let mut env = TestEnv::default();

        let value = eval_v1_expr(&mut env, V1::One, "as_map([])").unwrap();
        assert_eq!(value.unwrap_map().len(), 0);

        let value = eval_v1_expr(
            &mut env,
            V1::One,
            "as_map([('foo', 'bar'), ('bar', 'baz')])",
        )
        .unwrap();
        let elements: Vec<_> = value
            .as_map()
            .unwrap()
            .elements()
            .iter()
            .map(|(k, v)| {
                (
                    k.as_string().unwrap().as_str(),
                    v.as_string().unwrap().as_str(),
                )
            })
            .collect();
        assert_eq!(elements, [("foo", "bar"), ("bar", "baz")]);

        let value =
            eval_v1_expr(&mut env, V1::One, "as_map([('a', 1), ('c', 3), ('b', 2)])").unwrap();
        let elements: Vec<_> = value
            .as_map()
            .unwrap()
            .elements()
            .iter()
            .map(|(k, v)| (k.as_string().unwrap().as_str(), v.as_integer().unwrap()))
            .collect();
        assert_eq!(elements, [("a", 1), ("c", 3), ("b", 2)]);

        let value = eval_v1_expr(
            &mut env,
            V1::One,
            "as_map(as_pairs({'a': 1, 'c': 3, 'b': 2}))",
        )
        .unwrap();
        let elements: Vec<_> = value
            .as_map()
            .unwrap()
            .elements()
            .iter()
            .map(|(k, v)| (k.as_string().unwrap().as_str(), v.as_integer().unwrap()))
            .collect();
        assert_eq!(elements, [("a", 1), ("c", 3), ("b", 2)]);
    }
}
