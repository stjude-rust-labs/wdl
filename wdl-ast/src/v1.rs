//! WDL 1.x.

use wdl_core as core;
use wdl_grammar as grammar;

use core::lint::Linter;
use core::validation::Validator;

pub mod document;
pub mod lint;
pub mod macros;
pub mod validation;

pub use document::Document;

use crate::common::Tree;

/// An error related to building an abstract syntax tree.
#[derive(Debug)]
pub enum Error {
    /// A document error.
    Document(Box<document::Error>),

    /// Attempted to create an AST element from a node that was incompatible.
    InvalidNode(String),

    /// Missing a node that was expected to exist.
    MissingNode(String),

    /// The [parse tree](grammar::common::Tree) had no root nodes.
    MissingRootNode,

    /// Multiple nodes were found when only one was expected.
    MultipleNodes(String),

    /// The [parse tree](grammar::common::Tree) had multiple root nodes.
    MultipleRootNodes,
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidNode(explanation) => write!(f, "invalid node: {explanation}"),
            Error::Document(err) => write!(f, "document error: {err}"),
            Error::MissingNode(explanation) => write!(f, "missing node: {explanation}"),
            Error::MissingRootNode => write!(f, "parse tree had no root nodes"),
            Error::MultipleNodes(explanation) => write!(f, "multiple nodes: {explanation}"),
            Error::MultipleRootNodes => write!(f, "parse tree had multiple root nodes"),
        }
    }
}

impl std::error::Error for Error {}

/// Parses an abstract syntax tree (in the form of a [`Document`]) from a
/// [`Tree`].
///
/// # Examples
///
/// ```
/// use wdl_ast as ast;
/// use wdl_grammar as grammar;
///
/// use grammar::v1::Rule;
///
/// let pt = grammar::v1::parse("version 1.1").unwrap();
/// let ast = ast::v1::parse(pt).unwrap();
///
/// assert_eq!(ast.version(), &ast::v1::document::Version::OneDotOne);
/// ```
pub fn parse(tree: grammar::common::Tree<'_, grammar::v1::Rule>) -> Result<Tree, super::Error> {
    let document = Document::try_from(tree.into_inner())
        .map_err(|err| super::Error::ParseV1(Box::new(Error::Document(Box::new(err)))))?;

    Validator::validate(&document, validation::rules())
        .map_err(Box::new)
        .map_err(super::Error::Validation)?;

    let warnings = Linter::lint(&document, lint::rules()).unwrap();

    Ok(Tree::new(document, warnings))
}
