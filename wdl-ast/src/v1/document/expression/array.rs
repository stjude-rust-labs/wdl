//! An array.

use pest::iterators::Pair;

use wdl_grammar as grammar;

use grammar::v1::Rule;

use crate::v1::document::expression;
use crate::v1::document::Expression;
use crate::v1::macros::check_node;

/// An error related to an [`Array`].
#[derive(Debug)]
pub enum Error {
    /// A common error.
    Common(crate::v1::Error),

    /// An expression error.
    Expression(Box<expression::Error>),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Common(err) => write!(f, "{err}"),
            Error::Expression(err) => write!(f, "expression error: {err}"),
        }
    }
}

impl std::error::Error for Error {}

/// A [`Result`](std::result::Result) with an [`Error`].
type Result<T> = std::result::Result<T, Error>;

/// An array within an [`Expression`].
#[derive(Clone, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub struct Array(Vec<Expression>);

impl Array {
    /// Creates an empty [`Array`].
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_ast as ast;
    ///
    /// use ast::v1::document::expression::Array;
    ///
    /// let array = Array::empty();
    ///
    /// assert_eq!(array.inner().len(), 0);
    /// ```
    pub fn empty() -> Array {
        Array(Vec::new())
    }

    /// Gets the inner [`Vec<Expression>`] by reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_ast as ast;
    ///
    /// use ast::v1::document::expression::Array;
    /// use ast::v1::document::expression::Literal;
    /// use ast::v1::document::Expression;
    ///
    /// let expressions = vec![Expression::Literal(Literal::None)];
    /// let array = Array::from(expressions);
    ///
    /// let mut expressions = array.inner().into_iter();
    /// assert!(matches!(
    ///     expressions.next().unwrap(),
    ///     &Expression::Literal(Literal::None)
    /// ));
    /// ```
    pub fn inner(&self) -> &Vec<Expression> {
        &self.0
    }

    /// Consumes `self` and returns the inner [`Vec<Expression>`].
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_ast as ast;
    ///
    /// use ast::v1::document::expression::Array;
    /// use ast::v1::document::expression::Literal;
    /// use ast::v1::document::Expression;
    ///
    /// let expressions = vec![Expression::Literal(Literal::None)];
    /// let array = Array::from(expressions);
    ///
    /// let mut expressions = array.into_inner().into_iter();
    /// assert!(matches!(
    ///     expressions.next().unwrap(),
    ///     Expression::Literal(Literal::None)
    /// ));
    /// ```
    pub fn into_inner(self) -> Vec<Expression> {
        self.0
    }
}

impl From<Vec<Expression>> for Array {
    fn from(array: Vec<Expression>) -> Self {
        Array(array)
    }
}

impl TryFrom<Pair<'_, grammar::v1::Rule>> for Array {
    type Error = Error;

    fn try_from(node: Pair<'_, grammar::v1::Rule>) -> Result<Self> {
        check_node!(node, array_literal);

        let expressions = node
            .into_inner()
            .filter(|node| matches!(node.as_rule(), Rule::expression))
            .map(Expression::try_from)
            .collect::<std::result::Result<Vec<_>, _>>()
            .map_err(|err| Error::Expression(Box::new(err)))?;

        Ok(Array(expressions))
    }
}

#[cfg(test)]
mod tests {
    use crate::v1::document::expression::Literal;
    use crate::v1::macros::test::invalid_node;
    use crate::v1::macros::test::valid_node;

    use super::*;

    #[test]
    fn it_parses_from_a_supported_node_type() {
        let array = valid_node!(r#"["Hello", false]"#, array_literal, Array);
        assert_eq!(array.inner().len(), 2);

        let mut array = array.inner().iter();
        assert!(matches!(
            array.next().unwrap(),
            Expression::Literal(Literal::String(_))
        ));
    }

    #[test]
    fn it_fails_to_parse_from_an_unsupported_node_type() {
        invalid_node!(
            "version 1.1\n\ntask hello { command <<<>>> }",
            document,
            array_literal,
            Array
        );
    }
}
