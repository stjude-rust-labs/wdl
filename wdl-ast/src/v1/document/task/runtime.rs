//! Runtime section.

use std::collections::BTreeMap;

use pest::iterators::Pair;

use wdl_grammar as grammar;

use grammar::v1::Rule;

use crate::v1::document::expression;

use crate::v1::document::identifier::singular;
use crate::v1::document::identifier::singular::Identifier;
use crate::v1::document::Expression;
use crate::v1::macros::check_node;
use crate::v1::macros::extract_one;
use crate::v1::macros::unwrap_one;

mod builder;
pub mod container;
pub mod cpu;

pub use builder::Builder;

/// An error related to a [`Runtime`].
#[derive(Debug)]
pub enum Error {
    /// A builder error.
    Builder(builder::Error),

    /// A common error.
    Common(crate::v1::Error),

    /// An error with the `container` entry.
    Container(container::Error),

    /// An error with the `cpu` entry.
    Cpu(cpu::Error),

    /// An expression error.
    Expression(expression::Error),

    /// An identifier error.
    Identifier(singular::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Builder(err) => write!(f, "builder error: {err}"),
            Error::Common(err) => write!(f, "{err}"),
            Error::Container(err) => write!(f, "`container` entry error: {err}"),
            Error::Cpu(err) => write!(f, "`cpu` entry error: {err}"),
            Error::Expression(err) => write!(f, "expression error: {err}"),
            Error::Identifier(err) => write!(f, "identifier error: {err}"),
        }
    }
}

impl std::error::Error for Error {}

/// A runtime section.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Runtime {
    /// The container entry.
    container: Option<container::Value>,

    /// The cpu entry.
    cpu: cpu::Value,

    /// Other included runtime hints.
    hints: Option<BTreeMap<Identifier, Expression>>,
}

impl Runtime {
    /// Gets the `container` entry for this [`Runtime`] (if it exists).
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
    pub fn container(&self) -> Option<&container::Value> {
        self.container.as_ref()
    }

    /// Gets the `cpu` entry for this [`Runtime`].
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
    pub fn cpu(&self) -> &cpu::Value {
        &self.cpu
    }

    /// Gets the hints for this [`Runtime`] (if they exist).
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
    pub fn hints(&self) -> Option<&BTreeMap<Identifier, Expression>> {
        self.hints.as_ref()
    }
}

impl TryFrom<Pair<'_, Rule>> for Runtime {
    type Error = Error;

    fn try_from(node: Pair<'_, Rule>) -> Result<Self, Self::Error> {
        check_node!(node, task_runtime);
        let mut builder = Builder::default();

        for node in node.into_inner() {
            match node.as_rule() {
                Rule::task_runtime_mapping => {
                    let key_node = extract_one!(
                        node.clone(),
                        task_runtime_mapping_key,
                        task_runtime_mapping,
                        Error::Common
                    )?;
                    let value_node = extract_one!(
                        node,
                        task_runtime_mapping_value,
                        task_runtime_mapping,
                        Error::Common
                    )?;

                    match key_node.as_str() {
                        "container" | "docker" => {
                            let container =
                                container::Value::try_from(value_node).map_err(Error::Container)?;
                            builder = builder.container(container).map_err(Error::Builder)?
                        }
                        "cpu" => {
                            let cpu = cpu::Value::try_from(value_node).map_err(Error::Cpu)?;
                            builder = builder.cpu(cpu).map_err(Error::Builder)?;
                        }
                        _ => {
                            let identifier_node = unwrap_one!(key_node, task_runtime_mapping_key)?;
                            let key =
                                Identifier::try_from(identifier_node).map_err(Error::Identifier)?;

                            let expression_node =
                                unwrap_one!(value_node, task_runtime_mapping_value)?;
                            let value =
                                Expression::try_from(expression_node).map_err(Error::Expression)?;

                            builder = builder.insert_hint(key, value);
                        }
                    }
                }
                Rule::WHITESPACE => {}
                Rule::COMMENT => {}
                rule => unreachable!("task runtime should not contain {:?}", rule),
            }
        }

        builder.try_build().map_err(Error::Builder)
    }
}
