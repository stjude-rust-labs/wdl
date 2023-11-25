//! Invalid escape character(s) within a string.

use pest::iterators::Pair;

use wdl_core as core;

use core::validation;
use core::validation::Rule;
use core::Code;
use core::Location;
use core::Version;

use crate::v1;

/// An invalid escape character within a string.
#[derive(Debug)]
pub struct InvalidEscapeCharacter;

impl Rule<&Pair<'_, v1::Rule>> for InvalidEscapeCharacter {
    fn code(&self) -> Code {
        // SAFETY: this manually crafted to unwrap successfully every time.
        Code::try_new(Version::V1, 1).unwrap()
    }

    fn validate(&self, tree: &Pair<'_, v1::Rule>) -> validation::Result {
        tree.clone()
            .into_inner()
            .flatten()
            .try_for_each(|node| match node.as_rule() {
                v1::Rule::char_escaped_invalid => {
                    let location = match Location::try_from(node.as_span()) {
                        Ok(location) => location,
                        Err(_) => unreachable!(),
                    };

                    Err(validation::error::Builder::default()
                        .code(self.code())
                        .subject(format!(
                            "invalid escape character '{}' in string",
                            node.as_str(),
                        ))
                        .body("An invalid character was found in a string.")
                        .location(location)
                        .fix(
                            "Remove the invalid character. If the character contains \
                            escaped characters (e.g., `\\n`), then you may need to \
                            double escape the backslashes.",
                        )
                        .try_build()
                        .unwrap())
                }
                _ => Ok(()),
            })
    }
}

#[cfg(test)]
mod tests {
    use pest::Parser as _;

    use crate::v1::parse::Parser;
    use crate::v1::Rule;
    use wdl_core::validation::Rule as _;

    use super::*;

    #[test]
    fn it_catches_an_invalid_escape_character() -> Result<(), Box<dyn std::error::Error>> {
        let tree = Parser::parse(Rule::string, "\"\\.\"")?.next().unwrap();
        let error = InvalidEscapeCharacter.validate(&tree).unwrap_err();

        assert_eq!(
            error.to_string(),
            String::from("[v1::001] invalid escape character '\\.' in string (1:2-1:4)")
        );

        Ok(())
    }
}
