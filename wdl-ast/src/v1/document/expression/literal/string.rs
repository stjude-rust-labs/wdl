//! Literal strings.

pub mod inner;

use grammar::v1::Rule;
pub use inner::Inner;
use pest::iterators::Pair;
use wdl_grammar as grammar;
use wdl_macros::check_node;

/// A literal string.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum String {
    /// A single quoted string.
    SingleQuoted(Inner),

    /// A double quoted string.
    DoubleQuoted(Inner),
}

impl String {
    /// If the [`String`] is a [`String::SingleQuoted`], a reference to the
    /// inner value ([`Inner`]) is returned within [`Some`]. Else, [`None`] is
    /// returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_ast::v1::document::expression::literal::string::inner::Component;
    /// use wdl_ast::v1::document::expression::literal::string::Inner;
    /// use wdl_ast::v1::document::expression::literal::String;
    ///
    /// let value = String::SingleQuoted(Inner::new(vec![Component::LiteralContents(
    ///     std::string::String::from("Hello, world!"),
    /// )]));
    ///
    /// assert!(value.as_single_quoted().is_some());
    ///
    /// let value = String::DoubleQuoted(Inner::new(vec![Component::LiteralContents(
    ///     std::string::String::from("Hello, world!"),
    /// )]));
    ///
    /// assert!(value.as_single_quoted().is_none());
    /// ```
    pub fn as_single_quoted(&self) -> Option<&Inner> {
        match self {
            String::SingleQuoted(inner) => Some(inner),
            _ => None,
        }
    }

    /// If the [`String`] is a [`String::DoubleQuoted`], a reference to the
    /// inner value ([`Inner`]) is returned within [`Some`]. Else, [`None`]
    /// is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_ast::v1::document::expression::literal::string::inner::Component;
    /// use wdl_ast::v1::document::expression::literal::string::Inner;
    /// use wdl_ast::v1::document::expression::literal::String;
    ///
    /// let value = String::DoubleQuoted(Inner::new(vec![Component::LiteralContents(
    ///     std::string::String::from("Hello, world!"),
    /// )]));
    ///
    /// assert!(value.as_single_quoted().is_none());
    ///
    /// let value = String::DoubleQuoted(Inner::new(vec![Component::LiteralContents(
    ///     std::string::String::from("Hello, world!"),
    /// )]));
    ///
    /// assert!(value.as_double_quoted().is_some());
    /// ```
    pub fn as_double_quoted(&self) -> Option<&Inner> {
        match self {
            String::DoubleQuoted(inner) => Some(inner),
            _ => None,
        }
    }

    /// If the [`String`] is a [`String::SingleQuoted`], the inner value
    /// ([`Inner`]) is returned within [`Some`]. Else, [`None`] is returned. In
    /// both cases, `self` is consumed.
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_ast::v1::document::expression::literal::string::inner::Component;
    /// use wdl_ast::v1::document::expression::literal::string::Inner;
    /// use wdl_ast::v1::document::expression::literal::String;
    ///
    /// let value = String::SingleQuoted(Inner::new(vec![Component::LiteralContents(
    ///     std::string::String::from("Hello, world!"),
    /// )]));
    ///
    /// assert!(value.into_single_quoted().is_some());
    ///
    /// let value = String::DoubleQuoted(Inner::new(vec![Component::LiteralContents(
    ///     std::string::String::from("Hello, world!"),
    /// )]));
    ///
    /// assert!(value.into_single_quoted().is_none());
    /// ```
    pub fn into_single_quoted(self) -> Option<Inner> {
        match self {
            String::SingleQuoted(inner) => Some(inner),
            _ => None,
        }
    }

    /// If the [`String`] is a [`String::DoubleQuoted`], the inner value
    /// ([`Inner`]) is returned within [`Some`]. Else, [`None`] is returned. In
    /// both cases, `self` is consumed.
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_ast::v1::document::expression::literal::string::inner::Component;
    /// use wdl_ast::v1::document::expression::literal::string::Inner;
    /// use wdl_ast::v1::document::expression::literal::String;
    ///
    /// let value = String::SingleQuoted(Inner::new(vec![Component::LiteralContents(
    ///     std::string::String::from("Hello, world!"),
    /// )]));
    ///
    /// assert!(value.into_double_quoted().is_none());
    ///
    /// let value = String::DoubleQuoted(Inner::new(vec![Component::LiteralContents(
    ///     std::string::String::from("Hello, world!"),
    /// )]));
    ///
    /// assert!(value.into_double_quoted().is_some());
    /// ```
    pub fn into_double_quoted(self) -> Option<Inner> {
        match self {
            String::DoubleQuoted(inner) => Some(inner),
            _ => None,
        }
    }

    /// Returns the inner value ([`Inner`]) regardless of what type of
    /// [`String`] `self` is.
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_ast::v1::document::expression::literal::string::inner::Component;
    /// use wdl_ast::v1::document::expression::literal::string::Inner;
    /// use wdl_ast::v1::document::expression::literal::String;
    ///
    /// let value = String::SingleQuoted(Inner::new(vec![Component::LiteralContents(
    ///     std::string::String::from("Hello, world!"),
    /// )]));
    ///
    /// assert_eq!(value.into_inner().into_inner().len(), 1);
    ///
    /// let value = String::DoubleQuoted(Inner::new(vec![Component::LiteralContents(
    ///     std::string::String::from("Hello, world!"),
    /// )]));
    ///
    /// assert_eq!(value.into_inner().into_inner().len(), 1);
    /// ```
    pub fn into_inner(self) -> Inner {
        match self {
            String::SingleQuoted(inner) => inner,
            String::DoubleQuoted(inner) => inner,
        }
    }
}

impl From<Pair<'_, grammar::v1::Rule>> for String {
    fn from(value: Pair<'_, grammar::v1::Rule>) -> Self {
        check_node!(value, string);

        let mut nodes = value.into_inner();

        if nodes.len() != 2 {
            dbg!(&nodes);
            unreachable!("strings must always have two elements")
        }

        // SAFETY: we just ensured that exactly three elements exist. Thus,
        // these three elements should always unwrap.
        let quote = nodes.next().unwrap();
        let inner = nodes.next().unwrap();

        let inner = Inner::from(inner);

        match quote.as_rule() {
            Rule::double_quote => Self::DoubleQuoted(inner),
            Rule::single_quote => Self::SingleQuoted(inner),
            v => unreachable!("unexpected quote node: {:?}", v),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::v1::document::expression::literal::string::inner::Component;
    use crate::v1::document::expression::Literal;
    use crate::v1::document::Expression;

    #[test]
    fn it_correctly_parses_a_double_quoted_string_with_only_literal_contents()
    -> Result<(), Box<dyn std::error::Error>> {
        let string = wdl_macros::test::valid_node!(r#""hello, world!""#, string, String);
        let inner = string.into_double_quoted().unwrap();

        let components = inner.into_inner();
        assert_eq!(components.len(), 1);

        let first = components.into_iter().next().unwrap();
        assert!(matches!(first, Component::LiteralContents(ref v) if v == "hello, world!"));

        Ok(())
    }

    #[test]
    fn it_correctly_parses_a_single_quoted_string_with_only_literal_contents()
    -> Result<(), Box<dyn std::error::Error>> {
        let string = wdl_macros::test::valid_node!("'hello, world!'", string, String);
        let inner = string.into_single_quoted().unwrap();

        let components = inner.into_inner();
        assert_eq!(components.len(), 1);

        let first = components.into_iter().next().unwrap();
        assert!(matches!(first, Component::LiteralContents(ref v) if v == "hello, world!"));

        Ok(())
    }

    #[test]
    fn it_correctly_parses_a_double_quoted_string_with_only_placeholders()
    -> Result<(), Box<dyn std::error::Error>> {
        let string = wdl_macros::test::valid_node!(r#""${sep=';' var}""#, string, String);
        let inner = string.into_double_quoted().unwrap();

        let components = inner.into_inner();
        assert_eq!(components.len(), 1);

        let first = components.into_iter().next().unwrap();

        let placeholder = first.into_placeholder().unwrap();

        // Check placeholder inner value.

        // TODO: this can be done better when `as_<type>` methods are
        // implemented on the [`Expression`] type. Specifically, the exact value
        // of `var` can be checked. This is good enough for now.
        let inner = placeholder.inner().as_expression().unwrap();
        assert!(matches!(inner, Expression::Literal(Literal::Identifier(_))));

        // Check placeholder options.
        let options = placeholder.options().unwrap().inner();
        assert_eq!(options.len(), 1);

        let components = options
            .iter()
            .next()
            .unwrap()
            .as_separator()
            .unwrap()
            .as_single_quoted()
            .unwrap()
            .inner();
        assert_eq!(components.len(), 1);

        let value = components
            .iter()
            .next()
            .unwrap()
            .as_literal_contents()
            .unwrap();

        assert_eq!(value.as_str(), ";");

        Ok(())
    }

    #[test]
    fn it_correctly_parses_a_single_quoted_string_with_only_placeholders()
    -> Result<(), Box<dyn std::error::Error>> {
        let string = wdl_macros::test::valid_node!(r#"'${sep=";" var}'"#, string, String);
        let inner = string.into_single_quoted().unwrap();

        let components = inner.into_inner();
        assert_eq!(components.len(), 1);

        let first = components.into_iter().next().unwrap();

        let placeholder = first.into_placeholder().unwrap();

        // Check placeholder inner value.

        // TODO: this can be done better when `as_<type>` methods are
        // implemented on the [`Expression`] type. Specifically, the exact value
        // of `var` can be checked. This is good enough for now.
        let inner = placeholder.inner().as_expression().unwrap();
        assert!(matches!(inner, Expression::Literal(Literal::Identifier(_))));

        // Check placeholder options.
        let options = placeholder.options().unwrap().inner();
        assert_eq!(options.len(), 1);

        let components = options
            .iter()
            .next()
            .unwrap()
            .as_separator()
            .unwrap()
            .as_double_quoted()
            .unwrap()
            .inner();
        assert_eq!(components.len(), 1);

        let value = components
            .iter()
            .next()
            .unwrap()
            .as_literal_contents()
            .unwrap();

        assert_eq!(value.as_str(), ";");

        Ok(())
    }
}
