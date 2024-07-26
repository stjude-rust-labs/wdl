//! Format import statements.

use std::fmt::Write;

use anyhow::Result;
use wdl_ast::v1::ImportStatement;
use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::Ident;
use wdl_ast::SyntaxElement;
use wdl_ast::SyntaxKind;

use super::comments::format_inline_comment;
use super::comments::format_preceding_comments;
use super::Formattable;
use super::State;

impl Formattable for ImportStatement {
    fn format(&self, buffer: &mut String, state: &mut State) -> Result<()> {
        format_preceding_comments(
            &SyntaxElement::from(self.syntax().clone()),
            buffer,
            state,
            false,
        )?;

        let import_keyword = self
            .syntax()
            .children_with_tokens()
            .find(|element| element.kind() == SyntaxKind::ImportKeyword)
            .expect("Import statement should have an import keyword");
        buffer.push_str(&import_keyword.to_string());
        format_inline_comment(&import_keyword, buffer, state, true)?;

        let uri = self.uri();
        format_preceding_comments(
            &SyntaxElement::from(uri.syntax().clone()),
            buffer,
            state,
            true,
        )?;
        state.space_or_indent(buffer)?;
        uri.format(buffer, state)?;
        format_inline_comment(
            &SyntaxElement::from(uri.syntax().clone()),
            buffer,
            state,
            true,
        )?;

        let mut next = uri.syntax().next_sibling_or_token();
        while let Some(cur) = next {
            match cur.kind() {
                SyntaxKind::AsKeyword => {
                    format_preceding_comments(&cur, buffer, state, true)?;
                    state.space_or_indent(buffer)?;
                    buffer.push_str(&cur.to_string());
                    state.reset_interrupted();
                    format_inline_comment(&cur, buffer, state, true)?;
                }
                SyntaxKind::Ident => {
                    format_preceding_comments(&cur, buffer, state, true)?;
                    state.space_or_indent(buffer)?;
                    let ident =
                        Ident::cast(cur.as_token().expect("Ident should be a token").clone())
                            .expect("Ident should cast to an ident");
                    ident.format(buffer, state)?;
                    format_inline_comment(&cur, buffer, state, true)?;
                }
                SyntaxKind::ImportAliasNode => {
                    format_preceding_comments(&cur, buffer, state, true)?;
                    let mut second_ident_of_clause = false;
                    for alias_part in cur
                        .as_node()
                        .expect("Import alias should be a node")
                        .children_with_tokens()
                    {
                        match alias_part.kind() {
                            SyntaxKind::AliasKeyword => {
                                // Should always be first 'alias_part' processed
                                // so preceding comments were handled above.
                                state.space_or_indent(buffer)?;
                                buffer.push_str(&alias_part.to_string());
                                format_inline_comment(&alias_part, buffer, state, true)?;
                            }
                            SyntaxKind::Ident => {
                                format_preceding_comments(&alias_part, buffer, state, true)?;
                                state.space_or_indent(buffer)?;
                                write!(buffer, "{}", alias_part)?;
                                if !second_ident_of_clause {
                                    format_inline_comment(&alias_part, buffer, state, true)?;
                                    second_ident_of_clause = true;
                                } // else an inline comment will be handled by outer loop
                            }
                            SyntaxKind::AsKeyword => {
                                format_preceding_comments(&alias_part, buffer, state, true)?;
                                state.space_or_indent(buffer)?;
                                buffer.push_str(&alias_part.to_string());
                                format_inline_comment(&alias_part, buffer, state, true)?;
                            }
                            SyntaxKind::ImportAliasNode => {
                                // Ignore the root node
                            }
                            SyntaxKind::Whitespace => {
                                // Ignore
                            }
                            SyntaxKind::Comment => {
                                // This comment will be included by a call to
                                // 'format_inline_comment' or
                                // 'format_preceding_comments'
                                // in another match arm
                            }
                            _ => {
                                unreachable!("Unexpected syntax kind: {:?}", alias_part.kind());
                            }
                        }
                    }
                    format_inline_comment(&cur, buffer, state, true)?;
                }
                SyntaxKind::Comment => {
                    // This comment will be included by a call to
                    // 'format_inline_comment' or 'format_preceding_comments'
                    // in another match arm
                }
                SyntaxKind::Whitespace => {
                    // Ignore
                }
                _ => {
                    unreachable!("Unexpected syntax kind: {:?}", cur.kind());
                }
            }
            next = cur.next_sibling_or_token();
        }
        format_inline_comment(
            &SyntaxElement::from(self.syntax().clone()),
            buffer,
            state,
            false,
        )?;

        Ok(())
    }
}

/// Sorts import statements by their core components.
///
/// The core components of an import statement are the URI and the namespace.
/// These two elements guarentee a unique import statement.
pub fn sort_imports(a: &ImportStatement, b: &ImportStatement) -> std::cmp::Ordering {
    let a_core = format!(
        "{}{}",
        a.uri()
            .text()
            .expect("Import URI cannot have placeholders")
            .as_str(),
        a.namespace().expect("Import namespace should exist").0
    );
    let b_core = format!(
        "{}{}",
        b.uri()
            .text()
            .expect("Import URI cannot have placeholders")
            .as_str(),
        b.namespace().expect("Import namespace should exist").0
    );
    a_core.cmp(&b_core)
}
