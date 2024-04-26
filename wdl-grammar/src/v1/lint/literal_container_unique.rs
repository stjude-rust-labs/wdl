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

/// Detects literal containers with multiple tag versions.
///
/// Containers declared as literal strings should only use a single tag.
/// Multiple tasks should not use different versions of a single container.
#[derive(Debug)]
pub struct LiteralContainerUnique;

impl<'a> LiteralContainerUnique {
    /// Creates a warning if the same base image has multiple tags.
    fn literal_container_unique(&self, location: Location) -> lint::Warning
    where
        Self: Rule<&'a Pair<'a, v1::Rule>>,
    {
        lint::warning::Builder::default()
            .code(self.code())
            .level(lint::Level::Medium)
            .group(self.group())
            .subject("duplicate literal container")
            .body(
                "Literal containers should be unique within a task. This is because the same \
                 container can be used in multiple tasks.",
            )
            .push_location(location)
            .fix("Use one tag per container name.")
            .try_build()
            .unwrap()
    }

    /// Separate a docker image and its tag from a string
    fn separate_docker_image_tag(&self, container: &str) -> (String, String) {
        let mut split = container.split(':');
        let image = split.next().unwrap();
        let tag = split.next().unwrap_or("latest");
        (image.to_string(), tag.to_string())
    }
}

impl<'a> Rule<&'a Pair<'a, crate::v1::Rule>> for LiteralContainerUnique {
    fn code(&self) -> Code {
        Code::try_new(code::Kind::Warning, Version::V1, 6).unwrap()
    }

    fn group(&self) -> Group {
        Group::Container
    }

    fn check(&self, tree: &'a Pair<'a, crate::v1::Rule>) -> lint::Result {
        let mut warnings = VecDeque::new();

        let mut container_names: Vec<String> = Vec::new();
        for node in tree.clone().into_inner().flatten() {
            if node.as_rule() == crate::v1::Rule::task {
                for inner_node in node.clone().into_inner().flatten() {
                    if inner_node.as_rule() == crate::v1::Rule::task_runtime {
                        for runtime_node in inner_node.clone().into_inner().flatten() {
                            if runtime_node.as_rule() == crate::v1::Rule::task_runtime_mapping {
                                // Each runtime_node is a entry in the runtime block
                                let mut is_container = false;
                                for element_node in runtime_node.clone().into_inner().flatten() {
                                    // Each element_node is a pair in the runtime block
                                    if element_node.as_rule()
                                        == crate::v1::Rule::task_runtime_mapping_key
                                    {
                                        // Check to see if this is a `container` or `docker` key
                                        let field = element_node.as_str();
                                        if field == "container" || field == "docker" {
                                            is_container = true;
                                        }
                                    }
                                    if element_node.as_rule()
                                        == crate::v1::Rule::task_runtime_mapping_value
                                    {
                                        // Check to see if this is a literal container
                                        if is_container {
                                            for node in element_node.clone().into_inner().flatten()
                                            {
                                                if node.as_rule()
                                                    == crate::v1::Rule::string_literal_contents
                                                {
                                                    let start =
                                                        node.as_span().start_pos().line_col().1 - 1;
                                                    let end =
                                                        node.as_span().end_pos().line_col().1 + 1;
                                                    let container_start = element_node
                                                        .as_span()
                                                        .start_pos()
                                                        .line_col()
                                                        .1;
                                                    let container_end = element_node
                                                        .as_span()
                                                        .end_pos()
                                                        .line_col()
                                                        .1;

                                                    if container_start == start
                                                        && container_end == end
                                                    {
                                                        let mut was_seen = false;
                                                        let container = node.as_span().as_str();
                                                        let (image, tag) = self
                                                            .separate_docker_image_tag(container);
                                                        for seen in container_names.iter() {
                                                            let (seen_image, seen_tag) = self
                                                                .separate_docker_image_tag(seen);
                                                            if image == seen_image
                                                                && tag != seen_tag
                                                            {
                                                                let location = Location::try_from(
                                                                    element_node.as_span(),
                                                                )
                                                                .map_err(lint::Error::Location)?;
                                                                warnings.push_back(
                                                                    self.literal_container_unique(
                                                                        location,
                                                                    ),
                                                                );
                                                            }
                                                            if image == seen_image
                                                                && tag == seen_tag
                                                            {
                                                                was_seen = true;
                                                                break;
                                                            }
                                                        }
                                                        if !was_seen {
                                                            container_names
                                                                .push(container.to_string());
                                                        }
                                                    }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
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
    fn it_catches_duplicate_literal_container() -> Result<(), Box<dyn std::error::Error>> {
        let tree = Parser::parse(
            Rule::document,
            r#"version 1.1
            task hello { 
                runtime {
                    container: "ubuntu:latest"
                    disks: "10 GB"
                }
            }
            task hello2 {
                runtime {
                    container: "ubuntu:20.04"
                }
            }
            task hello3 {
                input {
                    String name = "hello"
                }
                runtime {
                    container: "test ~{name}"
                }
            }"#,
        )?
        .next()
        .unwrap();

        let warnings = LiteralContainerUnique.check(&tree)?.unwrap();

        assert_eq!(warnings.len(), 1);
        assert_eq!(
            warnings.first().to_string(),
            "[v1::W008::Container/Medium] duplicate literal container (10:32-10:46)"
        );
        Ok(())
    }

    #[test]
    fn it_does_not_catch_unique_literal_container() -> Result<(), Box<dyn std::error::Error>> {
        let tree = Parser::parse(
            Rule::document,
            r#"version 1.1
            task hello { 
                runtime {
                    container: "ubuntu:20.04"
                }
            }
            task hello2 {
                runtime {
                    container: "ubuntu:20.04"
                }
            }"#,
        )?
        .next()
        .unwrap();

        let warnings = LiteralContainerUnique.check(&tree)?;

        assert!(warnings.is_none());
        Ok(())
    }
}
