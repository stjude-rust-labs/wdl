//! Components of strings.

use grammar::v1::Rule;
use pest::iterators::Pair;
use wdl_grammar as grammar;

pub mod placeholder;

pub use placeholder::Placeholder;

/// A component within a string.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Component {
    /// A placeholder within a string.
    Placeholder(Placeholder),

    /// Literal contents within a string.
    LiteralContents(String),
}

impl Component {
    /// If the [`Component`] is a [`Component::Placeholder`], a reference to the
    /// inner value ([`Placeholder`]) is returned within [`Some`]. Else,
    /// [`None`] is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_ast::v1::document::expression::literal::string::inner::component::placeholder::Inner;
    /// use wdl_ast::v1::document::expression::literal::string::inner::component::Placeholder;
    /// use wdl_ast::v1::document::expression::literal::string::inner::Component;
    /// use wdl_ast::v1::document::expression::Literal;
    /// use wdl_ast::v1::document::identifier::singular::Identifier;
    /// use wdl_ast::v1::document::Expression;
    ///
    /// let expression = Expression::Literal(Literal::Identifier(Identifier::try_from("var").unwrap()));
    ///
    /// let value = Component::Placeholder(Placeholder::new(None, Inner::Expression(expression)));
    /// assert!(value.as_placeholder().is_some());
    ///
    /// let value = Component::LiteralContents(std::string::String::from("Hello, world!"));
    /// assert!(value.as_placeholder().is_none());
    /// ```
    pub fn as_placeholder(&self) -> Option<&Placeholder> {
        match self {
            Component::Placeholder(options) => Some(options),
            _ => None,
        }
    }

    /// If the [`Component`] is a [`Component::LiteralContents`], a reference to
    /// the inner value ([`String`]) is returned within [`Some`]. Else, [`None`]
    /// is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_ast::v1::document::expression::literal::string::inner::component::placeholder::Inner;
    /// use wdl_ast::v1::document::expression::literal::string::inner::component::Placeholder;
    /// use wdl_ast::v1::document::expression::literal::string::inner::Component;
    /// use wdl_ast::v1::document::expression::Literal;
    /// use wdl_ast::v1::document::identifier::singular::Identifier;
    /// use wdl_ast::v1::document::Expression;
    ///
    /// let expression = Expression::Literal(Literal::Identifier(Identifier::try_from("var").unwrap()));
    ///
    /// let value = Component::Placeholder(Placeholder::new(None, Inner::Expression(expression)));
    /// assert!(value.as_literal_contents().is_none());
    ///
    /// let value = Component::LiteralContents(std::string::String::from("Hello, world!"));
    /// assert!(value.as_literal_contents().is_some());
    /// ```
    pub fn as_literal_contents(&self) -> Option<&String> {
        match self {
            Component::LiteralContents(contents) => Some(contents),
            _ => None,
        }
    }

    /// Consumes `self` to return the inner contents ([`Placeholder`]) wrapped
    /// in [`Some`] if the [`Component`] is a [`Component::Placeholder`]. Else,
    /// returns [`None`].
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_ast::v1::document::expression::literal::string::inner::component::placeholder::Inner;
    /// use wdl_ast::v1::document::expression::literal::string::inner::component::Placeholder;
    /// use wdl_ast::v1::document::expression::literal::string::inner::Component;
    /// use wdl_ast::v1::document::expression::Literal;
    /// use wdl_ast::v1::document::identifier::singular::Identifier;
    /// use wdl_ast::v1::document::Expression;
    ///
    /// let expression = Expression::Literal(Literal::Identifier(Identifier::try_from("var").unwrap()));
    ///
    /// let value = Component::Placeholder(Placeholder::new(None, Inner::Expression(expression)));
    /// assert!(value.into_placeholder().is_some());
    ///
    /// let value = Component::LiteralContents(std::string::String::from("Hello, world!"));
    /// assert!(value.into_placeholder().is_none());
    /// ```
    pub fn into_placeholder(self) -> Option<Placeholder> {
        match self {
            Component::Placeholder(options) => Some(options),
            _ => None,
        }
    }

    /// Consumes `self` to return the inner contents ([`String`]) wrapped in
    /// [`Some`] if the [`Component`] is a [`Component::LiteralContents`]. Else,
    /// returns [`None`].
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_ast::v1::document::expression::literal::string::inner::component::placeholder::Inner;
    /// use wdl_ast::v1::document::expression::literal::string::inner::component::Placeholder;
    /// use wdl_ast::v1::document::expression::literal::string::inner::Component;
    /// use wdl_ast::v1::document::expression::Literal;
    /// use wdl_ast::v1::document::identifier::singular::Identifier;
    /// use wdl_ast::v1::document::Expression;
    ///
    /// let expression = Expression::Literal(Literal::Identifier(Identifier::try_from("var").unwrap()));
    ///
    /// let value = Component::Placeholder(Placeholder::new(None, Inner::Expression(expression)));
    /// assert!(value.into_literal_contents().is_none());
    ///
    /// let value = Component::LiteralContents(std::string::String::from("Hello, world!"));
    /// assert!(value.into_literal_contents().is_some());
    /// ```
    pub fn into_literal_contents(self) -> Option<String> {
        match self {
            Component::LiteralContents(contents) => Some(contents),
            _ => None,
        }
    }
}

impl From<Pair<'_, grammar::v1::Rule>> for Component {
    fn from(node: Pair<'_, grammar::v1::Rule>) -> Self {
        match node.as_rule() {
            Rule::string_placeholder => {
                let placeholder = Placeholder::from(node);
                Component::Placeholder(placeholder)
            }
            Rule::string_literal_contents => Component::LiteralContents(node.as_str().to_owned()),
            v => unreachable!("string components cannot be created from {v:?}"),
        }
    }
}
