//! Incorrect version declaration placement

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

/// Detects an improperly placed version declaration.
#[derive(Debug)]
pub struct VersionDeclarationPlacement;

impl<'a> VersionDeclarationPlacement {
    /// Generates a validation error for an improperly placed version
    /// declaration.
    fn misplaced_version(&self, location: Location) -> lint::Warning
    where
        Self: Rule<&'a Pair<'a, v1::Rule>>,
    {
        // SAFETY: this error is written so that it will always unwrap.
        lint::warning::Builder::default()
            .code(self.code())
            .level(lint::Level::Medium)
            .group(lint::Group::Spacing)
            .subject("Improperly placed version declaration")
            .body(
                "The version declaration must be the first line of a WDL document or one blank \
                 line after header comments.",
            )
            .push_location(location)
            .fix(
                "Move the version declaration to the first line of the WDL document or one blank \
                 line after header comments.",
            )
            .try_build()
            .unwrap()
    }
}

impl<'a> Rule<&Pair<'a, v1::Rule>> for VersionDeclarationPlacement {
    fn code(&self) -> Code {
        // SAFETY: this manually crafted to unwrap successfully every time.
        Code::try_new(code::Kind::Error, Version::V1, 9).unwrap()
    }

    fn group(&self) -> Group {
        Group::Spacing
    }

    fn check(&self, tree: &Pair<'_, v1::Rule>) -> lint::Result {
        let mut warnings = VecDeque::new();

        // Optionally consume comment nodes, if found, allow exactly one empty line
        // between the comment and the version declaration.
        let mut comment = 0;
        let mut newline = 0;
        let mut anything_else = 0;

        // This will never get used. Validation rules require a version statement.
        let mut location: Location = Location::Unplaced;

        for node in tree.clone().into_inner() {
            match node.as_rule() {
                v1::Rule::COMMENT => {
                    comment += 1;
                }
                v1::Rule::WHITESPACE => {
                    if node.as_str() == "\n" {
                        newline += 1;
                    } else {
                        anything_else += 1;
                    }
                }
                v1::Rule::version => {
                    location = Location::try_from(node.as_span()).map_err(lint::Error::Location)?;
                    break;
                }
                _ => {
                    anything_else += 1;
                }
            }
        }

        // If anything other than version, comment, and newlines detected.
        if (anything_else > 0)
            // If comments detected, there should be one empty line between comments and version.
            || (comment > 0 && newline != (comment + 1))
            // If no comments detected, version should be the first line.
            || (comment == 0 && newline != 0)
        {
            warnings.push_back(self.misplaced_version(location))
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
    fn it_catches_missing_newline() -> Result<(), Box<dyn std::error::Error>> {
        let tree = Parser::parse(
            Rule::document,
            r#"## Header comment
version 1.0"#,
        )?
        .next()
        .unwrap();

        let warnings = VersionDeclarationPlacement.check(&tree)?.unwrap();

        assert_eq!(warnings.len(), 1);
        assert_eq!(
            warnings.first().to_string(),
            "[v1::E009::Spacing/Medium] Improperly placed version declaration (2:1-2:12)"
        );
        Ok(())
    }

    #[test]
    fn it_catches_leading_newline_with_comment() -> Result<(), Box<dyn std::error::Error>> {
        let tree = Parser::parse(
            Rule::document,
            r#"
## Header comment

version 1.0"#,
        )?
        .next()
        .unwrap();

        let warnings = VersionDeclarationPlacement.check(&tree)?.unwrap();

        assert_eq!(warnings.len(), 1);
        assert_eq!(
            warnings.first().to_string(),
            "[v1::E009::Spacing/Medium] Improperly placed version declaration (4:1-4:12)"
        );
        Ok(())
    }

    #[test]
    fn it_catches_leading_newline() -> Result<(), Box<dyn std::error::Error>> {
        let tree = Parser::parse(
            Rule::document,
            r#"
version 1.0"#,
        )?
        .next()
        .unwrap();

        let warnings = VersionDeclarationPlacement.check(&tree)?.unwrap();

        assert_eq!(warnings.len(), 1);
        assert_eq!(
            warnings.first().to_string(),
            "[v1::E009::Spacing/Medium] Improperly placed version declaration (2:1-2:12)"
        );
        Ok(())
    }

    #[test]
    fn it_handles_correct() -> Result<(), Box<dyn std::error::Error>> {
        let tree = Parser::parse(
            Rule::document,
            r#"## Header comment

version 1.0"#,
        )?
        .next()
        .unwrap();

        assert!(VersionDeclarationPlacement.check(&tree)?.is_none());
        Ok(())
    }

    #[test]
    fn it_handles_multiple_comments_correct() -> Result<(), Box<dyn std::error::Error>> {
        let tree = Parser::parse(
            Rule::document,
            r#"## Header comment
## Another comment

version 1.0"#,
        )?
        .next()
        .unwrap();

        assert!(VersionDeclarationPlacement.check(&tree)?.is_none());
        Ok(())
    }
}
