//! Command section.

use pest::iterators::Pair;

use wdl_grammar as grammar;

use grammar::v1::Rule;

use crate::v1::macros::check_node;
use crate::v1::macros::dive_one;
use crate::v1::macros::unwrap_one;

mod contents;

pub use contents::Contents;

/// An error related to a [`Command`].
#[derive(Debug)]
pub enum Error {
    /// A common error.
    Common(crate::v1::Error),

    /// Contents error.
    Contents(contents::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Common(err) => write!(f, "{err}"),
            Error::Contents(err) => write!(f, "contents error: {err}"),
        }
    }
}

impl std::error::Error for Error {}

/// A command withing a task.
///
/// **Note:** this crate does no inspection of the underlying command. Instead,
/// we make the command available for other tools (e.g.,
/// [shellcheck](https://www.shellcheck.net/)).
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum Command {
    /// A heredoc style command.
    HereDoc(Contents),

    /// A curly bracket style command.
    Curly(Contents),
}

impl Command {
    /// Gets the inner contents of the command as a reference to a [`str`].
    ///
    /// # Examples
    ///
    /// ```
    /// use wdl_ast as ast;
    ///
    /// use ast::v1::document::task::command::Contents;
    /// use ast::v1::document::task::Command;
    ///
    /// let contents = "echo 'Hello, world!'".parse::<Contents>().unwrap();
    /// let command = Command::HereDoc(contents);
    /// assert_eq!(command.as_str(), "echo 'Hello, world!'");
    /// ```
    pub fn as_str(&self) -> &str {
        match self {
            Command::HereDoc(contents) => contents.as_str(),
            Command::Curly(contents) => contents.as_str(),
        }
    }
}

impl std::fmt::Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl TryFrom<Pair<'_, grammar::v1::Rule>> for Command {
    type Error = Error;

    fn try_from(node: Pair<'_, grammar::v1::Rule>) -> Result<Self, Self::Error> {
        check_node!(node, task_command);
        let node = unwrap_one!(node, task_command)?;

        Ok(match node.as_rule() {
            Rule::command_heredoc => {
                let contents_node = dive_one!(
                    node,
                    command_heredoc_contents,
                    command_heredoc,
                    Error::Common
                )?;
                let contents = contents_node
                    .as_str()
                    .parse::<Contents>()
                    .map_err(Error::Contents)?;
                Command::HereDoc(contents)
            }
            Rule::command_curly => {
                let contents_node =
                    dive_one!(node, command_curly_contents, command_curly, Error::Common)?;
                let contents = contents_node
                    .as_str()
                    .parse::<Contents>()
                    .map_err(Error::Contents)?;
                Command::Curly(contents)
            }
            _ => {
                unreachable!(
                    "a task command's inner element must be either a heredoc or a curly command"
                )
            }
        })
    }
}
