//! A module for formatting WDL code.

use anyhow::bail;
use anyhow::Result;
use wdl_ast::AstNode;
use wdl_ast::AstToken;
// use wdl_ast::Comment;
use wdl_ast::Document;
// use wdl_ast::Span;
use wdl_ast::SyntaxKind;
use wdl_ast::Validator;

/// Format the given WDL code.
pub fn format_wdl_code(code: &str) -> Result<String> {
    let parse = Document::parse(code).into_result();
    if let Err(diagnostics) = parse {
        for diagnostic in diagnostics.into_iter() {
            eprintln!("{}", diagnostic.message());
        }
        bail!("The document is not valid, so it cannot be formatted.")
    }
    let document = parse.unwrap();
    let validator = Validator::default();
    match validator.validate(&document) {
        Ok(_) => {
            // The document is valid, so we can format it.
        }
        Err(diagnostics) => {
            for diagnostic in diagnostics.into_iter() {
                eprintln!("{}", diagnostic.message());
            }
            bail!("The document is not valid, so it cannot be formatted.")
        }
    }

    let mut result = String::new();
    let version_statement = document.version_statement().unwrap();
    for child in version_statement.syntax().descendants_with_tokens() {
        match child.kind() {
            SyntaxKind::Comment => {
                result.push_str(child.as_token().unwrap().text().trim());
            }
            SyntaxKind::Whitespace => {
                // Skip whitespace
            }
            SyntaxKind::VersionKeyword => {
                if result.is_empty() {
                    result.push_str("version ");
                } else {
                    result.push_str("\n\nversion ");
                }
                result.push_str(version_statement.version().as_str());
            }
            SyntaxKind::Version => {
                // Handled by the version keyword
            }
            SyntaxKind::VersionStatementNode => {
                // Ignore the root node
            }
            _ => {
                unreachable!("Unexpected syntax kind: {:?}", child.kind());
            }
        }
    }

    // let ast = document.ast().as_v1().unwrap();
    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_wdl_code() {
        let code = "\n\n    ## preamble comment  \nversion 1.1\nworkflow test {}";
        let formatted = format_wdl_code(code).unwrap();
        assert_eq!(formatted, "version 1.1");
    }
}
