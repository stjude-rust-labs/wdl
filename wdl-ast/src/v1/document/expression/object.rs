//! An object literal.

use std::collections::BTreeMap;

use pest::iterators::Pair;

use wdl_grammar as grammar;

use grammar::v1::Rule;

use crate::v1::document::expression;
use crate::v1::document::identifier::singular;
use crate::v1::document::identifier::singular::Identifier;
use crate::v1::document::Expression;
use crate::v1::macros::check_node;
use crate::v1::macros::extract_one;

/// An error related to an [`Object`].
#[derive(Debug)]
pub enum Error {
    /// A common error.
    Common(crate::v1::Error),

    /// An expression error.
    Expression(Box<expression::Error>),

    /// An identifier error.
    Identifier(singular::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Common(err) => write!(f, "{err}"),
            Error::Expression(err) => write!(f, "expression error: {err}"),
            Error::Identifier(err) => write!(f, "identifier error: {err}"),
        }
    }
}

impl std::error::Error for Error {}

/// A [`Result`](std::result::Result) with an [`Error`].
type Result<T> = std::result::Result<T, Error>;

/// The inner mapping of keys and values.
type Inner = BTreeMap<Identifier, Expression>;

/// An object literal.
#[derive(Clone, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub struct Object(Inner);

impl Object {
    /// Gets the [inner map](BTreeMap) of the [`Object`] by reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeMap;
    ///
    /// use wdl_ast as ast;
    ///
    /// use ast::v1::document::expression::Literal;
    /// use ast::v1::document::expression::Object;
    /// use ast::v1::document::identifier::singular::Identifier;
    /// use ast::v1::document::Expression;
    ///
    /// let mut map = BTreeMap::new();
    /// map.insert(
    ///     Identifier::try_from("foo")?,
    ///     Expression::Literal(Literal::None),
    /// );
    ///
    /// let object = Object::from(map.clone());
    ///
    /// assert_eq!(object.inner(), &map);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn inner(&self) -> &Inner {
        &self.0
    }

    /// Consumes `self` and returns the [inner map](BTreeMap) of the [`Object`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeMap;
    ///
    /// use wdl_ast as ast;
    ///
    /// use ast::v1::document::expression::Literal;
    /// use ast::v1::document::expression::Object;
    /// use ast::v1::document::identifier::singular::Identifier;
    /// use ast::v1::document::Expression;
    ///
    /// let mut map = BTreeMap::new();
    /// map.insert(
    ///     Identifier::try_from("foo")?,
    ///     Expression::Literal(Literal::None),
    /// );
    ///
    /// let object = Object::from(map.clone());
    ///
    /// assert_eq!(object.into_inner(), map);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn into_inner(self) -> Inner {
        self.0
    }
}

impl From<Inner> for Object {
    fn from(inner: Inner) -> Self {
        Object(inner)
    }
}

impl TryFrom<Pair<'_, Rule>> for Object {
    type Error = Error;

    fn try_from(node: Pair<'_, Rule>) -> Result<Self> {
        check_node!(node, struct_literal);
        let mut inner = BTreeMap::new();

        for node in node.into_inner() {
            match node.as_rule() {
                Rule::identifier_based_kv_pair => {
                    let key_node = extract_one!(
                        node.clone(),
                        identifier_based_kv_key,
                        identifier_based_kv_pair,
                        Error::Common
                    )?;
                    let identifier_node = extract_one!(
                        key_node,
                        singular_identifier,
                        identifier_based_kv_key,
                        Error::Common
                    )?;
                    let identifier =
                        Identifier::try_from(identifier_node).map_err(Error::Identifier)?;

                    let value_node =
                        extract_one!(node, kv_value, identifier_based_kv_pair, Error::Common)?;
                    let expression_node =
                        extract_one!(value_node, expression, kv_value, Error::Common)?;
                    let expression = Expression::try_from(expression_node)
                        .map_err(Box::new)
                        .map_err(Error::Expression)?;

                    inner.insert(identifier, expression);
                }
                Rule::COMMA => {}
                Rule::COMMENT => {}
                Rule::WHITESPACE => {}
                rule => unreachable!("object literal should not contain {:?}", rule),
            }
        }

        Ok(Object(inner))
    }
}
