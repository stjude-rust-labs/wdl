//! Placeholders within a string literal.

mod inner;
mod option;
mod options;

use grammar::v1::Rule;
pub use inner::Inner;
pub use option::Option;
pub use options::Options;
use pest::iterators::Pair;
use wdl_grammar as grammar;
use wdl_macros::check_node;
use wdl_macros::extract_one;

/// A placeholder within a literal string.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Placeholder {
    /// A set of placeholders for the placeholder, if they exist.
    options: std::option::Option<Options>,

    /// The inner value of the placeholder.
    inner: Inner,
}

impl Placeholder {
    /// Creates a new [`Placeholder`].
    ///
    /// # Examples
    ///
    /// ```
    /// use nonempty::NonEmpty;
    /// use wdl_ast::v1::document::expression::literal::string;
    /// use wdl_ast::v1::document::expression::literal::string::inner::component::placeholder::Inner;
    /// use wdl_ast::v1::document::expression::literal::string::inner::component::placeholder::Option;
    /// use wdl_ast::v1::document::expression::literal::string::inner::component::placeholder::Options;
    /// use wdl_ast::v1::document::expression::literal::string::inner::component::Placeholder;
    /// use wdl_ast::v1::document::expression::literal::string::inner::Component;
    /// use wdl_ast::v1::document::expression::literal::String;
    /// use wdl_ast::v1::document::expression::Literal;
    /// use wdl_ast::v1::document::identifier::singular::Identifier;
    /// use wdl_ast::v1::document::Expression;
    ///
    /// let options = Options::from(NonEmpty::new(Option::Separator(String::DoubleQuoted(
    ///     string::Inner::new(vec![Component::LiteralContents(std::string::String::from(
    ///         ";",
    ///     ))]),
    /// ))));
    /// let inner = Inner::Expression(Expression::Literal(Literal::Identifier(
    ///     Identifier::try_from("var").unwrap(),
    /// )));
    ///
    /// let placeholder = Placeholder::new(Some(options.clone()), inner.clone());
    /// assert_eq!(placeholder.options(), Some(&options));
    /// assert_eq!(placeholder.inner(), &inner);
    /// ```
    pub fn new(options: std::option::Option<Options>, inner: Inner) -> Self {
        Self { options, inner }
    }

    /// Gets the [`Options`] from a [`Placeholder`] by reference (if they
    /// exist).
    ///
    /// # Examples
    ///
    /// ```
    /// use nonempty::NonEmpty;
    /// use wdl_ast::v1::document::expression::literal::string;
    /// use wdl_ast::v1::document::expression::literal::string::inner::component::placeholder::Inner;
    /// use wdl_ast::v1::document::expression::literal::string::inner::component::placeholder::Option;
    /// use wdl_ast::v1::document::expression::literal::string::inner::component::placeholder::Options;
    /// use wdl_ast::v1::document::expression::literal::string::inner::component::Placeholder;
    /// use wdl_ast::v1::document::expression::literal::string::inner::Component;
    /// use wdl_ast::v1::document::expression::literal::String;
    /// use wdl_ast::v1::document::expression::Literal;
    /// use wdl_ast::v1::document::identifier::singular::Identifier;
    /// use wdl_ast::v1::document::Expression;
    ///
    /// let options = Options::from(NonEmpty::new(Option::Separator(String::DoubleQuoted(
    ///     string::Inner::new(vec![Component::LiteralContents(std::string::String::from(
    ///         ";",
    ///     ))]),
    /// ))));
    /// let inner = Inner::Expression(Expression::Literal(Literal::Identifier(
    ///     Identifier::try_from("var").unwrap(),
    /// )));
    ///
    /// let placeholder = Placeholder::new(Some(options.clone()), inner.clone());
    /// assert_eq!(placeholder.options(), Some(&options));
    /// ```
    pub fn options(&self) -> std::option::Option<&Options> {
        self.options.as_ref()
    }

    /// Gets the [`Inner`] from a [`Placeholder`] by reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use nonempty::NonEmpty;
    /// use wdl_ast::v1::document::expression::literal::string;
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
    /// let placeholder = Placeholder::new(None, inner.clone());
    /// assert_eq!(placeholder.inner(), &inner);
    /// ```
    pub fn inner(&self) -> &Inner {
        &self.inner
    }

    /// Consumes `self` and returns the [`Options`] (if they exist).
    ///
    /// # Examples
    ///
    /// ```
    /// use nonempty::NonEmpty;
    /// use wdl_ast::v1::document::expression::literal::string;
    /// use wdl_ast::v1::document::expression::literal::string::inner::component::placeholder::Inner;
    /// use wdl_ast::v1::document::expression::literal::string::inner::component::placeholder::Option;
    /// use wdl_ast::v1::document::expression::literal::string::inner::component::placeholder::Options;
    /// use wdl_ast::v1::document::expression::literal::string::inner::component::Placeholder;
    /// use wdl_ast::v1::document::expression::literal::string::inner::Component;
    /// use wdl_ast::v1::document::expression::literal::String;
    /// use wdl_ast::v1::document::expression::Literal;
    /// use wdl_ast::v1::document::identifier::singular::Identifier;
    /// use wdl_ast::v1::document::Expression;
    ///
    /// let options = Options::from(NonEmpty::new(Option::Separator(String::DoubleQuoted(
    ///     string::Inner::new(vec![Component::LiteralContents(std::string::String::from(
    ///         ";",
    ///     ))]),
    /// ))));
    /// let inner = Inner::Expression(Expression::Literal(Literal::Identifier(
    ///     Identifier::try_from("var").unwrap(),
    /// )));
    ///
    /// let placeholder = Placeholder::new(Some(options.clone()), inner.clone());
    /// assert_eq!(placeholder.into_options(), Some(options));
    /// ```
    pub fn into_options(self) -> std::option::Option<Options> {
        self.options
    }

    /// Consumes `self` and returns the [`Inner`].
    ///
    /// # Examples
    ///
    /// ```
    /// use nonempty::NonEmpty;
    /// use wdl_ast::v1::document::expression::literal::string;
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
    /// let placeholder = Placeholder::new(None, inner.clone());
    /// assert_eq!(placeholder.into_inner(), inner);
    /// ```
    pub fn into_inner(self) -> Inner {
        self.inner
    }

    /// Consumes `self` and returns the constituent parts making up the
    /// [`Placeholder`].
    ///
    /// # Examples
    ///
    /// ```
    /// use nonempty::NonEmpty;
    /// use wdl_ast::v1::document::expression::literal::string;
    /// use wdl_ast::v1::document::expression::literal::string::inner::component::placeholder::Inner;
    /// use wdl_ast::v1::document::expression::literal::string::inner::component::placeholder::Option;
    /// use wdl_ast::v1::document::expression::literal::string::inner::component::placeholder::Options;
    /// use wdl_ast::v1::document::expression::literal::string::inner::component::Placeholder;
    /// use wdl_ast::v1::document::expression::literal::string::inner::Component;
    /// use wdl_ast::v1::document::expression::literal::String;
    /// use wdl_ast::v1::document::expression::Literal;
    /// use wdl_ast::v1::document::identifier::singular::Identifier;
    /// use wdl_ast::v1::document::Expression;
    ///
    /// let options = Options::from(NonEmpty::new(Option::Separator(String::DoubleQuoted(
    ///     string::Inner::new(vec![Component::LiteralContents(std::string::String::from(
    ///         ";",
    ///     ))]),
    /// ))));
    /// let inner = Inner::Expression(Expression::Literal(Literal::Identifier(
    ///     Identifier::try_from("var").unwrap(),
    /// )));
    ///
    /// let placeholder = Placeholder::new(Some(options.clone()), inner.clone());
    ///
    /// let (options_as_parts, inner_as_parts) = placeholder.into_parts();
    /// assert_eq!(options_as_parts, Some(options));
    /// assert_eq!(inner_as_parts, inner);
    /// ```
    pub fn into_parts(self) -> (std::option::Option<Options>, Inner) {
        (self.options, self.inner)
    }
}

impl From<Pair<'_, grammar::v1::Rule>> for Placeholder {
    fn from(node: Pair<'_, grammar::v1::Rule>) -> Self {
        check_node!(node, string_placeholder);

        // TODO: a clone is required here because Pest's `FlatPairs`
        // type does not support creating an iterator without taking
        // ownership (at the time of writing). This can be made
        // better with a PR to Pest.
        let children = node.clone().into_inner();

        let placeholder_options = children
            .filter(|node| matches!(node.as_rule(), Rule::placeholder_options))
            .collect::<Vec<_>>();

        let options = match placeholder_options.len() {
            0 => None,
            1 => {
                // SAFETY: we just checked to ensure exactly one element exists
                // in the [`Vec`], so this will always unwrap.
                let node = placeholder_options.into_iter().next().unwrap();
                Some(Options::from(node))
            }
            v => unreachable!(
                "a placeholder can only have zero or one placeholder options nodes, not {v}"
            ),
        };

        let node = extract_one!(
            node,
            string_expression_placeholder_expression,
            string_placeholder
        );

        let inner = Inner::from(node);

        Self { options, inner }
    }
}
