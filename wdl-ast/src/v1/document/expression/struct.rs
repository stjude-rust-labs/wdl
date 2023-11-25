//! A struct literal.

use std::collections::BTreeMap;

use pest::iterators::Pair;

use wdl_grammar as grammar;

use grammar::v1::Rule;

use crate::v1::document::expression;
use crate::v1::document::identifier::singular;
use crate::v1::document::identifier::singular::Identifier;
use crate::v1::document::Expression;
use crate::v1::macros::check_node;
use crate::v1::macros::dive_one;
use crate::v1::macros::extract_one;

/// An error related to a [`Struct`].
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

/// A struct literal.
#[derive(Clone, Debug, Hash, Eq, Ord, PartialEq, PartialOrd)]
pub struct Struct {
    /// The name.
    name: Identifier,

    /// The inner map.
    inner: Inner,
}

impl Struct {
    /// Creates a new [`Struct`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeMap;
    ///
    /// use wdl_ast as ast;
    ///
    /// use ast::v1::document::expression::Literal;
    /// use ast::v1::document::expression::Struct;
    /// use ast::v1::document::identifier::singular::Identifier;
    /// use ast::v1::document::Expression;
    ///
    /// let name = Identifier::try_from("foo")?;
    /// let mut map = BTreeMap::new();
    /// map.insert(
    ///     Identifier::try_from("bar")?,
    ///     Expression::Literal(Literal::None),
    /// );
    ///
    /// let r#struct = Struct::new(name.clone(), map.clone());
    ///
    /// assert_eq!(r#struct.name(), &name);
    /// assert_eq!(r#struct.inner(), &map);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn new(name: Identifier, inner: Inner) -> Self {
        Self { name, inner }
    }

    /// Gets the name of the [`Struct`] by reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeMap;
    ///
    /// use wdl_ast as ast;
    ///
    /// use ast::v1::document::expression::Struct;
    /// use ast::v1::document::identifier::singular::Identifier;
    ///
    /// let name = Identifier::try_from("foo")?;
    /// let mut map = BTreeMap::new();
    ///
    /// let r#struct = Struct::new(name.clone(), map);
    ///
    /// assert_eq!(r#struct.name(), &name);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn name(&self) -> &Identifier {
        &self.name
    }

    /// Gets the [inner map](BTreeMap) of the [`Struct`] by reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use std::collections::BTreeMap;
    ///
    /// use wdl_ast as ast;
    ///
    /// use ast::v1::document::expression::Literal;
    /// use ast::v1::document::expression::Struct;
    /// use ast::v1::document::identifier::singular::Identifier;
    /// use ast::v1::document::Expression;
    ///
    /// let name = Identifier::try_from("foo")?;
    /// let mut map = BTreeMap::new();
    /// map.insert(
    ///     Identifier::try_from("bar")?,
    ///     Expression::Literal(Literal::None),
    /// );
    ///
    /// let r#struct = Struct::new(name.clone(), map.clone());
    ///
    /// assert_eq!(r#struct.inner(), &map);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn inner(&self) -> &Inner {
        &self.inner
    }
}

impl TryFrom<Pair<'_, Rule>> for Struct {
    type Error = Error;

    fn try_from(node: Pair<'_, Rule>) -> Result<Self> {
        check_node!(node, struct_literal);

        let name_node = dive_one!(
            node.clone(),
            struct_literal_name,
            struct_literal,
            Error::Common
        )?;
        let identifier_node = extract_one!(
            name_node,
            singular_identifier,
            struct_literal_name,
            Error::Common
        )?;
        let name = Identifier::try_from(identifier_node).map_err(Error::Identifier)?;

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
                Rule::struct_literal_name => {}
                Rule::COMMA => {}
                Rule::COMMENT => {}
                Rule::WHITESPACE => {}
                rule => unreachable!("struct literal should not contain {:?}", rule),
            }
        }

        Ok(Struct { inner, name })
    }
}
