//! Header comments are full line comments starting with a double pound sign

use std::collections::VecDeque;

use nonempty::NonEmpty;
use pest::iterators::Pair;
use wdl_core::concern::code;
use wdl_core::concern::lint;
use wdl_core::concern::lint::Group;
use wdl_core::concern::lint::Rule;
use wdl_core::concern::Code;
use wdl_core::file::Location;
use wdl_core::Version;

use crate::v1;

/// Detects header comments declaration
#[derive(Debug)]
pub struct HeaderComment;

impl<'a> HeaderComment {
    /// Creates a warning for header comments without double pound sign
    fn missing_double_pound_sign(&self, location: Location) -> lint::Warning
    where
        Self: Rule<&'a Pair<'a, v1::Rule>>,
    {
        lint::warning::Builder::default()
            .code(self.code())
            .level(lint::Level::Low)
            .group(self.group())
            .subject("header comments without a double pound sign")
            .body(
                "Header comments are full line comments before the version declaration and they \
                 start with a double pound sign.",
            )
            .push_location(location)
            .fix("Add a pound sign at the start of the line.")
            .try_build()
            .unwrap()
    }

    /// Creates a warning for header comments after version declaration
    fn header_comment_after_version(&self, location: Location) -> lint::Warning
    where
        Self: Rule<&'a Pair<'a, v1::Rule>>,
    {
        lint::warning::Builder::default()
            .code(self.code())
            .level(lint::Level::Low)
            .group(self.group())
            .subject("double pound sign are reserved for header comment")
            .body(
                "Only full line comment before version declaration should start with a double \
                 pound sign.",
            )
            .push_location(location)
            .fix("Remove a pound sign at the start of the comment.")
            .try_build()
            .unwrap()
    }
}

impl<'a> Rule<&Pair<'a, v1::Rule>> for HeaderComment {
    fn code(&self) -> Code {
        // SAFETY: this manually crafted to unwrap successfully every time.
        Code::try_new(code::Kind::Warning, Version::V1, 10).unwrap()
    }

    fn group(&self) -> lint::Group {
        Group::Style
    }

    fn check(&self, tree: &Pair<'a, v1::Rule>) -> lint::Result {
        let mut warnings = VecDeque::new();

        let mut is_header = true;

        for node in tree.clone().into_inner().flatten() {
            match node.as_rule() {
                v1::Rule::version => {
                    is_header = false;
                }
                v1::Rule::COMMENT => {
                    // Catches missing double pound sign
                    if is_header & !node.as_str().starts_with("##") {
                        let location =
                            Location::try_from(node.as_span()).map_err(lint::Error::Location)?;
                        warnings.push_back(self.missing_double_pound_sign(location));
                    }

                    // Catches header comment after version declaration
                    if !is_header & node.as_str().starts_with("##") {
                        let location =
                            Location::try_from(node.as_span()).map_err(lint::Error::Location)?;
                        warnings.push_back(self.header_comment_after_version(location));
                    }
                }
                _ => {}
            }
        }

        match warnings.pop_front() {
            Some(front) => {
                let mut results = NonEmpty::new(front);
                results.extend(warnings);
                Ok(Some(results))
            }
            None => Ok(None),
        }
    }
}

#[cfg(test)]
mod tests {
    use pest::Parser as _;
    use wdl_core::concern::lint::Rule as _;

    use super::*;
    use crate::v1::parse::Parser;
    use crate::v1::Rule;

    #[test]
    fn it_catches_badly_formatted_header_comment() -> Result<(), Box<dyn std::error::Error>> {
        let tree = Parser::parse(
            Rule::document,
            r#"# a comment
version 1.0
"#,
        )?
        .next()
        .unwrap();

        let warnings = HeaderComment.check(&tree)?.unwrap();
        assert_eq!(warnings.len(), 1);
        assert_eq!(
            warnings.first().to_string(),
            "[v1::W010::Style/Low] header comments without a double pound sign (1:1-1:12)"
        );
        Ok(())
    }

    #[test]
    fn it_catches_header_comment_after_version() -> Result<(), Box<dyn std::error::Error>> {
        let tree = Parser::parse(
            Rule::document,
            r#"## a comment
version 1.0

## a wrong comment
"#,
        )?
        .next()
        .unwrap();

        let warnings = HeaderComment.check(&tree)?.unwrap();
        assert_eq!(warnings.len(), 1);
        assert_eq!(
            warnings.first().to_string(),
            "[v1::W010::Style/Low] double pound sign are reserved for header comment (4:1-4:19)"
        );
        Ok(())
    }

    #[test]
    fn it_ignores_a_properly_formatted_header_comment() -> Result<(), Box<dyn std::error::Error>> {
        let tree = Parser::parse(
            Rule::document,
            r#"## a comment
version 1.0
"#,
        )?
        .next()
        .unwrap();
        let warnings = HeaderComment.check(&tree)?;
        assert!(warnings.is_none());
        Ok(())
    }
}
