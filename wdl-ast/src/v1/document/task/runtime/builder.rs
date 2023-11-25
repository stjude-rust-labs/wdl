//! Builder for a [`Runtime`].

use std::collections::BTreeMap;

use crate::v1::document::identifier::singular::Identifier;
use crate::v1::document::task::runtime::container;
use crate::v1::document::task::runtime::cpu;
use crate::v1::document::task::Runtime;
use crate::v1::document::Expression;

/// An error that occurs when a required field is missing at build time.
#[derive(Debug)]
pub enum MissingError {
    /// A version was not provided to the [`Builder`].
    Container,
}

impl std::fmt::Display for MissingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MissingError::Container => write!(f, "version"),
        }
    }
}

impl std::error::Error for MissingError {}

/// An error that occurs when a multiple values were provded for a field that
/// only accepts a single value.
#[derive(Debug)]
pub enum MultipleError {
    /// Attempted to set multiple values for the container field within the
    /// [`Builder`].
    Container,

    /// Attempted to set multiple values for the cpu field within the
    /// [`Builder`].
    Cpu,
}

impl std::fmt::Display for MultipleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MultipleError::Container => write!(f, "container"),
            MultipleError::Cpu => write!(f, "cpu"),
        }
    }
}

impl std::error::Error for MultipleError {}

/// An error related to a [`Builder`].
#[derive(Debug)]
pub enum Error {
    /// A required field was missing at build time.
    Missing(MissingError),

    /// Multiple values were provided for a field that accepts a single value.
    Multiple(MultipleError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Missing(err) => write!(f, "missing value for field: {err}"),
            Error::Multiple(err) => {
                write!(f, "multiple values provided for single value field: {err}")
            }
        }
    }
}

impl std::error::Error for Error {}

/// A [`Result`](std::result::Result) with an [`Error`].
type Result<T> = std::result::Result<T, Error>;

/// A builder for a [`Runtime`].
#[derive(Debug, Default)]
pub struct Builder {
    /// The container entry.
    container: Option<container::Value>,

    /// The container entry.
    cpu: Option<cpu::Value>,

    /// Other included runtime hints.
    hints: Option<BTreeMap<Identifier, Expression>>,
}

impl Builder {
    /// Sets the `container` entry for this [`Runtime`].
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_ast as ast;
    ///
    /// use ast::v1::document::task::runtime::container::Value;
    /// use ast::v1::document::task::runtime::Builder;
    ///
    /// let container = Value::from(String::from("ubuntu:latest"));
    /// let runtime = Builder::default()
    ///     .container(container.clone())?
    ///     .try_build()?;
    ///
    /// assert_eq!(runtime.container(), Some(&container));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn container(mut self, container: container::Value) -> Result<Self> {
        if self.container.is_some() {
            return Err(Error::Multiple(MultipleError::Container));
        }

        self.container = Some(container);
        Ok(self)
    }

    /// Sets the `cpu` entry for this [`Runtime`].
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_ast as ast;
    ///
    /// use ast::v1::document::expression::Literal;
    /// use ast::v1::document::task::runtime::cpu::Value;
    /// use ast::v1::document::task::runtime::Builder;
    /// use ast::v1::document::Expression;
    ///
    /// let cpu = Value::try_from(Expression::Literal(Literal::Integer(4)))?;
    /// let runtime = Builder::default().cpu(cpu.clone())?.try_build()?;
    ///
    /// assert_eq!(runtime.cpu(), &cpu);
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn cpu(mut self, cpu: cpu::Value) -> Result<Self> {
        if self.cpu.is_some() {
            return Err(Error::Multiple(MultipleError::Cpu));
        }

        self.cpu = Some(cpu);
        Ok(self)
    }

    /// Inserts a hint into this [`Runtime`].
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_ast as ast;
    ///
    /// use ast::v1::document::expression::Literal;
    /// use ast::v1::document::identifier::singular::Identifier;
    /// use ast::v1::document::task::runtime::Builder;
    /// use ast::v1::document::Expression;
    ///
    /// let runtime = Builder::default()
    ///     .insert_hint(
    ///         Identifier::try_from("hello")?,
    ///         Expression::Literal(Literal::None),
    ///     )
    ///     .try_build()?;
    ///
    /// assert_eq!(
    ///     runtime.hints().unwrap().get("hello"),
    ///     Some(&Expression::Literal(Literal::None))
    /// );
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn insert_hint(mut self, key: Identifier, value: Expression) -> Self {
        let mut hints = self.hints.unwrap_or_default();
        hints.insert(key, value);
        self.hints = Some(hints);
        self
    }

    /// Consumes `self` to attempt to build a [`Runtime`].
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_ast as ast;
    ///
    /// use ast::v1::document::task::runtime::container::Value;
    /// use ast::v1::document::task::runtime::Builder;
    ///
    /// let container = Value::from(String::from("ubuntu:latest"));
    /// let runtime = Builder::default()
    ///     .container(container.clone())?
    ///     .try_build()?;
    ///
    /// assert_eq!(runtime.container(), Some(&container));
    ///
    /// # Ok::<(), Box<dyn std::error::Error>>(())
    /// ```
    pub fn try_build(self) -> Result<Runtime> {
        let cpu = self.cpu.unwrap_or_default();

        Ok(Runtime {
            container: self.container,
            cpu,
            hints: self.hints,
        })
    }
}
