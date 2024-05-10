//! Options within a placeholder.

use grammar::v1::Rule;
use nonempty::NonEmpty;
use pest::iterators::Pair;
use wdl_grammar as grammar;
use wdl_macros::check_node;

use super::Option;

/// A set of options within a placeholder.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Options(NonEmpty<Option>);

impl Options {
    /// Gets a reference to the inner [`NonEmpty<Option>`].
    ///
    /// # Examples
    ///
    /// ```
    /// use nonempty::NonEmpty;
    /// use wdl_ast::v1::document::expression::literal::string::inner::component::placeholder::Option;
    /// use wdl_ast::v1::document::expression::literal::string::inner::component::placeholder::Options;
    /// use wdl_ast::v1::document::expression::literal::string::inner::Component;
    /// use wdl_ast::v1::document::expression::literal::string::Inner;
    /// use wdl_ast::v1::document::expression::literal::String;
    ///
    /// let value = NonEmpty::new(Option::Separator(String::DoubleQuoted(Inner::new(vec![
    ///     Component::LiteralContents(std::string::String::from(";")),
    /// ]))));
    ///
    /// let options = Options::from(value.clone());
    /// assert_eq!(options.inner(), &value);
    /// ```
    pub fn inner(&self) -> &NonEmpty<Option> {
        &self.0
    }

    /// Consumes `self` and returns the inner [`NonEmpty<Option>`].
    ///
    /// # Examples
    ///
    /// ```
    /// use nonempty::NonEmpty;
    /// use wdl_ast::v1::document::expression::literal::string::inner::component::placeholder::Option;
    /// use wdl_ast::v1::document::expression::literal::string::inner::component::placeholder::Options;
    /// use wdl_ast::v1::document::expression::literal::string::inner::Component;
    /// use wdl_ast::v1::document::expression::literal::string::Inner;
    /// use wdl_ast::v1::document::expression::literal::String;
    ///
    /// let value = NonEmpty::new(Option::Separator(String::DoubleQuoted(Inner::new(vec![
    ///     Component::LiteralContents(std::string::String::from(";")),
    /// ]))));
    ///
    /// let options = Options::from(value.clone());
    /// assert_eq!(options.into_inner(), value);
    /// ```
    pub fn into_inner(self) -> NonEmpty<Option> {
        self.0
    }
}

impl From<NonEmpty<Option>> for Options {
    fn from(value: NonEmpty<Option>) -> Self {
        Self(value)
    }
}

impl From<Pair<'_, grammar::v1::Rule>> for Options {
    fn from(value: Pair<'_, grammar::v1::Rule>) -> Self {
        check_node!(value, placeholder_options);

        let mut options = value
            .into_inner()
            .filter(|node| !matches!(node.as_rule(), Rule::WHITESPACE | Rule::COMMENT))
            .map(Option::from);

        // SAFETY: the grammar for `placeholder_options` requires that one
        // option will always be present, so this will always unwrap.
        let first = options.next().unwrap();

        let mut result = NonEmpty::new(first);
        result.extend(options);

        Self(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_correctly_parses_options() -> Result<(), Box<dyn std::error::Error>> {
        let mut placeholders = wdl_macros::test::valid_node!(
            r#"sep=";" true="hello" false="world" default="foobar""#,
            placeholder_options,
            Options
        )
        .into_inner()
        .into_iter();

        // First placholder option.

        let components = placeholders
            .next()
            .unwrap()
            .into_separator()
            .unwrap()
            .into_double_quoted()
            .unwrap()
            .into_inner();
        assert_eq!(components.len(), 1);

        let value = components
            .into_iter()
            .next()
            .unwrap()
            .into_literal_contents()
            .unwrap();
        assert_eq!(value.as_str(), ";");

        // Second placholder option.

        let (boolean, value) = placeholders.next().unwrap().into_boolean().unwrap();
        assert!(boolean);

        let components = value.into_double_quoted().unwrap().into_inner();
        assert_eq!(components.len(), 1);

        let value = components
            .into_iter()
            .next()
            .unwrap()
            .into_literal_contents()
            .unwrap();
        assert_eq!(value.as_str(), "hello");

        // Third placholder option.

        let (boolean, value) = placeholders.next().unwrap().into_boolean().unwrap();
        assert!(!boolean);

        let components = value.into_double_quoted().unwrap().into_inner();
        assert_eq!(components.len(), 1);

        let value = components
            .into_iter()
            .next()
            .unwrap()
            .into_literal_contents()
            .unwrap();
        assert_eq!(value.as_str(), "world");

        // Fourth placholder option.

        let components = placeholders
            .next()
            .unwrap()
            .into_default()
            .unwrap()
            .into_double_quoted()
            .unwrap()
            .into_inner();
        assert_eq!(components.len(), 1);

        let value = components
            .into_iter()
            .next()
            .unwrap()
            .into_literal_contents()
            .unwrap();
        assert_eq!(value.as_str(), "foobar");

        Ok(())
    }
}
