//! Locations.
//!
//! ## [`Position`]
//!
//! A [`Position`] is a row and a column within a file. [`Positions`](Position)
//! are the foundation of many of the broader [`Location`] types.
//!
//! ## [`Location`]
//!
//! A [`Location`] refers to coordinate within a file where an element
//! originated (or lack thereof). [`Locations`](Location) can be one of the
//! following:
//!
//! * [`Location::Unplaced`], meaning the entity associated with the location
//!   did not originate from any location within a file. This is generally
//!   useful when you'd like to represent the location of an element generated
//!   by code rather than parsed from a file.
//! * [`Location::Position`], meaning an entity originated at a single position
//!   within a file.
//! * [`Location::Span`], meaning an entity is represented by a range between a
//!   start and end position within a file.
//!
//! Within `wdl-core`, [`Locations`](Location) are generally used in conjunction
//! with the [`Located<E>`] type.
//!
//! ## [`Located<E>`]
//!
//! This module introduces [`Located<E>`]—a wrapper type that pairs entities
//! (`E`) with a [`Location`]. The [`Located`] type provides direct access to
//! the `E` value via dereferencing and exposes the associated [`Location`]
//! through the [`Located::location()`] method. Notably, trait implementations
//! (excluding [`Clone`]) focus solely on the inner `E` value, meaning
//! operations like comparison, hashing, and ordering do not consider the
//! [`Location`]. This ensures that the type is generally treated as the inner
//! `E` while also providing the context of the [`Location`] when desired.

mod located;
mod position;

pub use located::Located;
pub use position::Position;

/// An error related to a [`Location`].
#[derive(Debug)]
pub enum Error {
    /// A position error.
    Position(position::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Position(err) => write!(f, "position error: {err}"),
        }
    }
}

impl std::error::Error for Error {}

/// A 1-based location.
///
/// See the [module documentation](crate::location) for more information.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Location {
    /// No location.
    ///
    /// This is generally the case when an element was programmatically
    /// generated instead of parsed from an existing document.
    Unplaced,

    /// A single position.
    Position {
        /// The position.
        position: Position,
    },

    /// Spanning from a start location to an end location (inclusive).
    Span {
        /// The start position.
        start: Position,

        /// The end position (inclusive).
        end: Position,
    },
}

impl Location {
    /// Converts a [`Location`] to a [`String`] (if it can be converted).
    ///
    /// Notably, this method conflicts with and does not implement
    /// [`std::string::ToString`]. This was an intentional decision, as that
    /// trait assumes that the struct may _always_ be able to be converted into
    /// a [`String`].
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_core as core;
    ///
    /// use std::num::NonZeroUsize;
    ///
    /// use core::location::Position;
    /// use core::Location;
    ///
    /// assert_eq!(Location::Unplaced.to_string(), None);
    /// assert_eq!(
    ///     Location::Position {
    ///         position: Position::new(
    ///             NonZeroUsize::try_from(1).unwrap(),
    ///             NonZeroUsize::try_from(2).unwrap()
    ///         )
    ///     }
    ///     .to_string(),
    ///     Some(String::from("1:2"))
    /// );
    /// assert_eq!(
    ///     Location::Span {
    ///         start: Position::new(
    ///             NonZeroUsize::try_from(1).unwrap(),
    ///             NonZeroUsize::try_from(2).unwrap()
    ///         ),
    ///         end: Position::new(
    ///             NonZeroUsize::try_from(3).unwrap(),
    ///             NonZeroUsize::try_from(4).unwrap()
    ///         )
    ///     }
    ///     .to_string(),
    ///     Some(String::from("1:2-3:4"))
    /// );
    /// ```
    pub fn to_string(&self) -> Option<String> {
        match self {
            Location::Unplaced => None,
            Location::Position { position } => Some(format!("{}", position)),
            Location::Span { start, end } => Some(format!("{}-{}", start, end)),
        }
    }
}

impl TryFrom<pest::Span<'_>> for Location {
    type Error = Error;

    fn try_from(span: pest::Span<'_>) -> Result<Self, Self::Error> {
        let start = Position::try_from(span.start_pos()).map_err(Error::Position)?;
        let end = Position::try_from(span.end_pos()).map_err(Error::Position)?;

        Ok(Location::Span { start, end })
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;

    use super::*;

    #[test]
    fn display_file() {
        assert_eq!(Location::Unplaced.to_string(), None);
    }

    #[test]
    fn display_position() {
        let result = Location::Position {
            position: Position::new(
                NonZeroUsize::try_from(1).unwrap(),
                NonZeroUsize::try_from(1).unwrap(),
            ),
        }
        .to_string();
        assert_eq!(result, Some(String::from("1:1")));
    }

    #[test]
    fn display_span() {
        let result = Location::Span {
            start: Position::new(
                NonZeroUsize::try_from(1).unwrap(),
                NonZeroUsize::try_from(1).unwrap(),
            ),
            end: Position::new(
                NonZeroUsize::try_from(5).unwrap(),
                NonZeroUsize::try_from(5).unwrap(),
            ),
        }
        .to_string();
        assert_eq!(result, Some(String::from("1:1-5:5")));
    }
}
