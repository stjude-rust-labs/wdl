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
    // The key is the contents of the literal string node and if present, the alias
    // name. The value is the formatted import statement with any found
    // comments.
    let mut import_map: HashMap<String, String> = HashMap::new();
    for import in imports {
        let mut key = String::new();
        let mut val = String::new();
        // Process alias clauses separately
        // to ensure they are written at the end of the import statement
        let mut alias_clauses = String::new();

        for child in import.syntax().children_with_tokens() {
            match child.kind() {
                SyntaxKind::ImportKeyword => {
                    // This should always be the first child processed
                    val.push_str(&format_preceeding_comments(
                        &SyntaxElement::Node(import.syntax().clone()),
                        0,
                        false,
                    ));
                    val.push_str("import");
                    val.push_str(&format_inline_comment(&child, INDENT, ""));
                    let mut next = child.next_sibling_or_token();
                    while let Some(cur) = next {
                        match cur.kind() {
                            SyntaxKind::LiteralStringNode => {
                                val.push_str(&format_preceeding_comments(&cur, 1, true));
                                if !val.ends_with(INDENT) {
                                    val.push(' ');
                                }
                                cur.as_node().unwrap().children_with_tokens().for_each(
                                    |string_part| match string_part.kind() {
                                        SyntaxKind::DoubleQuote | SyntaxKind::SingleQuote => {
                                            val.push('"');
                                        }
                                        SyntaxKind::LiteralStringText => {
                                            key.push_str(&string_part.to_string());
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
                                val.push_str(&format_inline_comment(&cur, "", NEWLINE));
                            }
                            SyntaxKind::AsKeyword => {
                                val.push_str(&format_preceeding_comments(&cur, 1, false));
                                val.push_str(INDENT);
                                val.push_str("as");
                                val.push_str(&format_inline_comment(&cur, INDENT, ""));
                            }
                            SyntaxKind::Ident => {
                                key.push_str(&cur.to_string());

                                val.push_str(&format_preceeding_comments(&cur, 1, true));
                                if !val.ends_with(INDENT) {
                                    val.push(' ');
                                }
                                val.push_str(&cur.to_string());
                                val.push_str(&format_inline_comment(&cur, "", NEWLINE));
                            }
                            SyntaxKind::ImportAliasNode => {
                                let mut second_ident_of_clause = false;
                                cur.as_node().unwrap().children_with_tokens().for_each(
                                    |alias_part| match alias_part.kind() {
                                        SyntaxKind::AliasKeyword => {
                                            // This should always be the first child processed
                                            alias_clauses.push_str(&format_preceeding_comments(
                                                &cur, // Parent node
                                                1, false,
                                            ));
                                            alias_clauses.push_str(INDENT);
                                            alias_clauses.push_str("alias");
                                            alias_clauses.push_str(&format_inline_comment(
                                                &alias_part,
                                                INDENT,
                                                "",
                                            ));
                                        }
                                        SyntaxKind::Ident => {
                                            alias_clauses.push_str(&format_preceeding_comments(
                                                &alias_part,
                                                1,
                                                true,
                                            ));
                                            if !alias_clauses.ends_with(INDENT) {
                                                alias_clauses.push(' ');
                                            }
                                            alias_clauses.push_str(&alias_part.to_string());
                                            if !second_ident_of_clause {
                                                alias_clauses.push_str(&format_inline_comment(
                                                    &alias_part,
                                                    INDENT,
                                                    "",
                                                ));
                                                second_ident_of_clause = true;
                                            } else {
                                                alias_clauses.push_str(&format_inline_comment(
                                                    &SyntaxElement::Node(import.syntax().clone()), // Parent's parent node
                                                    "", NEWLINE,
                                                ));
                                            }
                                        }
                                        SyntaxKind::AsKeyword => {
                                            alias_clauses.push_str(&format_preceeding_comments(
                                                &alias_part,
                                                1,
                                                false,
                                            ));
                                            if !alias_clauses.ends_with(INDENT) {
                                                alias_clauses.push(' ');
                                            }
                                            alias_clauses.push_str("as");
                                            alias_clauses.push_str(&format_inline_comment(
                                                &alias_part,
                                                INDENT,
                                                "",
                                            ));
                                        }
                                        SyntaxKind::ImportAliasNode => {
                                            // Ignore the root node
                                        }
                                        SyntaxKind::Whitespace => {
                                            // Ignore
                                        }
                                        SyntaxKind::Comment => {
                                            // This comment will be included by
                                            // a call to '
                                            // format_preceeding_comments' or
                                            // 'format_inline_comment'
                                            // in another match arm
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
                                // This comment will be included by a call to
                                // 'format_inline_comment'
                                // in another match arm
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

        val.push_str(&alias_clauses);

        // val.push_str(&format_inline_comment(
        //     &SyntaxElement::Node(import.syntax().clone()),
        //     "",
        //     NEWLINE,
        // ));

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
