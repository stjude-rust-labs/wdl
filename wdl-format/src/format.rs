//! A module for formatting WDL code.

use std::fmt::Write;

use anyhow::Error;
use anyhow::Ok;
use anyhow::Result;
use wdl_ast::v1::Decl;
use wdl_ast::v1::DocumentItem;
use wdl_ast::v1::InputSection;
use wdl_ast::v1::MetadataObjectItem;
use wdl_ast::v1::MetadataSection;
use wdl_ast::v1::OutputSection;
use wdl_ast::v1::ParameterMetadataSection;
use wdl_ast::v1::StructDefinition;
use wdl_ast::version;
use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::Comment;
use wdl_ast::Diagnostic;
use wdl_ast::Direction;
use wdl_ast::Document;
use wdl_ast::SyntaxElement;
use wdl_ast::SyntaxKind;
use wdl_ast::Validator;
use wdl_ast::Version;
use wdl_ast::VersionStatement;

mod comments;
// mod import;
// mod task;
// mod workflow;

use comments::format_inline_comment;
use comments::format_preceding_comments;
// use import::format_imports;
// use task::format_task;
// use workflow::format_workflow;

/// Newline constant used for formatting.
pub const NEWLINE: &str = "\n";
/// Space constant used for formatting.
pub const SPACE: &str = " ";
/// Indentation constant used for formatting.
pub const INDENT: &str = "    ";

struct FormatState {
    indent_level: usize,
}

impl Default for FormatState {
    fn default() -> Self {
        FormatState { indent_level: 0 }
    }
}

impl FormatState {
    fn indent(&self, buffer: &mut String) -> Result<(), Error> {
        let indent = INDENT.repeat(self.indent_level);
        write!(buffer, "{}", indent)?;
        Ok(())
    }

    fn indent_extra(&self, buffer: &mut String) -> Result<(), Error> {
        let indent = INDENT.repeat(self.indent_level + 1);
        write!(buffer, "{}", indent)?;
        Ok(())
    }

    fn increment_indent(&mut self) {
        self.indent_level += 1;
    }

    fn decrement_indent(&mut self) {
        self.indent_level = self.indent_level.saturating_sub(1);
    }
}

trait Formattable {
    fn format(&self, buffer: &mut String, state: &mut FormatState) -> Result<(), Error>;
    fn syntax_element(&self) -> SyntaxElement;
}

impl Formattable for Comment {
    fn format(&self, buffer: &mut String, _state: &mut FormatState) -> Result<(), Error> {
        let comment = self.as_str().trim();
        write!(buffer, "{}{}", comment, NEWLINE)?;
        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Token(self.syntax().clone())
    }
}

impl Formattable for Version {
    fn format(&self, buffer: &mut String, _state: &mut FormatState) -> Result<(), Error> {
        write!(buffer, "{}", self.as_str())?;
        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Token(self.syntax().clone())
    }
}

impl Formattable for VersionStatement {
    fn format(&self, buffer: &mut String, state: &mut FormatState) -> Result<(), Error> {
        let mut preceding_comments = Vec::new();
        let comment_buffer = &mut String::new();
        for sibling in self.syntax().siblings_with_tokens(Direction::Prev) {
            match sibling.kind() {
                SyntaxKind::Comment => {
                    let comment = Comment::cast(
                        sibling
                            .as_token()
                            .expect("Comment should be a token")
                            .clone(),
                    )
                    .expect("Comment should cast to a comment");
                    comment.format(comment_buffer, state)?;
                    preceding_comments.push(comment_buffer.clone());
                    comment_buffer.clear();
                }
                SyntaxKind::Whitespace => {
                    // Ignore
                }
                SyntaxKind::VersionStatementNode => {
                    // Ignore the root node
                }
                _ => {
                    unreachable!("Unexpected syntax kind: {:?}", sibling.kind());
                }
            }
        }

        for comment in preceding_comments.iter().rev() {
            buffer.push_str(comment);
        }

        // If there are preamble comments, ensure a blank line is inserted
        if !preceding_comments.is_empty() {
            buffer.push_str(NEWLINE);
        }

        buffer.push_str("version");
        let version_keyword = SyntaxElement::Token(
            self.syntax()
                .first_token()
                .expect("Version Statement should have a token")
                .clone(),
        );

        let version = self.version();
        if format_inline_comment(&version_keyword, buffer, state)?
            || format_preceding_comments(&version.syntax_element(), buffer, state)?
        {
            state.indent_extra(buffer)?;
        } else {
            write!(buffer, "{}", SPACE)?;
        }

        version.format(buffer, state)?;
        if !format_inline_comment(&self.syntax_element(), buffer, state)? {
            buffer.push_str(NEWLINE);
        }

        Ok(())
    }

    fn syntax_element(&self) -> SyntaxElement {
        SyntaxElement::Node(self.syntax().clone())
    }
}

// /// Format a version statement.
// fn format_version_statement(version_statement: VersionStatement) -> String {
//     // Collect comments that preceed the version statement.
//     // Note as this must be the first element in the document,
//     // the logic is slightly different than the 'format_preceding_comments'
//     // function. We are walking backwards through the syntax tree, so we must
//     // collect the comments in a vector and reverse them to get them in the
//     // correct order.
//     let mut preceding_comments = Vec::new();
//     for sibling in version_statement
//         .syntax()
//         .siblings_with_tokens(Direction::Prev)
//     {
//         match sibling.kind() {
//             SyntaxKind::Comment => {
//
// preceding_comments.push(sibling.to_string().trim().to_owned());             }
//             SyntaxKind::Whitespace => {
//                 // Ignore
//             }
//             SyntaxKind::VersionStatementNode => {
//                 // Ignore the root node
//             }
//             _ => {
//                 unreachable!("Unexpected syntax kind: {:?}", sibling.kind());
//             }
//         }
//     }

//     let mut result = String::new();
//     for comment in preceding_comments.iter().rev() {
//         result.push_str(comment);
//         result.push_str(NEWLINE);
//     }

//     if !result.is_empty() {
//         // If there are preamble comments, ensure a blank line is inserted
//         result.push_str(NEWLINE);
//     }
//     result.push_str("version");
//     let version_keyword = version_statement.syntax().first_token().unwrap();
//     result.push_str(&format_inline_comment(
//         &SyntaxElement::Token(version_keyword),
//         false,
//     ));

//     // result.push_str(&format_preceding_comments(
//     //
// &SyntaxElement::Token(version_statement.version().syntax().clone()),     //
// 1,     //     !result.ends_with(NEWLINE),
//     // ));
//     if result.ends_with("version") {
//         result.push(' ');
//     } else if result.ends_with(NEWLINE) {
//         result.push_str(INDENT);
//     }
//     result.push_str(version_statement.version().as_str());
//     result.push_str(&format_inline_comment(
//         &SyntaxElement::Node(version_statement.syntax().clone()),
//         true,
//     ));

//     result
// }

// /// Format the inner portion of a meta/parameter_meta section.
// fn format_metadata_item(item: &MetadataObjectItem) -> String {
//     let mut result = String::new();
//     let two_indents = INDENT.repeat(2);
//     let three_indents = INDENT.repeat(3);

//     result.push_str(&format_preceding_comments(
//         &SyntaxElement::Node(item.syntax().clone()),
//         2,
//         false,
//     ));
//     result.push_str(&two_indents);
//     result.push_str(item.name().as_str());
//     result.push_str(&format_inline_comment(
//         &SyntaxElement::Token(item.name().syntax().clone()),
//         false,
//     ));

//     let colon = item
//         .syntax()
//         .children_with_tokens()
//         .find(|c| c.kind() == SyntaxKind::Colon)
//         .expect("metadata item should have a colon");
//     result.push_str(&format_preceding_comments(
//         &colon,
//         1,
//         !result.ends_with(NEWLINE),
//     ));
//     if result.ends_with(NEWLINE) {
//         result.push_str(&three_indents);
//     }
//     result.push(':');
//     result.push_str(&format_inline_comment(&colon, false));

//     result.push_str(&format_preceding_comments(
//         &SyntaxElement::Node(item.value().syntax().clone()),
//         1,
//         !result.ends_with(NEWLINE),
//     ));
//     if result.ends_with(NEWLINE) {
//         result.push_str(&three_indents);
//     } else {
//         result.push(' ');
//     }
//     result.push_str(&item.value().syntax().to_string());
//     result.push_str(&format_inline_comment(
//         &SyntaxElement::Node(item.syntax().clone()),
//         true,
//     ));

//     result
// }

// /// Format a meta section.
// fn format_meta_section(meta: MetadataSection) -> String {
//     let mut result = String::new();

//     result.push_str(&format_preceding_comments(
//         &SyntaxElement::Node(meta.syntax().clone()),
//         1,
//         false,
//     ));

//     result.push_str(INDENT);
//     result.push_str("meta");
//     let meta_keyword = meta.syntax().first_token().unwrap();
//     result.push_str(&format_inline_comment(
//         &SyntaxElement::Token(meta_keyword.clone()),
//         false,
//     ));

//     let open_brace = meta
//         .syntax()
//         .children_with_tokens()
//         .find(|c| c.kind() == SyntaxKind::OpenBrace)
//         .expect("metadata section should have an open brace");
//     result.push_str(&format_preceding_comments(
//         &open_brace,
//         1,
//         !result.ends_with(NEWLINE),
//     ));
//     if result.ends_with(NEWLINE) {
//         result.push_str(INDENT);
//     } else {
//         result.push(' ');
//     }
//     result.push('{');
//     result.push_str(&format_inline_comment(&open_brace, true));

//     for item in meta.items() {
//         result.push_str(&format_metadata_item(&item));
//     }

//     let close_brace = meta
//         .syntax()
//         .children_with_tokens()
//         .find(|c| c.kind() == SyntaxKind::CloseBrace)
//         .expect("metadata section should have a close brace");
//     result.push_str(&format_preceding_comments(&close_brace, 0, false));
//     result.push_str(INDENT);
//     result.push('}');

//     result.push_str(&format_inline_comment(
//         &SyntaxElement::Node(meta.syntax().clone()),
//         true,
//     ));

//     result
// }

// /// Format a parameter meta section.
// fn format_parameter_meta_section(parameter_meta: ParameterMetadataSection) ->
// String {     let mut result = String::new();

//     result.push_str(&format_preceding_comments(
//         &SyntaxElement::Node(parameter_meta.syntax().clone()),
//         1,
//         false,
//     ));

//     result.push_str(INDENT);
//     result.push_str("parameter_meta");
//     let parameter_meta_keyword =
// parameter_meta.syntax().first_token().unwrap();     result.push_str(&
// format_inline_comment(         &SyntaxElement::Token(parameter_meta_keyword.
// clone()),         false,
//     ));

//     let open_brace = parameter_meta
//         .syntax()
//         .children_with_tokens()
//         .find(|c| c.kind() == SyntaxKind::OpenBrace)
//         .expect("parameter metadata section should have an open brace");
//     result.push_str(&format_preceding_comments(
//         &open_brace,
//         1,
//         !result.ends_with(NEWLINE),
//     ));
//     if result.ends_with(NEWLINE) {
//         result.push_str(INDENT);
//     } else {
//         result.push(' ');
//     }
//     result.push('{');
//     result.push_str(&format_inline_comment(&open_brace, true));

//     for item in parameter_meta.items() {
//         result.push_str(&format_metadata_item(&item));
//     }

//     let close_brace = parameter_meta
//         .syntax()
//         .children_with_tokens()
//         .find(|c| c.kind() == SyntaxKind::CloseBrace)
//         .expect("parameter metadata section should have a close brace");
//     result.push_str(&format_preceding_comments(&close_brace, 0, false));
//     result.push_str(INDENT);
//     result.push('}');

//     result.push_str(&format_inline_comment(
//         &SyntaxElement::Node(parameter_meta.syntax().clone()),
//         true,
//     ));

//     result
// }

// /// Format an input section.
// fn format_input_section(input: InputSection) -> String {
//     let mut result = String::new();

//     result.push_str(&format_preceding_comments(
//         &SyntaxElement::Node(input.syntax().clone()),
//         1,
//         false,
//     ));

//     result.push_str(INDENT);
//     result.push_str("input");
//     let input_keyword = input
//         .syntax()
//         .first_token()
//         .expect("input section should have a token");
//     result.push_str(&format_inline_comment(
//         &SyntaxElement::Token(input_keyword.clone()),
//         false,
//     ));

//     let open_brace = input
//         .syntax()
//         .children_with_tokens()
//         .find(|c| c.kind() == SyntaxKind::OpenBrace)
//         .expect("input section should have an open brace");
//     result.push_str(&format_preceding_comments(
//         &open_brace,
//         1,
//         !result.ends_with(NEWLINE),
//     ));
//     if result.ends_with(NEWLINE) {
//         result.push_str(INDENT);
//     } else {
//         result.push(' ');
//     }
//     result.push('{');
//     result.push_str(&format_inline_comment(&open_brace, true));

//     for decl in input.declarations() {
//         result.push_str(&format_declaration(&decl, 2));
//     }

//     result.push_str(&format_preceding_comments(
//         &SyntaxElement::Token(
//             input
//                 .syntax()
//                 .last_token()
//                 .expect("input section should have a token"),
//         ),
//         1,
//         false,
//     ));
//     result.push_str(INDENT);
//     result.push('}');
//     result.push_str(&format_inline_comment(
//         &SyntaxElement::Node(input.syntax().clone()),
//         true,
//     ));

//     result
// }

// /// Format an output section.
// fn format_output_section(output: OutputSection) -> String {
//     let mut result = String::new();

//     result.push_str(&format_preceding_comments(
//         &SyntaxElement::Node(output.syntax().clone()),
//         1,
//         false,
//     ));

//     result.push_str(INDENT);
//     result.push_str("output");
//     let output_keyword = output
//         .syntax()
//         .first_token()
//         .expect("output section should have a token");
//     result.push_str(&format_inline_comment(
//         &SyntaxElement::Token(output_keyword.clone()),
//         false,
//     ));
//     let open_brace = output
//         .syntax()
//         .children_with_tokens()
//         .find(|c| c.kind() == SyntaxKind::OpenBrace)
//         .expect("output section should have an open brace");
//     result.push_str(&format_preceding_comments(
//         &open_brace,
//         1,
//         !result.ends_with(NEWLINE),
//     ));
//     if result.ends_with(NEWLINE) {
//         result.push_str(INDENT);
//     } else {
//         result.push(' ');
//     }
//     result.push('{');
//     result.push_str(&format_inline_comment(&open_brace, true));

//     for decl in output.declarations() {
//         result.push_str(&format_declaration(&Decl::Bound(decl), 2));
//     }

//     result.push_str(&format_preceding_comments(
//         &SyntaxElement::Token(
//             output
//                 .syntax()
//                 .last_token()
//                 .expect("output section should have a token"),
//         ),
//         1,
//         false,
//     ));
//     result.push_str(INDENT);
//     result.push('}');
//     result.push_str(&format_inline_comment(
//         &SyntaxElement::Node(output.syntax().clone()),
//         true,
//     ));

//     result
// }

// /// Format a declaration.
// fn format_declaration(declaration: &Decl, num_indents: usize) -> String {
//     let mut result = String::new();
//     let next_indent_level = num_indents + 1;
//     let cur_indents = INDENT.repeat(num_indents);
//     let next_indents = INDENT.repeat(next_indent_level);

//     result.push_str(&format_preceding_comments(
//         &SyntaxElement::Node(declaration.syntax().clone()),
//         num_indents,
//         false,
//     ));
//     result.push_str(&cur_indents);

//     result.push_str(&declaration.ty().to_string());
//     result.push_str(&format_inline_comment(
//         &SyntaxElement::Node(declaration.ty().syntax().clone()),
//         false,
//     ));

//     result.push_str(&format_preceding_comments(
//         &SyntaxElement::Token(declaration.name().syntax().clone()),
//         next_indent_level,
//         !result.ends_with(NEWLINE),
//     ));
//     if result.ends_with(NEWLINE) {
//         result.push_str(&next_indents);
//     } else {
//         result.push(' ');
//     }
//     result.push_str(declaration.name().as_str());
//     result.push_str(&format_inline_comment(
//         &SyntaxElement::Token(declaration.name().syntax().clone()),
//         false,
//     ));

//     if let Some(expr) = declaration.expr() {
//         let equal_sign = declaration
//             .syntax()
//             .children_with_tokens()
//             .find(|c| c.kind() == SyntaxKind::Assignment)
//             .expect("Bound declaration should have an equal sign");

//         result.push_str(&format_preceding_comments(
//             &equal_sign,
//             next_indent_level,
//             !result.ends_with(NEWLINE),
//         ));
//         if result.ends_with(NEWLINE) {
//             result.push_str(&next_indents);
//         } else {
//             result.push(' ');
//         }
//         result.push('=');
//         result.push_str(&format_inline_comment(&equal_sign, false));

//         result.push_str(&format_preceding_comments(
//             &SyntaxElement::Node(expr.syntax().clone()),
//             next_indent_level,
//             !result.ends_with(NEWLINE),
//         ));
//         if result.ends_with(NEWLINE) {
//             result.push_str(&next_indents);
//         } else {
//             result.push(' ');
//         }
//         result.push_str(&expr.syntax().to_string()); // TODO: format
// expressions     }
//     result.push_str(&format_inline_comment(
//         &SyntaxElement::Node(declaration.syntax().clone()),
//         true,
//     ));

//     result
// }

// /// Format a struct definition
// fn format_struct_definition(struct_def: &StructDefinition) -> String {
//     let mut result = String::new();

//     result.push_str(&format_preceding_comments(
//         &SyntaxElement::Node(struct_def.syntax().clone()),
//         0,
//         false,
//     ));
//     result.push_str("struct");
//     let struct_keyword = struct_def
//         .syntax()
//         .first_token()
//         .expect("struct definition should have a token");
//     result.push_str(&format_inline_comment(
//         &SyntaxElement::Token(struct_keyword.clone()),
//         false,
//     ));

//     result.push_str(&format_preceding_comments(
//         &SyntaxElement::Token(struct_def.name().syntax().clone()),
//         1,
//         !result.ends_with(NEWLINE),
//     ));
//     if result.ends_with(NEWLINE) {
//         result.push_str(INDENT);
//     } else {
//         result.push(' ');
//     }
//     result.push_str(struct_def.name().as_str());
//     result.push_str(&format_inline_comment(
//         &SyntaxElement::Token(struct_def.name().syntax().clone()),
//         false,
//     ));

//     let open_brace = struct_def
//         .syntax()
//         .children_with_tokens()
//         .find(|c| c.kind() == SyntaxKind::OpenBrace)
//         .expect("struct definition should have an open brace");
//     result.push_str(&format_preceding_comments(
//         &open_brace,
//         0,
//         !result.ends_with(NEWLINE),
//     ));
//     if !result.ends_with(NEWLINE) {
//         result.push(' ');
//     }
//     result.push('{');
//     result.push_str(&format_inline_comment(&open_brace, true));

//     for decl in struct_def.members() {
//         result.push_str(&format_declaration(&Decl::Unbound(decl), 1));
//     }

//     let close_brace = struct_def
//         .syntax()
//         .children_with_tokens()
//         .find(|c| c.kind() == SyntaxKind::CloseBrace)
//         .expect("struct definition should have a close brace");
//     result.push_str(&format_preceding_comments(&close_brace, 0, false));
//     result.push('}');
//     result.push_str(&format_inline_comment(&close_brace, true));

//     result
// }

/// Format a WDL document.
pub fn format_document(code: &str) -> Result<String, Vec<Diagnostic>> {
    let (document, diagnostics) = Document::parse(code);
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let mut validator = Validator::default();
    match validator.validate(&document) {
        std::result::Result::Ok(_) => {
            // The document is valid, so we can format it.
        }
        Err(diagnostics) => return Err(diagnostics),
    }

    let mut result = String::new();
    let mut state = FormatState::default();

    let version_statement = document
        .version_statement()
        .expect("Document should have a version statement");
    match version_statement.format(&mut result, &mut state) {
        std::result::Result::Ok(_) => {}
        Err(_) => {
            return Err(vec![Diagnostic::error(
                "Failed to format version statement",
            )]);
        }
    }

    // let ast = document.ast();
    // let ast = ast.as_v1().unwrap();
    // result.push_str(&format_imports(ast.imports()));

    // ast.items().for_each(|item| {
    //     match item {
    //         DocumentItem::Import(_) => {
    //             // Imports have already been formatted
    //         }
    //         DocumentItem::Workflow(workflow_def) => {
    //             if !result.ends_with(&NEWLINE.repeat(2)) {
    //                 result.push_str(NEWLINE);
    //             }
    //             result.push_str(&format_workflow(&workflow_def));
    //         }
    //         DocumentItem::Task(task_def) => {
    //             if !result.ends_with(&NEWLINE.repeat(2)) {
    //                 result.push_str(NEWLINE);
    //             }
    //             result.push_str(&format_task(&task_def));
    //         }
    //         DocumentItem::Struct(struct_def) => {
    //             if !result.ends_with(&NEWLINE.repeat(2)) {
    //                 result.push_str(NEWLINE);
    //             }
    //             result.push_str(&format_struct_definition(&struct_def));
    //         }
    //     };
    // });

    std::result::Result::Ok(result)
}
