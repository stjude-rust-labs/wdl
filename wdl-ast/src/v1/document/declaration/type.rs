//! A type of declaration.

use grammar::v1::Rule;
use pest::iterators::Pair;

use wdl_grammar as grammar;

use crate::v1::document::identifier::singular;
use crate::v1::document::identifier::singular::Identifier;
use crate::v1::macros::check_node;
use crate::v1::macros::extract_one;

mod kind;

pub use kind::Kind;

/// An error that occurs when a required field is missing at build time.
#[derive(Debug)]
pub enum MissingError {
    /// A [`Kind`] was not provided.
    Kind,
}

impl std::fmt::Display for MissingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MissingError::Kind => write!(f, "kind"),
        }
    }
}

impl std::error::Error for MissingError {}

/// An error related to a [`Type`].
#[derive(Debug)]
pub enum Error {
    /// A common error.
    Common(crate::v1::Error),

    /// An identifier error.
    Identifier(singular::Error),

    /// A required field was missing at build time.
    Missing(MissingError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Common(err) => write!(f, "{err}"),
            Error::Identifier(err) => write!(f, "identifier error: {err}"),
            Error::Missing(err) => write!(f, "missing value for field: {err}"),
        }
    }
}

impl std::error::Error for Error {}

/// A WDL type.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Type {
    /// The kind of type.
    kind: Kind,

    /// Whether the type is marked as optional.
    optional: bool,
}

impl Type {
    /// Creates a new [`Type`].
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_ast as ast;
    ///
    /// use ast::v1::document::declaration::r#type::Kind;
    /// use ast::v1::document::declaration::Type;
    ///
    /// let r#type = Type::new(Kind::Boolean, false);
    /// assert_eq!(r#type.kind(), &Kind::Boolean);
    /// assert_eq!(r#type.optional(), false);
    /// ```
    pub fn new(kind: Kind, optional: bool) -> Self {
        Self { kind, optional }
    }

    /// Gets the kind of [`Type`] by reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_ast as ast;
    ///
    /// use ast::v1::document::declaration::r#type::Kind;
    /// use ast::v1::document::declaration::Type;
    ///
    /// let r#type = Type::new(Kind::Boolean, false);
    /// assert_eq!(r#type.kind(), &Kind::Boolean);
    /// ```
    pub fn kind(&self) -> &Kind {
        &self.kind
    }

    /// Returns whether the type is optional.
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_ast as ast;
    ///
    /// use ast::v1::document::declaration::r#type::Kind;
    /// use ast::v1::document::declaration::Type;
    ///
    /// let r#type = Type::new(Kind::Boolean, false);
    /// assert_eq!(r#type.kind(), &Kind::Boolean);
    /// assert_eq!(r#type.optional(), false);
    /// ```
    pub fn optional(&self) -> bool {
        self.optional
    }
}

impl TryFrom<Pair<'_, grammar::v1::Rule>> for Type {
    type Error = Error;

    fn try_from(node: Pair<'_, grammar::v1::Rule>) -> Result<Self, Self::Error> {
        check_node!(node, wdl_type);

        let mut kind = None;
        let mut optional = false;

        for node in node.into_inner() {
            match node.as_rule() {
                Rule::map_type => kind = Some(Kind::Map),
                Rule::array_type => kind = Some(Kind::Array),
                Rule::pair_type => kind = Some(Kind::Pair),
                Rule::string_type => kind = Some(Kind::String),
                Rule::file_type => kind = Some(Kind::File),
                Rule::bool_type => kind = Some(Kind::Boolean),
                Rule::int_type => kind = Some(Kind::Integer),
                Rule::float_type => kind = Some(Kind::Float),
                Rule::object_type => kind = Some(Kind::Object),
                Rule::struct_type => {
                    kind = {
                        let identifier_node =
                            extract_one!(node, singular_identifier, struct_type, Error::Common)?;
                        let identifier =
                            Identifier::try_from(identifier_node).map_err(Error::Identifier)?;
                        Some(Kind::Struct(identifier))
                    }
                }
                Rule::OPTION => optional = true,
                Rule::WHITESPACE => {}
                Rule::COMMENT => {}
                rule => unreachable!("type should not contain {:?}", rule),
            }
        }

        let kind = kind
            .map(Ok)
            .unwrap_or(Err(Error::Missing(MissingError::Kind)))?;

        Ok(Type { kind, optional })
    }
}
