//! Codes for validation failures and lint warnings.

use std::num::NonZeroUsize;

use crate::Version;

mod kind;

pub use kind::Kind;
use serde::Deserialize;
use serde::Serialize;

/// An error related to a [`Code`].
#[derive(Debug)]
pub enum Error {
    /// Attempted to make a code with an invalid index.
    InvalidIndex(usize),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidIndex(index) => write!(f, "invalid index: {index}"),
        }
    }
}

impl std::error::Error for Error {}

/// A [`Result`](std::result::Result) with an [`Error`].
type Result<T> = std::result::Result<T, Error>;

/// A code.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub struct Code {
    /// The kind of code.
    kind: Kind,

    /// The grammar for this code.
    grammar: Version,

    /// The index for this code.
    index: NonZeroUsize,
}

impl Code {
    /// Attempts to create a new [`Code`].
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_core::concern::code::Kind;
    /// use wdl_core::concern::Code;
    /// use wdl_core::Version;
    ///
    /// let code = Code::try_new(Kind::Warning, Version::V1, 1)?;
    /// assert_eq!(code.kind(), &Kind::Warning);
    /// assert_eq!(code.grammar(), &Version::V1);
    /// assert_eq!(code.index().get(), 1);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn try_new(kind: Kind, grammar: Version, index: usize) -> Result<Self> {
        let index = NonZeroUsize::try_from(index).map_err(|_| Error::InvalidIndex(index))?;

        Ok(Self {
            kind,
            grammar,
            index,
        })
    }

    /// Gets the [`Kind`] of concern for this [`Code`] by reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_core::concern::code::Kind;
    /// use wdl_core::concern::Code;
    /// use wdl_core::Version;
    ///
    /// let code = Code::try_new(Kind::Warning, Version::V1, 1)?;
    /// assert_eq!(code.kind(), &Kind::Warning);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn kind(&self) -> &Kind {
        &self.kind
    }

    /// Gets the grammar [`Version`] for this [`Code`] by reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_core::concern::code::Kind;
    /// use wdl_core::concern::Code;
    /// use wdl_core::Version;
    ///
    /// let code = Code::try_new(Kind::Warning, Version::V1, 1)?;
    /// assert_eq!(code.grammar(), &Version::V1);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn grammar(&self) -> &Version {
        &self.grammar
    }

    /// Gets the index of this [`Code`] by reference.
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_core::concern::code::Kind;
    /// use wdl_core::concern::Code;
    /// use wdl_core::Version;
    ///
    /// let code = Code::try_new(Kind::Warning, Version::V1, 1)?;
    /// assert_eq!(code.index().get(), 1);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn index(&self) -> NonZeroUsize {
        self.index
    }
}

impl std::fmt::Display for Code {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{}::{}{:03}",
            self.grammar.short_name(),
            self.kind.prefix(),
            self.index
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_index() {
        let err = Code::try_new(Kind::Error, Version::V1, 0).unwrap_err();
        assert!(matches!(err, Error::InvalidIndex(0)));
    }

    #[test]
    fn display() {
        let identity = Code::try_new(Kind::Error, Version::V1, 1).unwrap();
        assert_eq!(identity.to_string(), String::from("v1::E001"));

        let identity = Code::try_new(Kind::Warning, Version::V1, 1).unwrap();
        assert_eq!(identity.to_string(), String::from("v1::W001"));
    }
}
