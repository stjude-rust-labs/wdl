/// This module contains the functions for formatting import statements.
use std::collections::HashMap;

use wdl_ast::v1::ImportStatement;
use wdl_ast::AstChildren;
use wdl_ast::AstNode;
use wdl_ast::SyntaxElement;
use wdl_ast::SyntaxKind;

use super::comments::format_inline_comment;
use super::comments::format_preceding_comments;
use super::INDENT;
use super::NEWLINE;

/// Format a list of import statements.
pub fn format_imports(imports: AstChildren<ImportStatement>) -> String {
    // Collect the imports into a map so we can sort them
    // The key is the contents of the literal string node and if present, the alias
    // name. The value is the formatted import statement with any found
    // comments.
    let mut import_map: HashMap<String, String> = HashMap::new();
    let one_indent = INDENT;
    let two_indents = INDENT.repeat(2);
    for import in imports {
        let mut key = String::new();
        let mut val = String::new();

        val.push_str(&format_preceding_comments(
            &SyntaxElement::Node(import.syntax().clone()),
            0,
            false,
        ));

        val.push_str("import");
        let import_keyword = import.syntax().first_token().unwrap();
        val.push_str(&format_inline_comment(
            &SyntaxElement::Token(import_keyword.clone()),
            false,
        ));
        let mut next = import_keyword.next_sibling_or_token();
        while let Some(cur) = next {
            match cur.kind() {
                SyntaxKind::LiteralStringNode => {
                    val.push_str(&format_preceding_comments(&cur, 1, !val.ends_with(NEWLINE)));
                    if val.ends_with("import") {
                        val.push(' ');
                    } else if val.ends_with(NEWLINE) {
                        val.push_str(one_indent);
                    }
                    cur.as_node()
                        .unwrap()
                        .children_with_tokens()
                        .for_each(|string_part| match string_part.kind() {
                            SyntaxKind::DoubleQuote | SyntaxKind::SingleQuote => {
                                val.push('"');
                            }
                            SyntaxKind::LiteralStringText => {
                                key.push_str(&string_part.to_string());
                                val.push_str(&string_part.to_string());
                            }
                            _ => {
                                unreachable!("Unexpected syntax kind: {:?}", cur.kind());
                            }
                        });
                    val.push_str(&format_inline_comment(&cur, false));
                }
                SyntaxKind::AsKeyword => {
                    if !val.ends_with(NEWLINE) {
                        val.push_str(NEWLINE);
                    }
                    val.push_str(&format_preceding_comments(&cur, 1, false));
                    val.push_str(one_indent);
                    val.push_str("as");
                    val.push_str(&format_inline_comment(&cur, false));
                }
                SyntaxKind::Ident => {
                    key.push_str(&cur.to_string());

                    val.push_str(&format_preceding_comments(&cur, 2, !val.ends_with(NEWLINE)));
                    if val.ends_with("as") {
                        val.push(' ');
                    } else {
                        val.push_str(&two_indents);
                    }
                    val.push_str(&cur.to_string());
                    val.push_str(&format_inline_comment(&cur, false));
                }
                SyntaxKind::ImportAliasNode => {
                    if !val.ends_with(NEWLINE) {
                        val.push_str(NEWLINE);
                    }
                    val.push_str(&format_preceding_comments(&cur, 1, false));
                    let mut second_ident_of_clause = false;
                    cur.as_node()
                        .unwrap()
                        .children_with_tokens()
                        .for_each(|alias_part| match alias_part.kind() {
                            SyntaxKind::AliasKeyword => {
                                // This should always be the first child processed

                                val.push_str(one_indent);
                                val.push_str("alias");
                                val.push_str(&format_inline_comment(&alias_part, false));
                            }
                            SyntaxKind::Ident => {
                                val.push_str(&format_preceding_comments(
                                    &alias_part,
                                    2,
                                    !val.ends_with(NEWLINE),
                                ));
                                if val.ends_with("alias") || val.ends_with("as") {
                                    val.push(' ');
                                } else {
                                    val.push_str(&two_indents);
                                }
                                val.push_str(&alias_part.to_string());
                                if !second_ident_of_clause {
                                    val.push_str(&format_inline_comment(&alias_part, false));
                                    second_ident_of_clause = true;
                                } // else will be handled by outer loop
                            }
                            SyntaxKind::AsKeyword => {
                                val.push_str(&format_preceding_comments(
                                    &alias_part,
                                    2,
                                    !val.ends_with(NEWLINE),
                                ));
                                if val.ends_with(NEWLINE) {
                                    val.push_str(&two_indents);
                                } else {
                                    val.push(' ');
                                }
                                val.push_str("as");
                                val.push_str(&format_inline_comment(&alias_part, false));
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
                                // format_preceding_comments' or
                                // 'format_inline_comment'
                                // in another match arm
                            }
                            _ => {
                                unreachable!("Unexpected syntax kind: {:?}", alias_part.kind());
                            }
                        });
                }
                SyntaxKind::Whitespace => {
                    // Ignore
                }
                SyntaxKind::Comment => {
                    // This comment will be included by a call to
                    // 'format_inline_comment' or 'format_preceding_comments'
                    // in another match arm
                }
                _ => {
                    unreachable!("Unexpected syntax kind: {:?}", cur.kind());
                }
            }
            next = cur.next_sibling_or_token();
        }

        let newline_needed = !val.ends_with(NEWLINE);
        val.push_str(&format_inline_comment(
            &SyntaxElement::Node(import.syntax().clone()),
            newline_needed,
        ));

        import_map.insert(key, val);
    }

    let mut import_vec: Vec<_> = import_map.into_iter().collect();
    import_vec.sort_by(|a, b| a.0.cmp(&b.0));

    let mut result = String::new();
    for (_, val) in import_vec {
        result.push_str(&val);
    }

    result
}
