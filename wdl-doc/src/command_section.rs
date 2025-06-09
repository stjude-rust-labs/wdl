//! An extension of the [`CommandSection`] type that adds
//! functionality for rendering the command section in a WDL document.

use wdl_ast::AstNode;
use wdl_ast::v1::CommandPart;
use wdl_ast::v1::CommandSection;
use wdl_ast::v1::StrippedCommandPart;

/// An extension trait for the [`CommandSection`] type that provides
/// functionality for rendering the command section as a script string.
pub trait CommandSectionExt {
    /// Returns the command section as a script string, which is a concatenation
    /// of all text parts and placeholders with common whitespace stripped.
    fn script(&self) -> String;
}

impl CommandSectionExt for CommandSection {
    fn script(&self) -> String {
        let common_whitespace = self.count_whitespace();
        match self.strip_whitespace() {
            Some(v) => v
                .into_iter()
                .map(|s| match s {
                    StrippedCommandPart::Text(text) => text,
                    StrippedCommandPart::Placeholder(placeholder) => {
                        let common_whitespace =
                            common_whitespace.expect("common whitespace should be present");
                        let placeholder = placeholder.text().to_string();
                        placeholder
                            .lines()
                            .map(|line| {
                                if line.starts_with(&" ".repeat(common_whitespace))
                                    || line.starts_with(&"\t".repeat(common_whitespace))
                                {
                                    line[common_whitespace..].to_string()
                                } else {
                                    line.to_string()
                                }
                            })
                            .collect::<Vec<_>>()
                            .join("\n")
                    }
                })
                .collect::<Vec<_>>()
                .join(""),
            None => self
                .parts()
                .map(|p| match p {
                    CommandPart::Text(text) => {
                        let mut buffer = String::new();
                        text.unescape_to(self.is_heredoc(), &mut buffer);
                        buffer
                    }
                    CommandPart::Placeholder(placehoder) => placehoder.text().to_string(),
                })
                .collect::<Vec<_>>()
                .join(""),
        }
    }
}
