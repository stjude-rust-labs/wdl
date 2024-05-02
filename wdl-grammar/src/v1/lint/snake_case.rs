//! Workflows, tasks, and variables should be in snake case.

use std::collections::VecDeque;

use convert_case::Case;
use convert_case::Casing;
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

/// Detects names that should use snake case.
///
/// Workflows, tasks and variables should be declared using snake case.
#[derive(Debug)]
pub struct SnakeCase;

impl<'a> SnakeCase {
    /// Creates a warning for identifiers that are not proper snake case.
    fn not_snake_case(&self, warning: SnakeCaseWarning<'_>) -> lint::Warning
    where
        Self: Rule<&'a Pair<'a, v1::Rule>>,
    {
        lint::warning::Builder::default()
            .code(self.code())
            .level(lint::Level::Low)
            .group(self.group())
            .subject("identifier must be snake case")
            .body("Identifier must be formatted using snake case.")
            .push_location(warning.location)
            .fix(format!(
                "Replace {0} by {1}",
                warning.identifier, warning.properly_cased_identifier
            ))
            .try_build()
            .unwrap()
    }
}

/// Arguments for the `not_snake_case` function.
struct SnakeCaseWarning<'a> {
    /// Location of the warning.
    location: Location,

    /// Original identifier
    identifier: &'a str,

    /// Properly cased identifier
    properly_cased_identifier: &'a str,
}

impl Rule<&Pair<'_, v1::Rule>> for SnakeCase {
    fn code(&self) -> Code {
        Code::try_new(code::Kind::Warning, Version::V1, 6).unwrap()
    }

    fn group(&self) -> lint::Group {
        Group::Naming
    }

    fn check(&self, tree: &Pair<'_, v1::Rule>) -> lint::Result {
        let mut warnings = VecDeque::new();

        for node in tree.clone().into_inner().flatten() {
            if [
                crate::v1::Rule::task_name,
                crate::v1::Rule::workflow_name,
                crate::v1::Rule::bound_declaration_name,
                crate::v1::Rule::unbound_declaration_name,
            ]
            .contains(&node.as_rule())
            {
                let identifier: &str = node.as_span().as_str();
                let properly_cased_identifier: &str = &node.as_span().as_str().to_case(Case::Snake);
                if identifier != properly_cased_identifier {
                    warnings.push_back(SnakeCase.not_snake_case(SnakeCaseWarning {
                        location:
                            Location::try_from(node.as_span()).map_err(lint::Error::Location)?,
                        identifier,
                        properly_cased_identifier,
                    }));
                }
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
    fn it_catches_wrong_task_name() -> Result<(), Box<dyn std::error::Error>> {
        let tree = Parser::parse(
            Rule::task,
            r#"task wrongName {
            command <<< >>>
        }"#,
        )?
        .next()
        .unwrap();
        let warnings = SnakeCase.check(&tree)?.unwrap();

        assert_eq!(warnings.len(), 1);
        assert_eq!(
            warnings.first().to_string(),
            "[v1::W006::Naming/Low] identifier must be snake case (1:6-1:15)"
        );
        Ok(())
    }

    #[test]
    fn it_ignores_a_properly_cased_task_name() -> Result<(), Box<dyn std::error::Error>> {
        let tree = Parser::parse(
            Rule::task,
            r#"task good_name {
            command <<< >>>
        }"#,
        )?
        .next()
        .unwrap();
        let warnings = SnakeCase.check(&tree)?;
        assert!(warnings.is_none());
        Ok(())
    }

    #[test]
    fn it_catches_wrong_workflow_name() -> Result<(), Box<dyn std::error::Error>> {
        let tree = Parser::parse(
            Rule::workflow,
            r#"workflow wrongWorkflow {
                Int variable = 1
            }"#,
        )?
        .next()
        .unwrap();
        let warnings = SnakeCase.check(&tree)?.unwrap();

        assert_eq!(warnings.len(), 1);
        assert_eq!(
            warnings.first().to_string(),
            "[v1::W006::Naming/Low] identifier must be snake case (1:10-1:23)"
        );
        Ok(())
    }

    #[test]
    fn it_ignores_a_properly_cased_workflow_name() -> Result<(), Box<dyn std::error::Error>> {
        let tree = Parser::parse(
            Rule::workflow,
            r#"workflow good_workflow {
                Int variable = 1
            }"#,
        )?
        .next()
        .unwrap();
        let warnings = SnakeCase.check(&tree)?;
        assert!(warnings.is_none());
        Ok(())
    }

    #[test]
    fn it_catches_wrong_bound_declaration() -> Result<(), Box<dyn std::error::Error>> {
        let tree = Parser::parse(Rule::bound_declaration, r#"Int wrongVariable = 1"#)?
            .next()
            .unwrap();
        let warnings = SnakeCase.check(&tree)?.unwrap();

        assert_eq!(warnings.len(), 1);
        assert_eq!(
            warnings.first().to_string(),
            "[v1::W006::Naming/Low] identifier must be snake case (1:5-1:18)"
        );
        Ok(())
    }

    #[test]
    fn it_ignores_a_properly_cased_bound_declaration() -> Result<(), Box<dyn std::error::Error>> {
        let tree = Parser::parse(Rule::bound_declaration, r#"Int good_bound = 1"#)?
            .next()
            .unwrap();
        let warnings = SnakeCase.check(&tree)?;
        assert!(warnings.is_none());
        Ok(())
    }

    #[test]
    fn it_catches_wrong_unbound_declaration() -> Result<(), Box<dyn std::error::Error>> {
        let tree = Parser::parse(Rule::unbound_declaration, r#"Int wrongVariable"#)?
            .next()
            .unwrap();
        let warnings = SnakeCase.check(&tree)?.unwrap();

        assert_eq!(warnings.len(), 1);
        assert_eq!(
            warnings.first().to_string(),
            "[v1::W006::Naming/Low] identifier must be snake case (1:5-1:18)"
        );
        Ok(())
    }

    #[test]
    fn it_ignores_a_properly_cased_unbound_declaration() -> Result<(), Box<dyn std::error::Error>> {
        let tree = Parser::parse(Rule::unbound_declaration, r#"Int good_unbound"#)?
            .next()
            .unwrap();
        let warnings = SnakeCase.check(&tree)?;
        assert!(warnings.is_none());
        Ok(())
    }

    #[test]
    fn it_does_not_catch_struct_name() -> Result<(), Box<dyn std::error::Error>> {
        let tree = Parser::parse(
            Rule::document,
            r#"version 1.0
            struct myStruct {
                String my_string
                Int my_int
            }"#,
        )?
        .next()
        .unwrap();
        let warnings = SnakeCase.check(&tree)?;
        assert!(warnings.is_none());
        Ok(())
    }
}
