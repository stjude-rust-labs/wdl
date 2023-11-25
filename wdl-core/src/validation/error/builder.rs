//! A builder for a validation [`Error`](super::Error).

use crate::validation;
use crate::Code;
use crate::Location;

/// An error that occurs when a required field is missing at build time.
#[derive(Debug)]
pub enum MissingError {
    /// A code was not provided to the [`Builder`].
    Code,

    /// A location was not provided to the [`Builder`].
    Location,

    /// A subject was not provided to the [`Builder`].
    Subject,

    /// A body was not provided to the [`Builder`].
    Body,
}

impl std::fmt::Display for MissingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MissingError::Code => write!(f, "code"),
            MissingError::Location => write!(f, "location"),
            MissingError::Subject => write!(f, "subject"),
            MissingError::Body => write!(f, "body"),
        }
    }
}

impl std::error::Error for MissingError {}

/// An error related to a [`Builder`].
#[derive(Debug)]
pub enum Error {
    /// A required field was missing at build time.
    Missing(MissingError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Missing(err) => write!(f, "missing value for field: {err}"),
        }
    }
}

impl std::error::Error for Error {}

/// A [`Result`](std::result::Result) with a [`MissingError`].
pub type Result<T> = std::result::Result<T, Error>;

/// A builder for an [`Error`](validation::Error).
#[derive(Debug, Default)]
pub struct Builder {
    /// The code.
    code: Option<Code>,

    /// The location.
    location: Option<Location>,

    /// The subject.
    subject: Option<String>,

    /// The body.
    body: Option<String>,

    /// The (optional) text to describe how to fix the issue.
    fix: Option<String>,
}

impl Builder {
    /// Sets the code for this [`Builder`].
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_core as core;
    ///
    /// use core::location::Position;
    /// use core::validation::error::Builder;
    /// use core::Code;
    /// use core::Location;
    /// use core::Version;
    ///
    /// let code = Code::try_new(Version::V1, 1)?;
    /// let error = Builder::default()
    ///     .code(code.clone())
    ///     .location(Location::Unplaced)
    ///     .subject("Hello, world!")
    ///     .body("A body.")
    ///     .fix("Apply ample foobar.")
    ///     .try_build()?;
    ///
    /// assert_eq!(error.code(), &code);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn code(mut self, code: Code) -> Self {
        self.code = Some(code);
        self
    }

    /// Sets the location for this [`Builder`].
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_core as core;
    ///
    /// use core::location::Position;
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
    ///     .fix("Apply ample foobar.")
    ///     .try_build()?;
    ///
    /// assert_eq!(error.location(), &Location::Unplaced);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn location(mut self, location: Location) -> Self {
        self.location = Some(location);
        self
    }

    /// Sets the subject for this [`Builder`].
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_core as core;
    ///
    /// use core::location::Position;
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
    ///     .fix("Apply ample foobar.")
    ///     .try_build()?;
    ///
    /// assert_eq!(error.subject(), "Hello, world!");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn subject(mut self, subject: impl Into<String>) -> Self {
        let subject = subject.into();
        self.subject = Some(subject);
        self
    }

    /// Sets the body for this [`Builder`].
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_core as core;
    ///
    /// use core::location::Position;
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
    ///     .fix("Apply ample foobar.")
    ///     .try_build()?;
    ///
    /// assert_eq!(error.body(), "A body.");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn body(mut self, body: impl Into<String>) -> Self {
        let body = body.into();
        self.body = Some(body);
        self
    }

    /// Sets the fix for this [`Builder`].
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_core as core;
    ///
    /// use core::location::Position;
    /// use core::validation::error::Builder;
    /// use core::Code;
    /// use core::Location;
    /// use core::Version;
    ///
    /// let code = Code::try_new(Version::V1, 1)?;
    /// let error = Builder::default()
    ///     .code(code.clone())
    ///     .location(Location::Unplaced)
    ///     .subject("Hello, world!")
    ///     .body("A body.")
    ///     .fix("Apply ample foobar.")
    ///     .try_build()?;
    ///
    /// assert_eq!(error.fix().unwrap(), "Apply ample foobar.");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn fix(mut self, fix: impl Into<String>) -> Self {
        let fix = fix.into();
        self.fix = Some(fix);
        self
    }

    /// Consumes `self` to attempt to build an [`Error`](validation::Error).
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_core as core;
    ///
    /// use core::validation::error::Builder;
    /// use core::Code;
    /// use core::Version;
    /// use core::Location;
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
    /// assert_eq!(error.code().grammar(), &Version::V1);
    /// assert_eq!(error.code().index().get(), 1);
    /// assert_eq!(error.location(), &Location::Unplaced);
    /// assert_eq!(error.subject(), "Hello, world!");
    /// assert_eq!(error.body(), "A body.");
    /// assert_eq!(error.fix().unwrap(), "Apply ample foobar.");
    /// assert_eq!(error.to_string(), "[v1::001] Hello, world!");
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    pub fn try_build(self) -> Result<validation::Error> {
        let code = self
            .code
            .map(Ok)
            .unwrap_or(Err(Error::Missing(MissingError::Code)))?;

        let location = self
            .location
            .map(Ok)
            .unwrap_or(Err(Error::Missing(MissingError::Location)))?;

        let subject = self
            .subject
            .map(Ok)
            .unwrap_or(Err(Error::Missing(MissingError::Subject)))?;

        let body = self
            .body
            .map(Ok)
            .unwrap_or(Err(Error::Missing(MissingError::Body)))?;

        Ok(validation::Error {
            code,
            location,
            subject,
            body,
            fix: self.fix,
        })
    }
}
