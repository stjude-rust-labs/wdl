//! Various lints for undesired whitespace.

use core::location::Position;
use core::Version;
use std::num::NonZeroUsize;

use pest::iterators::Pair;

use wdl_core as core;

use crate::v1;
use core::lint;
use core::lint::Group;
use core::lint::Rule;
use core::Code;
use core::Location;

/// Various lints for undesired whitespace.
#[derive(Debug)]
pub struct Whitespace;

impl<'a> Whitespace {
    /// Creates an error corresponding to a line being filled only with blank
    /// spaces.
    fn empty_line(&self, location: Location) -> lint::Warning
    where
        Self: Rule<&'a Pair<'a, v1::Rule>>,
    {
        // SAFETY: this error is written so that it will always unwrap.
        lint::warning::Builder::default()
            .code(self.code())
            .level(lint::Level::Low)
            .group(lint::Group::Style)
            .location(location)
            .subject("line contains only whitespace")
            .body(
                "Blank lines should be completely empty with no characters 
                between newlines.",
            )
            .fix("Remove the whitespace(s).")
            .try_build()
            .unwrap()
    }

    /// Creates an error corresponding to a line with a trailing space.
    fn trailing_space(&self, location: Location) -> lint::Warning
    where
        Self: Rule<&'a Pair<'a, v1::Rule>>,
    {
        // SAFETY: this error is written so that it will always unwrap.
        lint::warning::Builder::default()
            .code(self.code())
            .level(lint::Level::Low)
            .group(lint::Group::Style)
            .location(location)
            .subject("trailing space")
            .body(
                "This line contains one or more a trailing space(s).
                
                Blank lines should be completely empty with no characters
                between newlines.",
            )
            .fix("Remove the trailing space(s).")
            .try_build()
            .unwrap()
    }

    /// Creates an error corresponding to a line with a trailing tab.
    fn trailing_tab(&self, location: Location) -> lint::Warning
    where
        Self: Rule<&'a Pair<'a, v1::Rule>>,
    {
        // SAFETY: this error is written so that it will always unwrap.
        lint::warning::Builder::default()
            .code(self.code())
            .level(lint::Level::Low)
            .group(lint::Group::Style)
            .location(location)
            .subject("trailing tab")
            .body(
                "This line contains one or more a trailing tab(s).
                
                Blank lines should be completely empty with no characters
                between newlines.",
            )
            .fix("Remove the trailing tab(s).")
            .try_build()
            .unwrap()
    }
}

impl<'a> Rule<&Pair<'a, v1::Rule>> for Whitespace {
    fn code(&self) -> Code {
        // SAFETY: this manually crafted to unwrap successfully every time.
        Code::try_new(Version::V1, 1).unwrap()
    }

    fn group(&self) -> lint::Group {
        Group::Style
    }

    fn check(&self, tree: &Pair<'a, v1::Rule>) -> lint::Result {
        let mut results = Vec::new();

        for (i, line) in tree.as_str().lines().enumerate() {
            if line.is_empty() {
                continue;
            }

            // SAFETY: this will always unwrap because we add one to the current
            // enumeration index. Technically it will not unwrap for usize::MAX
            // - 1, but we don't expect that any WDL document will have that
            //   many lines.
            let line_no = NonZeroUsize::try_from(i + 1).unwrap();

            // NOTE: empty lines will always start at the first column of the
            // line.
            //
            // SAFTEY: a literal `1` will always unwrap to a [`NonZeroUsize`].
            let start = Position::new(line_no, NonZeroUsize::try_from(1).unwrap());

            // SAFETY: we just ensured above that the line is not empty. As
            // such, this will always unwrap.
            let end = Position::new(line_no, NonZeroUsize::try_from(line.len()).unwrap());

            let trimmed_line = line.trim();

            if trimmed_line.is_empty() && line != trimmed_line {
                results.push(self.empty_line(Location::Span { start, end }));
            } else if line.ends_with(' ') {
                results.push(self.trailing_space(Location::Position { position: end }));
            } else if line.ends_with('\t') {
                results.push(self.trailing_tab(Location::Position { position: end }));
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
    use pest::Parser as _;

    use crate::v1::parse::Parser;
    use crate::v1::Rule;
    use wdl_core::lint::Rule as _;

    use super::*;

    #[test]
    fn it_catches_an_empty_line() -> Result<(), Box<dyn std::error::Error>> {
        let tree = Parser::parse(Rule::document, "version 1.1\n   \n")?
            .next()
            .unwrap();
        let warning = Whitespace.check(&tree)?.unwrap();

        assert_eq!(warning.len(), 1);
        assert_eq!(
            warning.first().unwrap().to_string(),
            "[v1::001::Style/Low] line contains only whitespace (2:1-2:3)"
        );

        Ok(())
    }

    #[test]
    fn it_catches_a_trailing_space() -> Result<(), Box<dyn std::error::Error>> {
        let tree = Parser::parse(Rule::document, "version 1.1 ")?
            .next()
            .unwrap();
        let warning = Whitespace.check(&tree)?.unwrap();

        assert_eq!(warning.len(), 1);
        assert_eq!(
            warning.first().unwrap().to_string(),
            "[v1::001::Style/Low] trailing space (1:12)"
        );

        Ok(())
    }

    #[test]
    fn it_catches_a_trailing_tab() -> Result<(), Box<dyn std::error::Error>> {
        let tree = Parser::parse(Rule::document, "version 1.1\t")?
            .next()
            .unwrap();
        let warning = Whitespace.check(&tree)?.unwrap();

        assert_eq!(warning.len(), 1);
        assert_eq!(
            warning.first().unwrap().to_string(),
            "[v1::001::Style/Low] trailing tab (1:12)"
        );

        Ok(())
    }

    #[test]
    fn it_unwraps_a_trailing_space_error() {
        let warning = Whitespace.trailing_space(Location::Position {
            position: Position::new(
                NonZeroUsize::try_from(1).unwrap(),
                NonZeroUsize::try_from(1).unwrap(),
            ),
        });
        assert_eq!(
            warning.to_string(),
            "[v1::001::Style/Low] trailing space (1:1)"
        )
    }

    #[test]
    fn it_unwraps_a_trailing_tab_error() {
        let warning = Whitespace.trailing_tab(Location::Position {
            position: Position::new(
                NonZeroUsize::try_from(1).unwrap(),
                NonZeroUsize::try_from(1).unwrap(),
            ),
        });
        assert_eq!(
            warning.to_string(),
            "[v1::001::Style/Low] trailing tab (1:1)"
        )
    }

    #[test]
    fn it_unwraps_an_empty_line_error() {
        let warning = Whitespace.empty_line(Location::Span {
            start: Position::new(
                NonZeroUsize::try_from(1).unwrap(),
                NonZeroUsize::try_from(1).unwrap(),
            ),
            end: Position::new(
                NonZeroUsize::try_from(1).unwrap(),
                NonZeroUsize::try_from(1).unwrap(),
            ),
        });
        assert_eq!(
            warning.to_string(),
            "[v1::001::Style/Low] line contains only whitespace (1:1-1:1)"
        )
    }
}
