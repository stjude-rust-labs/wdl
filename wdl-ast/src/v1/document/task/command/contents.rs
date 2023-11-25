//! Contents of a command.

use std::collections::HashMap;

/// The line ending.
#[cfg(windows)]
const LINE_ENDING: &str = "\r\n";
/// The line ending.
#[cfg(not(windows))]
const LINE_ENDING: &str = "\n";

/// An error when parsing [`Contents`].
#[derive(Debug)]
pub enum ParseError {
    /// Mixed tabs and spaces.
    MixedIndentationStyle,
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::MixedIndentationStyle => write!(f, "mixed indentation characters"),
        }
    }
}

impl std::error::Error for ParseError {}

/// An error related to [`Contents`].
#[derive(Debug)]
pub enum Error {
    /// A parse error.
    Parse(ParseError),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::Parse(err) => write!(f, "parse error: {err}"),
        }
    }
}

impl std::error::Error for Error {}

/// A [`Result`](std::result::Result) with an [`Error`].
type Result<T> = std::result::Result<T, Error>;

/// Contents of a command.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Contents(String);

impl std::ops::Deref for Contents {
    type Target = String;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl std::str::FromStr for Contents {
    type Err = Error;

    fn from_str(s: &str) -> Result<Self> {
        if s.is_empty() {
            return Ok(Self(String::new()));
        }

        let mut lines = s.lines();
        let mut results = Vec::new();

        // SAFETY: we just ensured that exactly one line exists, so this
        // will unwrap.
        let first_line = lines.next().unwrap();

        // NOTE: the first line is treated separately from the remaining lines.
        // This is because the first line is either (a) empty, which is harmless
        // and pushes and empty line into the results, or (b) has some content,
        // which means it is on the same line as the command. In the case of
        // (b), we don't want the spacing for the first line to influence the
        // stripping of whitespace on the remaining lines. For example,
        //
        // ```
        // command <<< echo 'hello'
        //     echo 'world'
        //     exit 0
        // >>>
        // ```
        //
        // Althought the above is considered bad form, the single space on the
        // first line should not dictate the stripping of whitespace for the
        // remaining lines (which are clearly indented with four spaces).

        if !first_line.is_empty() {
            results.push(first_line.to_string());
        }

        results.extend(strip_leading_whitespace(lines.collect())?);

        // If the last line is pure whitespace, ignore it.
        if let Some(line) = results.pop() {
            if !line.trim().is_empty() {
                results.push(line)
            }
        }

        Ok(Self(results.join(LINE_ENDING)))
    }
}

/// Strips common leading whitespace from a [`Vec<&str>`].
fn strip_leading_whitespace(lines: Vec<&str>) -> Result<Vec<String>> {
    // Count up all preceeding whitespace characters (including if whitespace
    // characters are mixed).
    let whitespace_by_line = lines
        .iter()
        .filter(|line| !line.trim().is_empty())
        .map(|line| {
            line.chars()
                .take_while(|c| c.is_whitespace())
                .fold(HashMap::new(), |mut counts, c| {
                    *counts.entry(c).or_insert(0usize) += 1usize;
                    counts
                })
        })
        .collect::<Vec<HashMap<char, usize>>>();

    let all_whitespace =
        whitespace_by_line
            .iter()
            .fold(HashMap::new(), |mut total_counts, line_counts| {
                for (c, count) in line_counts {
                    *total_counts.entry(*c).or_insert(0usize) += count
                }

                total_counts
            });

    if all_whitespace.len() > 1 {
        return Err(Error::Parse(ParseError::MixedIndentationStyle));
    }

    let indent = whitespace_by_line
        .into_iter()
        .map(|counts| counts.values().sum())
        .min()
        .unwrap_or_default();

    Ok(lines
        .iter()
        .map(|line| {
            if line.len() >= indent {
                line.chars().skip(indent).collect()
            } else {
                line.to_string()
            }
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_parses_contents_with_spaces_correctly(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let contents = "echo 'hello,'
    echo 'there'
    echo 'world'"
            .parse::<Contents>()?;

        assert_eq!(
            contents.as_str(),
            "echo 'hello,'\necho 'there'\necho 'world'"
        );

        let contents = "
    echo 'hello,'
    echo 'there'
    echo 'world'"
            .parse::<Contents>()?;

        assert_eq!(
            contents.as_str(),
            "echo 'hello,'\necho 'there'\necho 'world'"
        );

        let contents = "
        echo 'hello,'
    echo 'there'
    echo 'world'"
            .parse::<Contents>()?;

        assert_eq!(
            contents.as_str(),
            "    echo 'hello,'\necho 'there'\necho 'world'"
        );

        Ok(())
    }

    #[test]
    fn it_parses_contents_with_tabs_correctly(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let contents = "
\t\techo 'hello,'
\t\techo 'there'
\t\techo 'world'"
            .parse::<Contents>()?;
        assert_eq!(
            contents.as_str(),
            "echo 'hello,'\necho 'there'\necho 'world'"
        );

        Ok(())
    }

    #[test]
    fn it_keeps_preceeding_whitespace_on_the_same_line_as_the_command(
    ) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let contents = "    \nhello".parse::<Contents>()?;
        assert_eq!(contents.as_str(), "    \nhello");

        Ok(())
    }

    #[test]
    fn it_fails_on_mixed_spaces_and_tabs() {
        let err = "
            \techo 'hello,'
            echo 'there'
            echo 'world'"
            .parse::<Contents>()
            .unwrap_err();

        assert!(matches!(
            err,
            Error::Parse(ParseError::MixedIndentationStyle)
        ));
    }
}
