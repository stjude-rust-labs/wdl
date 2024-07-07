/// This module contains the functions for formatting import statements.
use std::collections::HashMap;

use wdl_ast::v1::ImportStatement;
use wdl_ast::AstChildren;
use wdl_ast::AstNode;
use wdl_ast::SyntaxElement;
use wdl_ast::SyntaxKind;

use super::comments::format_inline_comment;
use super::comments::format_preceeding_comments;
use super::INDENT;
use super::NEWLINE;

/// Format a list of import statements.
pub fn format_imports(imports: AstChildren<ImportStatement>) -> String {
    // Collect the imports into a map so we can sort them
    // The key is the "body" of the import statement (which we will sort on)
    // and the value is the formatted import statement _with any found comments_.
    let mut import_map: HashMap<String, String> = HashMap::new();
    for import in imports {
        // TODO: should 'key' get formatted before sorting?
        let key = import.syntax().to_string();
        let mut val = String::new();

        val.push_str(&format_preceeding_comments(
            &SyntaxElement::Node(import.syntax().clone()),
            0,
        ));

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
                                            val.push_str(&string_part.to_string());
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
                                val.push_str(&cur.to_string());
                            }
                            SyntaxKind::ImportAliasNode => {
                                cur.as_node().unwrap().children_with_tokens().for_each(
                                    |alias_part| match alias_part.kind() {
                                        SyntaxKind::AliasKeyword => {
                                            // This should always be the first child processed
                                            val.push_str(" alias ");
                                        }
                                        SyntaxKind::Ident => {
                                            val.push_str(&alias_part.to_string());
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
                                            val.push_str(alias_part.to_string().trim());
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
                                val.push_str(cur.to_string().trim());
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

        val.push_str(&format_inline_comment(
            &SyntaxElement::Node(import.syntax().clone()),
            "",
            NEWLINE,
        ));

        import_map.insert(key, val);
    }

    let mut import_vec: Vec<_> = import_map.into_iter().collect();
    import_vec.sort_by(|a, b| a.0.cmp(&b.0));

    let mut result = String::new();
    for (_, val) in import_vec {
        result.push_str(&val);
    }
    if !result.is_empty() {
        // There should always be a blank line after the imports
        // (if they are present), so add a second newline here.
        result.push_str(NEWLINE);
    }
    result
}
