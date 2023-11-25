//! Replace curly command blocks with heredoc command blocks.

use core::Version;

use pest::iterators::Pair;

use wdl_core as core;

use crate::v1;
use core::lint;
use core::lint::Group;
use core::lint::Rule;
use core::Code;
use core::Location;

/// Replace curly command blocks with heredoc command blocks.
///
/// Curly command blocks are no longer considered idiomatic WDL
/// ([link](https://github.com/openwdl/wdl/blob/main/versions/1.1/SPEC.md#command-section)).
/// Idiomatic WDL code uses heredoc command blocks instead.
#[derive(Debug)]
pub struct NoCurlyCommands;

impl<'a> NoCurlyCommands {
    /// Creates an error corresponding to a line with a trailing tab.
    fn no_curly_commands(&self, location: Location) -> lint::Warning
    where
        Self: Rule<&'a Pair<'a, v1::Rule>>,
    {
        // SAFETY: this error is written so that it will always unwrap.
        lint::warning::Builder::default()
            .code(self.code())
            .level(lint::Level::Medium)
            .group(lint::Group::Pedantic)
            .location(location)
            .subject("curly command found")
            .body(
                "Command blocks using curly braces (`{}`) are considered less
                idiomatic than heredoc commands.",
            )
            .fix("Replace the curly command block with a heredoc command block.")
            .try_build()
            .unwrap()
    }
}

impl<'a> Rule<&'a Pair<'a, v1::Rule>> for NoCurlyCommands {
    fn code(&self) -> Code {
        // SAFETY: this manually crafted to unwrap successfully every time.
        Code::try_new(Version::V1, 2).unwrap()
    }

    fn group(&self) -> lint::Group {
        Group::Style
    }

    fn check(&self, tree: &'a Pair<'_, v1::Rule>) -> lint::Result {
        let mut results = Vec::new();

        for node in tree.clone().into_inner().flatten() {
            if node.as_rule() == v1::Rule::command_curly {
                let location = Location::try_from(node.as_span())?;
                results.push(self.no_curly_commands(location));
            }
        }

        match results.is_empty() {
            true => Ok(None),
            false => Ok(Some(results)),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::num::NonZeroUsize;
    use wdl_core::location::Position;

    use pest::Parser as _;

    use crate::v1::parse::Parser;
    use crate::v1::Rule;
    use wdl_core::lint::Rule as _;

    use super::*;

    #[test]
    fn it_catches_a_curly_command() -> Result<(), Box<dyn std::error::Error>> {
        let tree = Parser::parse(
            Rule::task,
            "task hello {
    command {}
}",
        )?
        .next()
        .unwrap();
        let warning = NoCurlyCommands.check(&tree)?.unwrap();

        assert_eq!(warning.len(), 1);
        assert_eq!(
            warning.first().unwrap().to_string(),
            "[v1::002::Pedantic/Medium] curly command found (2:5-2:15)"
        );

        Ok(())
    }

    #[test]
    fn it_does_not_catch_a_heredoc_command() -> Result<(), Box<dyn std::error::Error>> {
        let tree = Parser::parse(Rule::command_heredoc, "command <<<>>>")?
            .next()
            .unwrap();
        assert!(NoCurlyCommands.check(&tree)?.is_none());

        Ok(())
    }

    #[test]
    fn it_unwraps_a_no_curly_commands_error() {
        let location = Location::Position {
            position: Position::new(
                NonZeroUsize::try_from(1).unwrap(),
                NonZeroUsize::try_from(1).unwrap(),
            ),
        };

        let warning = NoCurlyCommands.no_curly_commands(location);
        assert_eq!(
            warning.to_string(),
            "[v1::002::Pedantic/Medium] curly command found (1:1)"
        )
    }
}
