//! An option within a placeholder.

use grammar::v1::Rule;
use pest::iterators::Pair;
use wdl_grammar as grammar;
use wdl_macros::check_node;
use wdl_macros::extract_one;

use crate::v1::document::expression::literal::String;

/// A kind of placeholder option.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Option {
    /// A separator placholder option.
    Separator(String),

    /// A boolean placeholder option.
    Boolean(bool, String),

    /// A default placeholder option.
    Default(String),
}

impl Option {
    /// If the [`Option`] is a [`Option::Separator`], a reference to the
    /// inner value ([`String`]) is returned within [`Some`]. Else,
    /// [`None`] is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_ast::v1::document::expression::literal::string::inner::component::placeholder::Option;
    /// use wdl_ast::v1::document::expression::literal::string::inner::Component;
    /// use wdl_ast::v1::document::expression::literal::string::Inner;
    /// use wdl_ast::v1::document::expression::literal::String;
    ///
    /// let value = String::SingleQuoted(Inner::new(vec![Component::LiteralContents(
    ///     std::string::String::from("Hello, world!"),
    /// )]));
    ///
    /// let option = Option::Separator(value.clone());
    /// assert!(option.as_separator().is_some());
    ///
    /// let option = Option::Boolean(false, value.clone());
    /// assert!(option.as_separator().is_none());
    ///
    /// let option = Option::Default(value);
    /// assert!(option.as_separator().is_none());
    /// ```
    pub fn as_separator(&self) -> std::option::Option<&String> {
        match self {
            Option::Separator(value) => Some(value),
            _ => None,
        }
    }

    /// If the [`Option`] is a [`Option::Boolean`], a reference to the inner
    /// value is returned within [`Some`]. Else, [`None`] is returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_ast::v1::document::expression::literal::string::inner::component::placeholder::Option;
    /// use wdl_ast::v1::document::expression::literal::string::inner::Component;
    /// use wdl_ast::v1::document::expression::literal::string::Inner;
    /// use wdl_ast::v1::document::expression::literal::String;
    ///
    /// let value = String::SingleQuoted(Inner::new(vec![Component::LiteralContents(
    ///     std::string::String::from("Hello, world!"),
    /// )]));
    ///
    /// let option = Option::Separator(value.clone());
    /// assert!(option.as_boolean().is_none());
    ///
    /// let option = Option::Boolean(false, value.clone());
    /// assert!(option.as_boolean().is_some());
    ///
    /// let option = Option::Default(value);
    /// assert!(option.as_boolean().is_none());
    /// ```
    pub fn as_boolean(&self) -> std::option::Option<(bool, &String)> {
        match self {
            Option::Boolean(b, value) => Some((*b, value)),
            _ => None,
        }
    }

    /// If the [`Option`] is a [`Option::Default`], a reference to the inner
    /// value ([`String`]) is returned within [`Some`]. Else, [`None`] is
    /// returned.
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_ast::v1::document::expression::literal::string::inner::component::placeholder::Option;
    /// use wdl_ast::v1::document::expression::literal::string::inner::Component;
    /// use wdl_ast::v1::document::expression::literal::string::Inner;
    /// use wdl_ast::v1::document::expression::literal::String;
    ///
    /// let value = String::SingleQuoted(Inner::new(vec![Component::LiteralContents(
    ///     std::string::String::from("Hello, world!"),
    /// )]));
    ///
    /// let option = Option::Separator(value.clone());
    /// assert!(option.as_default().is_none());
    ///
    /// let option = Option::Boolean(false, value.clone());
    /// assert!(option.as_default().is_none());
    ///
    /// let option = Option::Default(value);
    /// assert!(option.as_default().is_some());
    /// ```
    pub fn as_default(&self) -> std::option::Option<&String> {
        match self {
            Option::Default(value) => Some(value),
            _ => None,
        }
    }

    /// If the [`Option`] is a [`Option::Separator`], the inner value is
    /// returned within [`Some`]. Else, [`None`] is returned. In both cases,
    /// `self` is consumed.
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_ast::v1::document::expression::literal::string::inner::component::placeholder::Option;
    /// use wdl_ast::v1::document::expression::literal::string::inner::Component;
    /// use wdl_ast::v1::document::expression::literal::string::Inner;
    /// use wdl_ast::v1::document::expression::literal::String;
    ///
    /// let value = String::SingleQuoted(Inner::new(vec![Component::LiteralContents(
    ///     std::string::String::from("Hello, world!"),
    /// )]));
    ///
    /// let option = Option::Separator(value.clone());
    /// assert!(option.into_separator().is_some());
    ///
    /// let option = Option::Boolean(false, value.clone());
    /// assert!(option.into_separator().is_none());
    ///
    /// let option = Option::Default(value);
    /// assert!(option.into_separator().is_none());
    /// ```
    pub fn into_separator(self) -> std::option::Option<String> {
        match self {
            Option::Separator(value) => Some(value),
            _ => None,
        }
    }

    /// If the [`Option`] is a [`Option::Boolean`], the inner value is returned
    /// within [`Some`]. Else, [`None`] is returned. In both cases, `self` is
    /// consumed.
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_ast::v1::document::expression::literal::string::inner::component::placeholder::Option;
    /// use wdl_ast::v1::document::expression::literal::string::inner::Component;
    /// use wdl_ast::v1::document::expression::literal::string::Inner;
    /// use wdl_ast::v1::document::expression::literal::String;
    ///
    /// let value = String::SingleQuoted(Inner::new(vec![Component::LiteralContents(
    ///     std::string::String::from("Hello, world!"),
    /// )]));
    ///
    /// let option = Option::Separator(value.clone());
    /// assert!(option.into_boolean().is_none());
    ///
    /// let option = Option::Boolean(false, value.clone());
    /// assert!(option.into_boolean().is_some());
    ///
    /// let option = Option::Default(value);
    /// assert!(option.into_boolean().is_none());
    /// ```
    pub fn into_boolean(self) -> std::option::Option<(bool, String)> {
        match self {
            Option::Boolean(b, value) => Some((b, value)),
            _ => None,
        }
    }

    /// If the [`Option`] is a [`Option::Default`], the inner value ([`String`])
    /// is returned within [`Some`]. Else, [`None`] is returned. In both cases,
    /// `self` is consumed.
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_ast::v1::document::expression::literal::string::inner::component::placeholder::Option;
    /// use wdl_ast::v1::document::expression::literal::string::inner::Component;
    /// use wdl_ast::v1::document::expression::literal::string::Inner;
    /// use wdl_ast::v1::document::expression::literal::String;
    ///
    /// let value = String::SingleQuoted(Inner::new(vec![Component::LiteralContents(
    ///     std::string::String::from("Hello, world!"),
    /// )]));
    ///
    /// let option = Option::Separator(value.clone());
    /// assert!(option.into_default().is_none());
    ///
    /// let option = Option::Boolean(false, value.clone());
    /// assert!(option.into_default().is_none());
    ///
    /// let option = Option::Default(value);
    /// assert!(option.into_default().is_some());
    /// ```
    pub fn into_default(self) -> std::option::Option<String> {
        match self {
            Option::Default(value) => Some(value),
            _ => None,
        }
    }
}

impl From<Pair<'_, grammar::v1::Rule>> for Option {
    fn from(node: Pair<'_, grammar::v1::Rule>) -> Self {
        check_node!(node, placeholder_option);

        let mut children = node.into_inner().collect::<Vec<_>>();

        let child = match children.len() {
            // SAFETY: we just checked to ensure that exactly one element
            // exists, so this will always unwrap.
            1 => children.pop().unwrap(),

            // SAFETY: the rule is hardcoded to be a choice between a separator
            // placholder, a boolean placeholder, and a default placeholder. As
            // such, the rule dictates there must be one (and only one) child
            // node.
            v => {
                unreachable!(
                    "the `placholder_option` rule must contain exactly one child element, \
                     contains {v}"
                )
            }
        };

        match child.as_rule() {
            Rule::placeholder_option_sep => {
                let node = extract_one!(child, string, placeholder_option_sep);
                let value = String::from(node);

                Option::Separator(value)
            }
            Rule::placeholder_option_boolean => {
                // TODO: a clone is required here because Pest's `FlatPairs`
                // type does not support creating an iterator without taking
                // ownership (at the time of writing). This can be made
                // better with a PR to Pest.
                let key =
                    match extract_one!(child.clone(), boolean, placeholder_option_sep).as_str() {
                        "true" => true,
                        "false" => false,
                        v => unreachable!("{v} is not a valid boolean value"),
                    };

                let node = extract_one!(child, string, placeholder_option_sep);
                let value = String::from(node);

                Option::Boolean(key, value)
            }
            Rule::placeholder_option_default => {
                let node = extract_one!(child, string, placeholder_option_sep);
                let value = String::from(node);

                Option::Default(value)
            }
            v => unreachable!("unexpected placeholder option child: {v:?}"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_correctly_parses_a_separator_placeholder_option() -> Result<(), Box<dyn std::error::Error>>
    {
        let option = wdl_macros::test::valid_node!(r#"sep=";""#, placeholder_option, Option);

        let components = option
            .into_separator()
            .unwrap()
            .into_double_quoted()
            .unwrap()
            .into_inner();
        assert_eq!(components.len(), 1);

        let first = components
            .into_iter()
            .next()
            .unwrap()
            .into_literal_contents()
            .unwrap();
        assert_eq!(first.as_str(), ";");

        Ok(())
    }

    #[test]
    fn it_correctly_parses_a_boolean_placeholder_option() -> Result<(), Box<dyn std::error::Error>>
    {
        let (boolean, value) =
            wdl_macros::test::valid_node!(r#"true="a""#, placeholder_option, Option)
                .into_boolean()
                .unwrap();
        assert!(boolean);

        let components = value.into_double_quoted().unwrap().into_inner();
        assert_eq!(components.len(), 1);

        let first = components.into_iter().next().unwrap();
        let value = first.into_literal_contents().unwrap();

        assert_eq!(value.as_str(), "a");

        let (boolean, value) =
            wdl_macros::test::valid_node!(r#"false="b""#, placeholder_option, Option)
                .into_boolean()
                .unwrap();
        assert!(!boolean);

        let components = value.into_double_quoted().unwrap().into_inner();
        assert_eq!(components.len(), 1);

        let first = components.into_iter().next().unwrap();
        let value = first.into_literal_contents().unwrap();

        assert_eq!(value.as_str(), "b");

        Ok(())
    }

    #[test]
    fn it_correctly_parses_a_default_placeholder_option() -> Result<(), Box<dyn std::error::Error>>
    {
        let option =
            wdl_macros::test::valid_node!(r#"default="foobar""#, placeholder_option, Option);

        let components = option
            .into_default()
            .unwrap()
            .into_double_quoted()
            .unwrap()
            .into_inner();
        assert_eq!(components.len(), 1);

        let first = components
            .into_iter()
            .next()
            .unwrap()
            .into_literal_contents()
            .unwrap();
        assert_eq!(first.as_str(), "foobar");

        Ok(())
    }

    #[test]
    fn it_correctly_parses_a_separator_placeholder_option_with_spaces()
    -> Result<(), Box<dyn std::error::Error>> {
        let option = wdl_macros::test::valid_node!(r#"sep   =  ";""#, placeholder_option, Option);

        let components = option
            .into_separator()
            .unwrap()
            .into_double_quoted()
            .unwrap()
            .into_inner();
        assert_eq!(components.len(), 1);

        let first = components
            .into_iter()
            .next()
            .unwrap()
            .into_literal_contents()
            .unwrap();
        assert_eq!(first.as_str(), ";");

        Ok(())
    }

    #[test]
    fn it_correctly_parses_a_boolean_placeholder_option_with_spaces()
    -> Result<(), Box<dyn std::error::Error>> {
        let (boolean, value) =
            wdl_macros::test::valid_node!(r#"true    =  "a""#, placeholder_option, Option)
                .into_boolean()
                .unwrap();
        assert!(boolean);

        let components = value.into_double_quoted().unwrap().into_inner();
        assert_eq!(components.len(), 1);

        let first = components.into_iter().next().unwrap();
        let value = first.into_literal_contents().unwrap();

        assert_eq!(value.as_str(), "a");

        let (boolean, value) =
            wdl_macros::test::valid_node!(r#"false    =  "b""#, placeholder_option, Option)
                .into_boolean()
                .unwrap();
        assert!(!boolean);

        let components = value.into_double_quoted().unwrap().into_inner();
        assert_eq!(components.len(), 1);

        let first = components.into_iter().next().unwrap();
        let value = first.into_literal_contents().unwrap();

        assert_eq!(value.as_str(), "b");

        Ok(())
    }

    #[test]
    fn it_correctly_parses_a_default_placeholder_option_with_spaces()
    -> Result<(), Box<dyn std::error::Error>> {
        let option =
            wdl_macros::test::valid_node!(r#"default    =  "foobar""#, placeholder_option, Option);

        let components = option
            .into_default()
            .unwrap()
            .into_double_quoted()
            .unwrap()
            .into_inner();
        assert_eq!(components.len(), 1);

        let first = components
            .into_iter()
            .next()
            .unwrap()
            .into_literal_contents()
            .unwrap();
        assert_eq!(first.as_str(), "foobar");

        Ok(())
    }
}
