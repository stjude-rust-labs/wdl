//! A module for formatting WDL code.

use std::collections::HashMap;

use anyhow::bail;
use anyhow::Result;
use wdl_ast::v1::ImportStatement;
use wdl_ast::AstChildren;
use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::Direction;
use wdl_ast::Document;
use wdl_ast::SyntaxKind;
use wdl_ast::Validator;
use wdl_ast::VersionStatement;

const NEWLINE: &str = "\n";
const INDENT: &str = "    ";

/// Format a version statement.
fn format_version_statement(version_statement: VersionStatement) -> String {
    let mut result = String::new();
    for sibling in version_statement
        .syntax()
        .siblings_with_tokens(Direction::Prev)
    {
        match sibling.kind() {
            SyntaxKind::Comment => {
                result.push_str(sibling.as_token().unwrap().text().trim());
                result.push_str(NEWLINE);
            }
            SyntaxKind::Whitespace => {
                // Skip whitespace
            }
            SyntaxKind::VersionStatementNode => {
                // Ignore the root node
            }
            _ => {
                unreachable!("Unexpected syntax kind: {:?}", sibling.kind());
            }
        }
    }

    for child in version_statement.syntax().descendants_with_tokens() {
        match child.kind() {
            SyntaxKind::Comment => {
                result.push_str(child.as_token().unwrap().text().trim());
                result.push_str(NEWLINE);
            }
            SyntaxKind::Whitespace => {
                // Skip whitespace
            }
            SyntaxKind::VersionKeyword => {
                if result.is_empty() {
                    result.push_str("version ");
                } else {
                    result.push_str("\nversion ");
                }
                result.push_str(version_statement.version().as_str());
                result.push_str(NEWLINE);
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
    result.push_str(NEWLINE);
    result
}

fn format_imports(imports: AstChildren<ImportStatement>) -> String {
    let mut import_map: HashMap<String, String> = HashMap::new();
    for import in imports {
        let key = import.syntax().to_string();
        let mut val = String::new();
        let mut preceeding_comments = Vec::new();
        let mut processed_root = false;
        // Collect any comments before the import statement
        for sibling in import.syntax().siblings_with_tokens(Direction::Prev) {
            match sibling.kind() {
                SyntaxKind::Comment => {
                    // Ensure this comment "belongs" to the import statement.
                    // A preceeding comment on a blank line is considered to belong to the import
                    // statement. Othewise, the comment "belongs" to whatever
                    // else is on that line.
                    let mut prev = sibling.prev_sibling_or_token();
                    while let Some(cur) = prev {
                        match cur.kind() {
                            SyntaxKind::Whitespace => {
                                if cur.as_token().unwrap().text().contains('\n') {
                                    // The 'sibling' comment is on is on its own line.
                                    // It "belongs" to the current import statement.
                                    preceeding_comments.push(
                                        sibling.as_token().unwrap().text().trim().to_string(),
                                    );
                                    break;
                                }
                            }
                            _ => {
                                // The 'sibling' comment is on the same line as this token.
                                // It "belongs" to whatever is currently being processed.
                                break;
                            }
                        }
                        prev = cur.next_sibling_or_token();
                    }
                }
                SyntaxKind::Whitespace => {
                    // Skip whitespace
                }
                SyntaxKind::ImportStatementNode => {
                    if processed_root {
                        // This must be a previous import statement
                        break;
                    }
                    processed_root = true;
                }
                _ => {
                    // We've backed up past any trivia, so we can stop
                    break;
                }
            }
        }

        for comment in preceeding_comments.iter().rev() {
            val.push_str(comment);
            val.push_str(NEWLINE);
        }

        // Collect the import statement
        for child in import.syntax().children_with_tokens() {
            match child.kind() {
                SyntaxKind::ImportKeyword => {
                    // This should always be the first child processed
                    val.push_str("import ");
                    let mut next = child.next_sibling_or_token();
                    while let Some(cur) = next {
                        match cur.kind() {
                            SyntaxKind::LiteralStringNode => {
                                cur.as_node().unwrap().children_with_tokens().for_each(
                                    |string_part| match string_part.kind() {
                                        SyntaxKind::DoubleQuote | SyntaxKind::SingleQuote => {
                                            val.push('"');
                                        }
                                        SyntaxKind::LiteralStringText => {
                                            val.push_str(string_part.as_token().unwrap().text());
                                        }
                                        _ => {
                                            unreachable!(
                                                "Unexpected syntax kind: {:?}",
                                                child.kind()
                                            );
                                        }
                                    },
                                );
                            }
                            SyntaxKind::AsKeyword => {
                                val.push_str(" as ");
                            }
                            SyntaxKind::Ident => {
                                val.push_str(cur.as_token().unwrap().text());
                            }
                            SyntaxKind::ImportAliasNode => {
                                cur.as_node().unwrap().children_with_tokens().for_each(
                                    |alias_part| match alias_part.kind() {
                                        SyntaxKind::AliasKeyword => {
                                            // This should always be the first child processed
                                            val.push_str(" alias ");
                                        }
                                        SyntaxKind::Ident => {
                                            val.push_str(alias_part.as_token().unwrap().text());
                                        }
                                        SyntaxKind::AsKeyword => {
                                            val.push_str(" as ");
                                        }
                                        SyntaxKind::ImportAliasNode => {
                                            // Ignore the root node
                                        }
                                        SyntaxKind::Whitespace => {
                                            // Ignore
                                        }
                                        SyntaxKind::Comment => {
                                            // This comment will cause a lint warning
                                            // But we'll include it anyway
                                            if !val.ends_with(" ") {
                                                val.push(' ');
                                            }
                                            val.push(' ');
                                            val.push_str(
                                                alias_part.as_token().unwrap().text().trim(),
                                            );
                                            val.push_str(NEWLINE);
                                            val.push_str(INDENT);
                                        }
                                        _ => {
                                            unreachable!(
                                                "Unexpected syntax kind: {:?}",
                                                alias_part.kind()
                                            );
                                        }
                                    },
                                );
                            }
                            SyntaxKind::Whitespace => {
                                // Ignore
                            }
                            SyntaxKind::Comment => {
                                // This comment will cause a lint warning
                                // But we'll include it anyway
                                if !val.ends_with(" ") {
                                    val.push(' ');
                                }
                                val.push(' ');
                                val.push_str(cur.as_token().unwrap().text().trim());
                                val.push_str(NEWLINE);
                                val.push_str(INDENT);
                            }
                            _ => {
                                unreachable!("Unexpected syntax kind: {:?}", cur.kind());
                            }
                        }
                        next = cur.next_sibling_or_token();
                    }
                }
                SyntaxKind::Whitespace => {
                    // Ignore
                }
                SyntaxKind::ImportStatementNode => {
                    // Ignore the root node
                }
                SyntaxKind::LiteralStringNode
                | SyntaxKind::Comment
                | SyntaxKind::AsKeyword
                | SyntaxKind::Ident
                | SyntaxKind::ImportAliasNode => {
                    // Handled by the import keyword
                }
                _ => {
                    unreachable!("Unexpected syntax kind: {:?}", child.kind());
                }
            }
        }

        // Check for comments _immediately_ after the import statement
        // (i.e., on the same line)
        let mut next = import.syntax().next_sibling_or_token();
        while let Some(cur) = next {
            match cur.kind() {
                SyntaxKind::Comment => {
                    val.push_str("  ");
                    val.push_str(cur.as_token().unwrap().text().trim());
                }
                SyntaxKind::Whitespace => {
                    // Ignore
                }
                _ => {
                    // We've backed up past any trivia, so we can stop
                    break;
                }
            }
            next = cur.next_sibling_or_token();
        }

        val.push_str(NEWLINE);

        import_map.insert(key, val);
    }

    let mut import_vec: Vec<_> = import_map.into_iter().collect();
    import_vec.sort_by(|a, b| a.0.cmp(&b.0));

    let mut result = String::new();
    for (_, val) in import_vec {
        result.push_str(&val);
    }
    result.push_str(NEWLINE);
    result
}

/// Format a WDL document.
pub fn format_document(code: &str) -> Result<String> {
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
    result.push_str(&format_version_statement(
        document.version_statement().unwrap(),
    ));

    let ast = document.ast();
    let ast = ast.as_v1().unwrap();
    result.push_str(&format_imports(ast.imports()));

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_with_comments() {
        let code = "\n\n    ## preamble comment  \nversion # weird comment\n 1.1\nworkflow test {}";
        let formatted = format_document(code).unwrap();
        assert_eq!(
            formatted,
            "## preamble comment\n\nversion 1.1\n# weird comment\n\n"
        );
    }

    #[test]
    fn test_format_without_comments() {
        let code = "version 1.1\nworkflow test {}";
        let formatted = format_document(code).unwrap();
        assert_eq!(formatted, "version 1.1\n\n");
    }

    #[test]
    fn test_format_with_imports() {
        let code = "
        version 1.1

        # this comment belongs to fileB
        import \"fileB.wdl\" as foo # also fileB
        import \"fileA.wdl\" as bar # after fileA
            alias qux as Qux
        workflow test {}
        # this comment belongs to fileC
        import \"fileC.wdl\"";
        let formatted = format_document(code).unwrap();
        assert_eq!(
            formatted,
            "version 1.1\n\nimport \"fileA.wdl\" as bar  # after fileA\n     alias qux as Qux\n# \
             this comment belongs to fileB\nimport \"fileB.wdl\" as foo  # also fileB\n# this \
             comment belongs to fileC\nimport \"fileC.wdl\"\n\n"
        );
    }
}
