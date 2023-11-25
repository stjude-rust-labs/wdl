//! Positions.

use std::num::NonZeroUsize;
use std::num::TryFromIntError;

/// An error related to a [`Position`].
#[derive(Debug)]
pub enum Error {
    /// A [`TryFromIntError`] was encountered.
    TryFromInt(TryFromIntError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::TryFromInt(err) => write!(f, "{err}"),
        }
    }
}

impl std::error::Error for Error {}

/// A position.
///
/// [`Positions`](Position) consist of a line number (`line_no`) and column
/// number (`col_no`). [`Positions`](Position) are 1-based.
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Position {
    /// The line number, starting at one.
    line_no: NonZeroUsize,

    /// The column number, starting a one.
    col_no: NonZeroUsize,
}

impl Position {
    /// Creates a new [`Position`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::num::NonZeroUsize;
    ///
    /// use wdl_core as core;
    ///
    /// use core::location::Position;
    ///
    /// let position = Position::new(
    ///     NonZeroUsize::try_from(1).unwrap(),
    ///     NonZeroUsize::try_from(1).unwrap(),
    /// );
    ///
    /// assert_eq!(position.line_no().get(), 1);
    /// assert_eq!(position.col_no().get(), 1);
    /// ```
    pub fn new(line_no: NonZeroUsize, col_no: NonZeroUsize) -> Self {
        Self { line_no, col_no }
    }

    /// Creates the line number from the [`Position`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::num::NonZeroUsize;
    ///
    /// use wdl_core as core;
    ///
    /// use core::location::Position;
    ///
    /// let position = Position::new(
    ///     NonZeroUsize::try_from(1).unwrap(),
    ///     NonZeroUsize::try_from(1).unwrap(),
    /// );
    ///
    /// assert_eq!(position.line_no().get(), 1);
    /// ```
    pub fn line_no(&self) -> NonZeroUsize {
        self.line_no
    }

    /// Gets the column number from the [`Position`].
    ///
    /// # Examples
    ///
    /// ```
    /// use std::num::NonZeroUsize;
    ///
    /// use wdl_core as core;
    ///
    /// use core::location::Position;
    ///
    /// let position = Position::new(
    ///     NonZeroUsize::try_from(1).unwrap(),
    ///     NonZeroUsize::try_from(1).unwrap(),
    /// );
    ///
    /// assert_eq!(position.col_no().get(), 1);
    /// ```
    pub fn col_no(&self) -> NonZeroUsize {
        self.col_no
    }
}

impl std::fmt::Display for Position {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}:{}", self.line_no, self.col_no)
    }
}

impl TryFrom<pest::Position<'_>> for Position {
    type Error = Error;

    fn try_from(position: pest::Position<'_>) -> Result<Self, Self::Error> {
        let (line_no, col_no) = position.line_col();

        let line_no = NonZeroUsize::try_from(line_no).map_err(Error::TryFromInt)?;
        let col_no = NonZeroUsize::try_from(col_no).map_err(Error::TryFromInt)?;

        Ok(Position { line_no, col_no })
    }
}
