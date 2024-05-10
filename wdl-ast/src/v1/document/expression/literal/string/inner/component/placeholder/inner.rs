//! The inner contents of a placeholder.

use grammar::v1::Rule;
use pest::iterators::Pair;
use wdl_grammar as grammar;
use wdl_macros::check_node;
use wdl_macros::unwrap_one;

use crate::v1::document::expression::literal::string::inner::component::Placeholder;
use crate::v1::document::Expression;

/// The inner contents of a placeholder.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Inner {
    /// An expression within the placeholder.
    Expression(Expression),

    /// Another placeholder within the placeholder.
    Placeholder(Box<Placeholder>),
}

impl Inner {
    /// If the [`Inner`] is a [`Inner::Expression`], a reference to the
    /// inner value ([`Expression`]) is returned within [`Some`]. Else,
    /// [`None`] is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_ast::v1::document::expression::literal::string::inner::component::placeholder::Inner;
    /// use wdl_ast::v1::document::expression::literal::string::inner::component::Placeholder;
    /// use wdl_ast::v1::document::expression::Literal;
    /// use wdl_ast::v1::document::identifier::singular::Identifier;
    /// use wdl_ast::v1::document::Expression;
    ///
    /// let inner = Inner::Expression(Expression::Literal(Literal::Identifier(
    ///     Identifier::try_from("var").unwrap(),
    /// )));
    ///
    /// assert!(inner.as_expression().is_some());
    ///
    /// let inner = Inner::Placeholder(Box::new(Placeholder::new(
    ///     None,
    ///     Inner::Expression(Expression::Literal(Literal::Identifier(
    ///         Identifier::try_from("var").unwrap(),
    ///     ))),
    /// )));
    ///
    /// assert!(inner.as_expression().is_none());
    /// ```
    pub fn as_expression(&self) -> Option<&Expression> {
        match self {
            Inner::Expression(expression) => Some(expression),
            _ => None,
        }
    }

    /// If the [`Inner`] is a [`Inner::Placeholder`], a reference to the inner
    /// value ([`Box<Placeholder>`]) is returned within [`Some`]. Else, [`None`]
    /// is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_ast::v1::document::expression::literal::string::inner::component::placeholder::Inner;
    /// use wdl_ast::v1::document::expression::literal::string::inner::component::Placeholder;
    /// use wdl_ast::v1::document::expression::Literal;
    /// use wdl_ast::v1::document::identifier::singular::Identifier;
    /// use wdl_ast::v1::document::Expression;
    ///
    /// let inner = Inner::Expression(Expression::Literal(Literal::Identifier(
    ///     Identifier::try_from("var").unwrap(),
    /// )));
    ///
    /// assert!(inner.as_placeholder().is_none());
    ///
    /// let inner = Inner::Placeholder(Box::new(Placeholder::new(
    ///     None,
    ///     Inner::Expression(Expression::Literal(Literal::Identifier(
    ///         Identifier::try_from("var").unwrap(),
    ///     ))),
    /// )));
    ///
    /// assert!(inner.as_placeholder().is_some());
    /// ```
    pub fn as_placeholder(&self) -> Option<&Placeholder> {
        match self {
            Inner::Placeholder(placeholder) => Some(placeholder),
            _ => None,
        }
    }

    /// Consumes `self` to return the inner contents ([`Expression`]) wrapped in
    /// [`Some`] if the [`Inner`] is a [`Inner::Expression`]. Else, returns
    /// [`None`].
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_ast::v1::document::expression::literal::string::inner::component::placeholder::Inner;
    /// use wdl_ast::v1::document::expression::literal::string::inner::component::Placeholder;
    /// use wdl_ast::v1::document::expression::Literal;
    /// use wdl_ast::v1::document::identifier::singular::Identifier;
    /// use wdl_ast::v1::document::Expression;
    ///
    /// let inner = Inner::Expression(Expression::Literal(Literal::Identifier(
    ///     Identifier::try_from("var").unwrap(),
    /// )));
    ///
    /// assert!(inner.into_expression().is_some());
    ///
    /// let inner = Inner::Placeholder(Box::new(Placeholder::new(
    ///     None,
    ///     Inner::Expression(Expression::Literal(Literal::Identifier(
    ///         Identifier::try_from("var").unwrap(),
    ///     ))),
    /// )));
    ///
    /// assert!(inner.into_expression().is_none());
    /// ```
    pub fn into_expression(self) -> Option<Expression> {
        match self {
            Inner::Expression(expression) => Some(expression),
            _ => None,
        }
    }

    /// Consumes `self` to return the inner contents ([`Box<Placeholder>`])
    /// wrapped in [`Some`] if the [`Inner`] is a [`Inner::Placeholder`].
    /// Else, returns [`None`].
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_ast::v1::document::expression::literal::string::inner::component::placeholder::Inner;
    /// use wdl_ast::v1::document::expression::literal::string::inner::component::Placeholder;
    /// use wdl_ast::v1::document::expression::Literal;
    /// use wdl_ast::v1::document::identifier::singular::Identifier;
    /// use wdl_ast::v1::document::Expression;
    ///
    /// let inner = Inner::Expression(Expression::Literal(Literal::Identifier(
    ///     Identifier::try_from("var").unwrap(),
    /// )));
    ///
    /// assert!(inner.into_placeholder().is_none());
    ///
    /// let inner = Inner::Placeholder(Box::new(Placeholder::new(
    ///     None,
    ///     Inner::Expression(Expression::Literal(Literal::Identifier(
    ///         Identifier::try_from("var").unwrap(),
    ///     ))),
    /// )));
    ///
    /// assert!(inner.into_placeholder().is_some());
    /// ```
    pub fn into_placeholder(self) -> Option<Box<Placeholder>> {
        match self {
            Inner::Placeholder(placeholder) => Some(placeholder),
            _ => None,
        }
    }
}

impl From<Pair<'_, grammar::v1::Rule>> for Inner {
    fn from(node: Pair<'_, grammar::v1::Rule>) -> Self {
        check_node!(node, string_expression_placeholder_expression);
        let node = unwrap_one!(node, string_expression_placeholder_expression);

        match node.as_rule() {
            Rule::expression => {
                // SAFETY: we just checked to ensure that the rule is a
                // `Rule::expression`, and we know that creating an
                // [`Expression`] from a `Rule::expression` will always
                // succeed.
                let expression = Expression::try_from(node).unwrap();
                Inner::Expression(expression)
            }
            Rule::string_placeholder => {
                // SAFETY: we just checked to ensure that the rule is a
                // `Rule::expression`, and we know that creating an
                // [`Expression`] from a `Rule::expression` will always
                // succeed.
                let placeholder = Placeholder::from(node);
                Inner::Placeholder(Box::new(placeholder))
            }
            v => panic!("the inner contents of a placeholder cannot be parsed from {v:?}"),
        }
    }
}
