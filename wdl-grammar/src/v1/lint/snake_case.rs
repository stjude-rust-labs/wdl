use std::collections::VecDeque;

use convert_case::{Case, Casing};
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

#[derive(Debug)]
pub struct NotSnakeCase;

impl<'a> NotSnakeCase {
    fn not_snake_case(&self, warning: SnakeCaseWarning<'_>) -> lint::Warning
    where
        Self: Rule<&'a Pair<'a, v1::Rule>>,
    {
        lint::warning::Builder::default()
            .code(self.code())
            .level(lint::Level::Low)
            .group(self.group())
            .subject("missing snake case")
            .body("Declaration must be formatted using snake case.")
            .push_location(warning.location)
            .fix(format!(
                "Replace {0} by {1}",
                warning.declaration, warning.cased_declaration
            ))
            .try_build()
            .unwrap()
    }
}

struct SnakeCaseWarning<'a> {
    location: Location,
    declaration: &'a str,
    cased_declaration: &'a str,
}

impl Rule<&Pair<'_, v1::Rule>> for NotSnakeCase {
    fn code(&self) -> Code {
        Code::try_new(code::Kind::Warning, Version::V1, 5).unwrap()
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
                let declaration: &str = node.as_span().as_str();
                let cased_declaration: &str = &node.as_span().as_str().to_case(Case::Snake);
                if declaration != cased_declaration {
                    let warning = SnakeCaseWarning {
                        location: Location::try_from(node.as_span())
                            .map_err(lint::Error::Location)?,
                        declaration,
                        cased_declaration,
                    };
                    warnings.push_back(NotSnakeCase.not_snake_case(warning));
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
            "task thisBad {
            command <<< >>>
        }",
        )?
        .next()
        .unwrap();
        let warnings = NotSnakeCase.check(&tree)?.unwrap();

        assert_eq!(warnings.len(), 1);
        assert_eq!(
            warnings.first().to_string(),
            "[v1::W005::Naming/Low] missing snake case (1:6-1:13)"
        );
        Ok(())
    }
}
