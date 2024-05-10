//! Inner contents within a literal string.

use grammar::v1::Rule;
use pest::iterators::Pair;
use wdl_grammar as grammar;
use wdl_macros::check_node;

pub mod component;

pub use component::Component;

/// The inner value of a string.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Inner(Vec<Component>);

impl Inner {
    /// Creates a new [`Inner`] from a [`Vec<Component>`].
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_ast::v1::document::expression::literal::string::inner::Component;
    /// use wdl_ast::v1::document::expression::literal::string::Inner;
    ///
    /// let inner = Inner::new(vec![Component::LiteralContents(std::string::String::from(
    ///     "Hello, world!",
    /// ))]);
    ///
    /// assert_eq!(inner.into_inner().len(), 1);
    /// ```
    pub fn new(value: Vec<Component>) -> Self {
        Self(value)
    }

    /// Gets a reference to the inner [`Vec<Component>`].
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_ast::v1::document::expression::literal::string::inner::Component;
    /// use wdl_ast::v1::document::expression::literal::string::Inner;
    ///
    /// let inner = Inner::new(vec![Component::LiteralContents(std::string::String::from(
    ///     "Hello, world!",
    /// ))]);
    ///
    /// assert_eq!(inner.inner().len(), 1);
    /// ```
    pub fn inner(&self) -> &Vec<Component> {
        &self.0
    }

    /// Consumes `self` and returns the inner [`Vec<Component>`].
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_ast::v1::document::expression::literal::string::inner::Component;
    /// use wdl_ast::v1::document::expression::literal::string::Inner;
    ///
    /// let inner = Inner::new(vec![Component::LiteralContents(std::string::String::from(
    ///     "Hello, world!",
    /// ))]);
    ///
    /// assert_eq!(inner.into_inner().len(), 1);
    /// ```
    pub fn into_inner(self) -> Vec<Component> {
        self.0
    }
}

impl From<Pair<'_, grammar::v1::Rule>> for Inner {
    fn from(value: Pair<'_, grammar::v1::Rule>) -> Self {
        check_node!(value, string_inner);

        let components = value
            .into_inner()
            .filter(|node| !matches!(node.as_rule(), Rule::WHITESPACE | Rule::COMMENT))
            .map(Component::from)
            .collect::<Vec<_>>();

        Self(components)
    }
}
