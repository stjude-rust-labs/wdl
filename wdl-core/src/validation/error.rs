//! Validation errors.

mod builder;

pub use builder::Builder;

use crate::display;
use crate::Code;
use crate::Location;

/// A validation error.
#[derive(Clone, Debug)]
pub struct Error {
    /// The code.
    code: Code,

    /// The location.
    location: Location,

    /// The subject.
    subject: String,

    /// The body.
    body: String,

    /// The (optional) text to describe how to fix the issue.
    fix: Option<String>,
}

impl Error {
    /// Gets the code for this [`Error`].
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_core as core;
    ///
    /// use core::validation::error::Builder;
    /// use core::Code;
    /// use core::Location;
    /// use core::Version;
    ///
    /// let code = Code::try_new(Version::V1, 1)?;
    /// let error = Builder::default()
    ///     .code(code)
    ///     .location(Location::Unplaced)
    ///     .subject("Hello, world!")
    ///     .body("A body.")
    ///     .fix("How to fix the issue.")
    ///     .try_build()?;
    ///
    /// assert_eq!(error.code().grammar(), &Version::V1);
    /// assert_eq!(error.code().index().get(), 1);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn code(&self) -> &Code {
        &self.code
    }

    /// Gets the location for this [`Error`].
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_core as core;
    ///
    /// use core::validation::error::Builder;
    /// use core::Code;
    /// use core::Location;
    /// use core::Version;
    ///
    /// let code = Code::try_new(Version::V1, 1)?;
    /// let error = Builder::default()
    ///     .code(code)
    ///     .location(Location::Unplaced)
    ///     .subject("Hello, world!")
    ///     .body("A body.")
    ///     .fix("How to fix the issue.")
    ///     .try_build()?;
    ///
    /// assert_eq!(error.location(), &Location::Unplaced);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn location(&self) -> &Location {
        &self.location
    }

    /// Gets the subject for this [`Error`].
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_core as core;
    ///
    /// use core::validation::error::Builder;
    /// use core::Code;
    /// use core::Location;
    /// use core::Version;
    ///
    /// let code = Code::try_new(Version::V1, 1)?;
    /// let error = Builder::default()
    ///     .code(code)
    ///     .location(Location::Unplaced)
    ///     .subject("Hello, world!")
    ///     .body("A body.")
    ///     .fix("How to fix the issue.")
    ///     .try_build()?;
    ///
    /// assert_eq!(error.subject(), "Hello, world!");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn subject(&self) -> &str {
        self.subject.as_str()
    }

    /// Gets the body for this [`Error`].
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_core as core;
    ///
    /// use core::validation::error::Builder;
    /// use core::Code;
    /// use core::Location;
    /// use core::Version;
    ///
    /// let code = Code::try_new(Version::V1, 1)?;
    /// let error = Builder::default()
    ///     .code(code)
    ///     .location(Location::Unplaced)
    ///     .subject("Hello, world!")
    ///     .body("A body.")
    ///     .fix("How to fix the issue.")
    ///     .try_build()?;
    ///
    /// assert_eq!(error.body(), "A body.");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn body(&self) -> &str {
        self.body.as_str()
    }

    /// Gets the fix for this [`Error`] (if it exists).
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_core as core;
    ///
    /// use core::validation::error::Builder;
    /// use core::Code;
    /// use core::Location;
    /// use core::Version;
    ///
    /// let code = Code::try_new(Version::V1, 1)?;
    /// let error = Builder::default()
    ///     .code(code)
    ///     .location(Location::Unplaced)
    ///     .subject("Hello, world!")
    ///     .body("A body.")
    ///     .fix("How to fix the issue.")
    ///     .try_build()?;
    ///
    /// assert_eq!(error.fix().unwrap(), "How to fix the issue.");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn fix(&self) -> Option<&str> {
        self.fix.as_deref()
    }

    /// Displays an error according to the `mode` specified.
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_core as core;
    ///
    /// use std::fmt::Write as _;
    /// use std::path::PathBuf;
    ///
    /// use core::validation::error::Builder;
    /// use core::Code;
    /// use core::Location;
    /// use core::Version;
    /// use core::display;
    ///
    /// let code = Code::try_new(Version::V1, 1)?;
    /// let error = Builder::default()
    ///     .code(code)
    ///     .location(Location::Unplaced)
    ///     .subject("Hello, world!")
    ///     .body("A body.")
    ///     .fix("Apply ample foobar.")
    ///     .try_build()?;
    ///
    /// let mut result = String::new();
    /// error.display(&mut result, display::Mode::OneLine)?;
    /// assert_eq!(result, String::from("[v1::001] Hello, world!"));
    ///
    /// result.clear();
    /// error.display(&mut result, display::Mode::Full)?;
    /// assert_eq!(result, String::from("[v1::001] Hello, world!\n\nA body.\n\nTo fix this error, apply ample foobar."));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    pub fn display(&self, f: &mut impl std::fmt::Write, mode: display::Mode) -> std::fmt::Result {
        match mode {
            display::Mode::OneLine => display_one_line(self, f),
            display::Mode::Full => display_full(self, f),
        }
    }
}

/// Displays the error as a single line.
fn display_one_line(error: &Error, f: &mut impl std::fmt::Write) -> std::fmt::Result {
    write!(f, "[{}] {}", error.code, error.subject)?;

    if let Some(location) = error.location.to_string() {
        write!(f, " ({})", location)?;
    }

    Ok(())
}

/// Displays all information about the error.
fn display_full(error: &Error, f: &mut impl std::fmt::Write) -> std::fmt::Result {
    display_one_line(error, f)?;
    write!(f, "\n\n{}", error.body)?;

    if let Some(fix) = error.fix() {
        write!(f, "\n\nTo fix this error, {}", fix.to_ascii_lowercase())?;
    }

    Ok(())
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.display(f, display::Mode::OneLine)
    }
}

impl std::error::Error for Error {}

/// A [`Result`](std::error::Error) with a zero or more validation [`Error`]s.
pub type Result = std::result::Result<(), Vec<Error>>;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn display() -> std::result::Result<(), Box<dyn std::error::Error>> {
        let code = Code::try_new(crate::Version::V1, 1)?;
        let error = Builder::default()
            .code(code)
            .location(Location::Unplaced)
            .subject("Hello, world!")
            .body("A body.")
            .fix("How to fix the issue.")
            .try_build()?;

        assert_eq!(error.to_string(), "[v1::001] Hello, world!");

        Ok(())
    }
}
