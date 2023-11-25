//! `cpu`.

use pest::iterators::Pair;

use wdl_grammar as grammar;

use grammar::v1::Rule;

use crate::v1::document::expression;
use crate::v1::document::expression::ensure_number;
use crate::v1::document::expression::Literal;
use crate::v1::document::Expression;
use crate::v1::macros::check_node;
use crate::v1::macros::unwrap_one;

/// An error related to a [`Value`].
#[derive(Debug)]
pub enum Error {
    /// A common error.
    Common(crate::v1::Error),

    /// An expression error.
    Expression(expression::Error),

    /// An invalid format for a [`Value`].
    InvalidFormat(Expression),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Common(err) => write!(f, "{err}"),
            Error::Expression(err) => write!(f, "expression error: {err}"),
            Error::InvalidFormat(_) => write!(f, "invalid value"),
        }
    }
}

impl std::error::Error for Error {}

/// A value for the runtime section's `container` entry.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Value(Expression);

impl Default for Value {
    fn default() -> Self {
        Self(Expression::Literal(Literal::Integer(1)))
    }
}

impl Value {
    /// Gets the inner [`Expression`] of the [`Value`] by reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_ast as ast;
    ///
    /// use ast::v1::document::expression::Literal;
    /// use ast::v1::document::task::runtime::cpu::Value;
    /// use ast::v1::document::Expression;
    ///
    /// let expr = Expression::Literal(Literal::Integer(4));
    /// let cpu = Value::try_from(expr.clone())?;
    ///
    /// assert_eq!(cpu.inner(), &expr);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn inner(&self) -> &Expression {
        &self.0
    }

    /// Consumes `self` to return the inner [`Expression`].
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_ast as ast;
    ///
    /// use ast::v1::document::expression::Literal;
    /// use ast::v1::document::task::runtime::cpu::Value;
    /// use ast::v1::document::Expression;
    ///
    /// let expr = Expression::Literal(Literal::Integer(4));
    /// let cpu = Value::try_from(expr.clone())?;
    ///
    /// assert_eq!(cpu.into_inner(), expr);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn into_inner(self) -> Expression {
        self.0
    }
}

impl TryFrom<Expression> for Value {
    type Error = Error;

    fn try_from(expression: Expression) -> Result<Self, Self::Error> {
        if ensure_number(&expression).is_some() {
            Ok(Value(expression))
        } else {
            Err(Error::InvalidFormat(expression))
        }
    }
}

impl TryFrom<Pair<'_, Rule>> for Value {
    type Error = Error;

    fn try_from(node: Pair<'_, Rule>) -> Result<Self, Self::Error> {
        check_node!(node, task_runtime_mapping_value);

        let expression_node = unwrap_one!(node, task_runtime_mapping_value)?;
        let expression = Expression::try_from(expression_node).map_err(Error::Expression)?;

        Self::try_from(expression)
    }
}

#[cfg(test)]
mod tests {
    use ordered_float::OrderedFloat;

    use super::*;
    use crate::v1::document::expression::Literal;
    use crate::v1::document::expression::UnarySigned;
    use crate::v1::macros;

    #[test]
    fn the_default_value_is_correct() {
        assert_eq!(
            Value::default(),
            Value(Expression::Literal(Literal::Integer(1)))
        );
    }

    #[test]
    fn it_correctly_parses_integers() {
        let value = macros::test::valid_node!("1", task_runtime_mapping_value, Value);
        assert_eq!(value.into_inner(), Expression::Literal(Literal::Integer(1)));

        let value = macros::test::valid_node!("+1", task_runtime_mapping_value, Value);
        assert_eq!(
            value.into_inner(),
            Expression::UnarySigned(UnarySigned::Positive(Box::new(Expression::Literal(
                Literal::Integer(1)
            ))))
        );

        let value = macros::test::valid_node!("-1", task_runtime_mapping_value, Value);
        assert_eq!(
            value.into_inner(),
            Expression::UnarySigned(UnarySigned::Negative(Box::new(Expression::Literal(
                Literal::Integer(1)
            ))))
        );

        let value = macros::test::valid_node!("-+--1", task_runtime_mapping_value, Value);
        assert_eq!(
            value.into_inner(),
            Expression::UnarySigned(UnarySigned::Negative(Box::new(Expression::UnarySigned(
                UnarySigned::Positive(Box::new(Expression::UnarySigned(UnarySigned::Negative(
                    Box::new(Expression::UnarySigned(UnarySigned::Negative(Box::new(
                        Expression::Literal(Literal::Integer(1))
                    ))))
                ))))
            ))))
        )
    }

    #[test]
    fn it_correctly_parses_floats() {
        let value = macros::test::valid_node!("1.0", task_runtime_mapping_value, Value);
        assert_eq!(
            value.into_inner(),
            Expression::Literal(Literal::Float(OrderedFloat(1.0)))
        );

        let value = macros::test::valid_node!("+1.0", task_runtime_mapping_value, Value);
        assert_eq!(
            value.into_inner(),
            Expression::UnarySigned(UnarySigned::Positive(Box::new(Expression::Literal(
                Literal::Float(OrderedFloat(1.0))
            ))))
        );

        let value = macros::test::valid_node!("-1.0", task_runtime_mapping_value, Value);
        assert_eq!(
            value.into_inner(),
            Expression::UnarySigned(UnarySigned::Negative(Box::new(Expression::Literal(
                Literal::Float(OrderedFloat(1.0))
            ))))
        );

        let value = macros::test::valid_node!("-+--1.5", task_runtime_mapping_value, Value);
        assert_eq!(
            value.into_inner(),
            Expression::UnarySigned(UnarySigned::Negative(Box::new(Expression::UnarySigned(
                UnarySigned::Positive(Box::new(Expression::UnarySigned(UnarySigned::Negative(
                    Box::new(Expression::UnarySigned(UnarySigned::Negative(Box::new(
                        Expression::Literal(Literal::Float(OrderedFloat(1.5)))
                    ))))
                ))))
            ))))
        )
    }

    #[test]
    fn it_fails_to_parse_from_an_invalid_expression() {
        let parse_node = wdl_grammar::v1::parse_rule(Rule::expression, "None")
            .unwrap()
            .into_inner();

        let err = Value::try_from(parse_node).unwrap_err();
        matches!(err, Error::InvalidFormat(_));
    }
}
