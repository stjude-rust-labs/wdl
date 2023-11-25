//! `container`.

use pest::iterators::Pair;

use wdl_grammar as grammar;

use grammar::v1::Rule;

use crate::v1::document::expression;
use crate::v1::document::expression::Literal;
use crate::v1::document::Expression;
use crate::v1::macros::check_node;
use crate::v1::macros::unwrap_one;

/// An error related to a [`Value`].
#[derive(Debug)]
pub enum Error {
    /// A common error.
    Common(crate::v1::Error),

    /// Attempted to create an empty array.
    EmptyArray,

    /// An expression error.
    Expression(expression::Error),

    /// An invalid format for a [`Value`].
    InvalidFormat(Expression),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Common(err) => write!(f, "{err}"),
            Error::EmptyArray => write!(f, "empty array"),
            Error::Expression(err) => write!(f, "expression error: {err}"),
            Error::InvalidFormat(_) => write!(f, "invalid value"),
        }
    }
}

impl std::error::Error for Error {}

/// A value for the runtime section's `container` entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Value {
    /// A single URI.
    Single(String),

    /// Multiple URIs.
    Multiple(Vec<String>),
}

impl From<String> for Value {
    fn from(value: String) -> Self {
        Value::Single(value)
    }
}

impl From<Vec<String>> for Value {
    fn from(value: Vec<String>) -> Self {
        Value::Multiple(value)
    }
}

impl TryFrom<Pair<'_, Rule>> for Value {
    type Error = Error;

    fn try_from(node: Pair<'_, Rule>) -> Result<Self, Self::Error> {
        check_node!(node, task_runtime_mapping_value);

        let expression_node = unwrap_one!(node, task_runtime_mapping_value)?;
        let expression = Expression::try_from(expression_node).map_err(Error::Expression)?;

        match expression {
            Expression::Array(array) => {
                if array.inner().is_empty() {
                    return Err(Error::EmptyArray);
                }

                let values =
                    array
                        .clone()
                        .into_inner()
                        .into_iter()
                        .try_fold(Vec::new(), |mut acc, expr| match expr {
                            Expression::Literal(Literal::String(value)) => {
                                acc.push(value);
                                Some(acc)
                            }
                            _ => None,
                        });

                match values {
                    Some(values) => Ok(Value::Multiple(values)),
                    None => Err(Error::InvalidFormat(Expression::Array(array))),
                }
            }
            Expression::Literal(Literal::String(value)) => Ok(Value::Single(value)),
            expr => Err(Error::InvalidFormat(expr)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v1::macros;

    #[test]
    fn it_correctly_parses_a_single_string() {
        let value =
            macros::test::valid_node!(r#""ubuntu:latest""#, task_runtime_mapping_value, Value);
        assert_eq!(value, Value::Single(String::from("ubuntu:latest")));
    }

    #[test]
    fn it_correctly_parses_a_array_of_strings() {
        let value = macros::test::valid_node!(
            r#"["ubuntu:latest", "debian:latest"]"#,
            task_runtime_mapping_value,
            Value
        );
        assert_eq!(
            value,
            Value::Multiple(vec![
                String::from("ubuntu:latest"),
                String::from("debian:latest")
            ])
        );
    }

    #[test]
    fn it_fails_to_parse_from_an_invalid_expression() {
        let parse_node = wdl_grammar::v1::parse_rule(Rule::task_runtime_mapping_value, "None")
            .unwrap()
            .into_inner();

        let err = Value::try_from(parse_node).unwrap_err();
        matches!(err, Error::InvalidFormat(_));
    }

    #[test]
    fn it_fails_to_parse_from_an_array_with_a_non_string() {
        let parse_node = wdl_grammar::v1::parse_rule(
            Rule::task_runtime_mapping_value,
            r#"["ubuntu:latest", None]"#,
        )
        .unwrap()
        .into_inner();

        let err = Value::try_from(parse_node).unwrap_err();
        matches!(err, Error::InvalidFormat(_));
    }

    #[test]
    fn it_fails_to_parse_from_an_empty_array() {
        let parse_node = wdl_grammar::v1::parse_rule(Rule::task_runtime_mapping_value, "[]")
            .unwrap()
            .into_inner();

        let err = Value::try_from(parse_node).unwrap_err();
        matches!(err, Error::EmptyArray);
    }
}
