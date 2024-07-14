//! A module for formatting WDL code.

use anyhow::Result;
use wdl_ast::v1::Decl;
use wdl_ast::v1::DocumentItem;
use wdl_ast::v1::InputSection;
use wdl_ast::v1::MetadataObjectItem;
use wdl_ast::v1::MetadataSection;
use wdl_ast::v1::OutputSection;
use wdl_ast::v1::ParameterMetadataSection;
use wdl_ast::v1::StructDefinition;
use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::Diagnostic;
use wdl_ast::Direction;
use wdl_ast::Document;
use wdl_ast::SyntaxElement;
use wdl_ast::SyntaxKind;
use wdl_ast::Validator;
use wdl_ast::VersionStatement;

/// Newline constant used for formatting.
pub const NEWLINE: &str = "\n";
/// Indentation constant used for formatting.
pub const INDENT: &str = "    ";

mod comments;
mod import;
mod task;
mod workflow;

use comments::format_inline_comment;
use comments::format_preceding_comments;
use import::format_imports;
use task::format_task;
use workflow::format_workflow;

/// Format a version statement.
fn format_version_statement(version_statement: VersionStatement) -> String {
    // Collect comments that preceed the version statement.
    // Note as this must be the first element in the document,
    // the logic is slightly different than the 'format_preceding_comments' function.
    // We are walking backwards through the syntax tree, so we must collect
    // the comments in a vector and reverse them to get them in the correct order.
    let mut preceding_comments = Vec::new();
    for sibling in version_statement
        .syntax()
        .siblings_with_tokens(Direction::Prev)
    {
        match sibling.kind() {
            SyntaxKind::Comment => {
                preceding_comments.push(sibling.to_string().trim().to_owned());
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

    let mut result = String::new();
    for comment in preceding_comments.iter().rev() {
        result.push_str(comment);
        result.push_str(NEWLINE);
    }

    if !result.is_empty() {
        // If there are preamble comments, ensure a blank line is inserted
        result.push_str(NEWLINE);
    }
    result.push_str("version");
    let version_keyword = version_statement.syntax().first_token().unwrap();
    result.push_str(&format_inline_comment(
        &SyntaxElement::Token(version_keyword),
        false,
    ));

    result.push_str(&format_preceding_comments(
        &SyntaxElement::Token(version_statement.version().syntax().clone()),
        1,
        !result.ends_with(NEWLINE),
    ));
    if result.ends_with("version") {
        result.push(' ');
    } else if result.ends_with(NEWLINE) {
        result.push_str(INDENT);
    }
    result.push_str(version_statement.version().as_str());
    result.push_str(&format_inline_comment(
        &SyntaxElement::Node(version_statement.syntax().clone()),
        true,
    ));

    result
}

/// Format the inner portion of a meta/parameter_meta section.
fn format_metadata_item(item: &MetadataObjectItem) -> String {
    let mut result = String::new();
    let two_indents = INDENT.repeat(2);
    let three_indents = INDENT.repeat(3);

    result.push_str(&format_preceding_comments(
        &SyntaxElement::Node(item.syntax().clone()),
        2,
        false,
    ));
    result.push_str(&two_indents);
    result.push_str(item.name().as_str());
    result.push_str(&format_inline_comment(
        &SyntaxElement::Token(item.name().syntax().clone()),
        false,
    ));

    let colon = item
        .syntax()
        .children_with_tokens()
        .find(|c| c.kind() == SyntaxKind::Colon)
        .expect("metadata item should have a colon");
    result.push_str(&format_preceding_comments(
        &colon,
        1,
        !result.ends_with(NEWLINE),
    ));
    if result.ends_with(NEWLINE) {
        result.push_str(&three_indents);
    }
    result.push(':');
    result.push_str(&format_inline_comment(&colon, false));

    result.push_str(&format_preceding_comments(
        &SyntaxElement::Node(item.value().syntax().clone()),
        1,
        !result.ends_with(NEWLINE),
    ));
    if result.ends_with(NEWLINE) {
        result.push_str(&three_indents);
    } else {
        result.push(' ');
    }
    result.push_str(&item.value().syntax().to_string());
    result.push_str(&format_inline_comment(
        &SyntaxElement::Node(item.syntax().clone()),
        true,
    ));

    result
}

/// Format a meta section.
fn format_meta_section(meta: MetadataSection) -> String {
    let mut result = String::new();

    result.push_str(&format_preceding_comments(
        &SyntaxElement::Node(meta.syntax().clone()),
        1,
        false,
    ));

    result.push_str(INDENT);
    result.push_str("meta");
    let meta_keyword = meta.syntax().first_token().unwrap();
    result.push_str(&format_inline_comment(
        &SyntaxElement::Token(meta_keyword.clone()),
        false,
    ));

    let open_brace = meta
        .syntax()
        .children_with_tokens()
        .find(|c| c.kind() == SyntaxKind::OpenBrace)
        .expect("metadata section should have an open brace");
    result.push_str(&format_preceding_comments(
        &open_brace,
        1,
        !result.ends_with(NEWLINE),
    ));
    if result.ends_with(NEWLINE) {
        result.push_str(INDENT);
    } else {
        result.push(' ');
    }
    result.push('{');
    result.push_str(&format_inline_comment(&open_brace, true));

    for item in meta.items() {
        result.push_str(&format_metadata_item(&item));
    }

    let close_brace = meta
        .syntax()
        .children_with_tokens()
        .find(|c| c.kind() == SyntaxKind::CloseBrace)
        .expect("metadata section should have a close brace");
    result.push_str(&format_preceding_comments(&close_brace, 0, false));
    result.push_str(INDENT);
    result.push('}');

    result.push_str(&format_inline_comment(
        &SyntaxElement::Node(meta.syntax().clone()),
        true,
    ));

    result
}

/// Format a parameter meta section.
fn format_parameter_meta_section(parameter_meta: ParameterMetadataSection) -> String {
    let mut result = String::new();

    result.push_str(&format_preceding_comments(
        &SyntaxElement::Node(parameter_meta.syntax().clone()),
        1,
        false,
    ));

    result.push_str(INDENT);
    result.push_str("parameter_meta");
    let parameter_meta_keyword = parameter_meta.syntax().first_token().unwrap();
    result.push_str(&format_inline_comment(
        &SyntaxElement::Token(parameter_meta_keyword.clone()),
        false,
    ));

    let open_brace = parameter_meta
        .syntax()
        .children_with_tokens()
        .find(|c| c.kind() == SyntaxKind::OpenBrace)
        .expect("parameter metadata section should have an open brace");
    result.push_str(&format_preceding_comments(
        &open_brace,
        1,
        !result.ends_with(NEWLINE),
    ));
    if result.ends_with(NEWLINE) {
        result.push_str(INDENT);
    } else {
        result.push(' ');
    }
    result.push('{');
    result.push_str(&format_inline_comment(&open_brace, true));

    for item in parameter_meta.items() {
        result.push_str(&format_metadata_item(&item));
    }

    let close_brace = parameter_meta
        .syntax()
        .children_with_tokens()
        .find(|c| c.kind() == SyntaxKind::CloseBrace)
        .expect("parameter metadata section should have a close brace");
    result.push_str(&format_preceding_comments(&close_brace, 0, false));
    result.push_str(INDENT);
    result.push('}');

    result.push_str(&format_inline_comment(
        &SyntaxElement::Node(parameter_meta.syntax().clone()),
        true,
    ));

    result
}

/// Format an input section.
fn format_input_section(input: InputSection) -> String {
    let mut result = String::new();

    result.push_str(&format_preceding_comments(
        &SyntaxElement::Node(input.syntax().clone()),
        1,
        false,
    ));

    result.push_str(INDENT);
    result.push_str("input");
    let input_keyword = input
        .syntax()
        .first_token()
        .expect("input section should have a token");
    result.push_str(&format_inline_comment(
        &SyntaxElement::Token(input_keyword.clone()),
        false,
    ));

    let open_brace = input
        .syntax()
        .children_with_tokens()
        .find(|c| c.kind() == SyntaxKind::OpenBrace)
        .expect("input section should have an open brace");
    result.push_str(&format_preceding_comments(
        &open_brace,
        1,
        !result.ends_with(NEWLINE),
    ));
    if result.ends_with(NEWLINE) {
        result.push_str(INDENT);
    } else {
        result.push(' ');
    }
    result.push('{');
    result.push_str(&format_inline_comment(&open_brace, true));

    for decl in input.declarations() {
        result.push_str(&format_declaration(&decl, 2));
    }

    result.push_str(&format_preceding_comments(
        &SyntaxElement::Token(
            input
                .syntax()
                .last_token()
                .expect("input section should have a token"),
        ),
        1,
        false,
    ));
    result.push_str(INDENT);
    result.push('}');
    result.push_str(&format_inline_comment(
        &SyntaxElement::Node(input.syntax().clone()),
        true,
    ));

    result
}

/// Format an output section.
fn format_output_section(output: OutputSection) -> String {
    let mut result = String::new();

    result.push_str(&format_preceding_comments(
        &SyntaxElement::Node(output.syntax().clone()),
        1,
        false,
    ));

    result.push_str(INDENT);
    result.push_str("output");
    let output_keyword = output
        .syntax()
        .first_token()
        .expect("output section should have a token");
    result.push_str(&format_inline_comment(
        &SyntaxElement::Token(output_keyword.clone()),
        false,
    ));
    let open_brace = output
        .syntax()
        .children_with_tokens()
        .find(|c| c.kind() == SyntaxKind::OpenBrace)
        .expect("output section should have an open brace");
    result.push_str(&format_preceding_comments(
        &open_brace,
        1,
        !result.ends_with(NEWLINE),
    ));
    if result.ends_with(NEWLINE) {
        result.push_str(INDENT);
    } else {
        result.push(' ');
    }
    result.push('{');
    result.push_str(&format_inline_comment(&open_brace, true));

    for decl in output.declarations() {
        result.push_str(&format_declaration(&Decl::Bound(decl), 2));
    }

    result.push_str(&format_preceding_comments(
        &SyntaxElement::Token(
            output
                .syntax()
                .last_token()
                .expect("output section should have a token"),
        ),
        1,
        false,
    ));
    result.push_str(INDENT);
    result.push('}');
    result.push_str(&format_inline_comment(
        &SyntaxElement::Node(output.syntax().clone()),
        true,
    ));

    result
}

/// Format a declaration.
fn format_declaration(declaration: &Decl, num_indents: usize) -> String {
    let mut result = String::new();
    let next_indent_level = num_indents + 1;
    let cur_indents = INDENT.repeat(num_indents);
    let next_indents = INDENT.repeat(next_indent_level);

    result.push_str(&format_preceding_comments(
        &SyntaxElement::Node(declaration.syntax().clone()),
        num_indents,
        false,
    ));
    result.push_str(&cur_indents);

    result.push_str(&declaration.ty().to_string());
    result.push_str(&format_inline_comment(
        &SyntaxElement::Node(declaration.ty().syntax().clone()),
        false,
    ));

    result.push_str(&format_preceding_comments(
        &SyntaxElement::Token(declaration.name().syntax().clone()),
        next_indent_level,
        !result.ends_with(NEWLINE),
    ));
    if result.ends_with(NEWLINE) {
        result.push_str(&next_indents);
    } else {
        result.push(' ');
    }
    result.push_str(declaration.name().as_str());
    result.push_str(&format_inline_comment(
        &SyntaxElement::Token(declaration.name().syntax().clone()),
        false,
    ));

    if let Some(expr) = declaration.expr() {
        let equal_sign = declaration
            .syntax()
            .children_with_tokens()
            .find(|c| c.kind() == SyntaxKind::Assignment)
            .expect("Bound declaration should have an equal sign");

        result.push_str(&format_preceding_comments(
            &equal_sign,
            next_indent_level,
            !result.ends_with(NEWLINE),
        ));
        if result.ends_with(NEWLINE) {
            result.push_str(&next_indents);
        } else {
            result.push(' ');
        }
        result.push('=');
        result.push_str(&format_inline_comment(&equal_sign, false));

        result.push_str(&format_preceding_comments(
            &SyntaxElement::Node(expr.syntax().clone()),
            next_indent_level,
            !result.ends_with(NEWLINE),
        ));
        if result.ends_with(NEWLINE) {
            result.push_str(&next_indents);
        } else {
            result.push(' ');
        }
        result.push_str(&expr.syntax().to_string());
    }
    result.push_str(&format_inline_comment(
        &SyntaxElement::Node(declaration.syntax().clone()),
        true,
    ));

    result
}

/// Format a struct definition
fn format_struct_definition(struct_def: &StructDefinition) -> String {
    let mut result = String::new();

    result.push_str(&format_preceding_comments(
        &SyntaxElement::Node(struct_def.syntax().clone()),
        0,
        false,
    ));
    result.push_str("struct");
    let struct_keyword = struct_def
        .syntax()
        .first_token()
        .expect("struct definition should have a token");
    result.push_str(&format_inline_comment(
        &SyntaxElement::Token(struct_keyword.clone()),
        false,
    ));

    result.push_str(&format_preceding_comments(
        &SyntaxElement::Token(struct_def.name().syntax().clone()),
        1,
        !result.ends_with(NEWLINE),
    ));
    if result.ends_with(NEWLINE) {
        result.push_str(INDENT);
    } else {
        result.push(' ');
    }
    result.push_str(struct_def.name().as_str());
    result.push_str(&format_inline_comment(
        &SyntaxElement::Token(struct_def.name().syntax().clone()),
        false,
    ));

    let open_brace = struct_def
        .syntax()
        .children_with_tokens()
        .find(|c| c.kind() == SyntaxKind::OpenBrace)
        .expect("struct definition should have an open brace");
    result.push_str(&format_preceding_comments(
        &open_brace,
        0,
        !result.ends_with(NEWLINE),
    ));
    if !result.ends_with(NEWLINE) {
        result.push(' ');
    }
    result.push('{');
    result.push_str(&format_inline_comment(&open_brace, true));

    for decl in struct_def.members() {
        result.push_str(&format_declaration(&Decl::Unbound(decl), 1));
    }

    let close_brace = struct_def
        .syntax()
        .children_with_tokens()
        .find(|c| c.kind() == SyntaxKind::CloseBrace)
        .expect("struct definition should have a close brace");
    result.push_str(&format_preceding_comments(&close_brace, 0, false));
    result.push('}');
    result.push_str(&format_inline_comment(&close_brace, true));

    result
}

/// Format a WDL document.
pub fn format_document(code: &str) -> Result<String, Vec<Diagnostic>> {
    let (document, diagnostics) = Document::parse(code);
    if !diagnostics.is_empty() {
        return Err(diagnostics);
    }
    let mut validator = Validator::default();
    match validator.validate(&document) {
        Ok(_) => {
            // The document is valid, so we can format it.
        }
        Err(diagnostics) => return Err(diagnostics),
    }

    let mut result = String::new();
    result.push_str(&format_version_statement(
        document.version_statement().unwrap(),
    ));
    result.push_str(NEWLINE);

    let ast = document.ast();
    let ast = ast.as_v1().unwrap();
    result.push_str(&format_imports(ast.imports()));

    ast.items().for_each(|item| {
        match item {
            DocumentItem::Import(_) => {
                // Imports have already been formatted
            }
            DocumentItem::Workflow(workflow_def) => {
                if !result.ends_with(&NEWLINE.repeat(2)) {
                    result.push_str(NEWLINE);
                }
                result.push_str(&format_workflow(&workflow_def));
            }
            DocumentItem::Task(task_def) => {
                if !result.ends_with(&NEWLINE.repeat(2)) {
                    result.push_str(NEWLINE);
                }
                result.push_str(&format_task(&task_def));
            }
            DocumentItem::Struct(struct_def) => {
                if !result.ends_with(&NEWLINE.repeat(2)) {
                    result.push_str(NEWLINE);
                }
                result.push_str(&format_struct_definition(&struct_def));
            }
        };
    });

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_with_imports_and_preceding_comments() {
        let code = "
        version 1.1

        workflow test {}
        # this comment belongs to fileC
        import \"fileC.wdl\"
        # this comment belongs to fileB
        import \"fileB.wdl\" as foo
        # fileA 1
        import
        # fileA 2.1
        # fileA 2.2
        \"fileA.wdl\"
        # fileA 3
        as
        # fileA 4
        bar
            # fileA 5
            alias
            # fileA 6
            qux
            # fileA 7
            as
            # fileA 8
            Qux";
        let formatted = format_document(code).unwrap();
        assert_eq!(
            formatted,
            "version 1.1\n\n# fileA 1\nimport\n    # fileA 2.1\n    # fileA 2.2\n    \
             \"fileA.wdl\"\n    # fileA 3\n    as\n        # fileA 4\n        bar\n    # fileA \
             5\n    alias\n        # fileA 6\n        qux\n        # fileA 7\n        as\n        \
             # fileA 8\n        Qux\n# this comment belongs to fileB\nimport \"fileB.wdl\"\n    \
             as foo\n# this comment belongs to fileC\nimport \"fileC.wdl\"\n\nworkflow test {\n}\n"
        );
    }

    #[test]
    fn test_format_with_imports_and_inline_comments() {
        let code = "
        version 1.0

        import \"fileB.wdl\" as foo # fileB
        workflow test {}
        import \"fileC.wdl\"
        import # fileA 1
        \"fileA.wdl\" # fileA 2
        as # fileA 3
        bar # fileA 4
            alias # fileA 5
            qux # fileA 6
            as # fileA 7
            Qux # fileA 8
        ";
        let formatted = format_document(code).unwrap();
        assert_eq!(
            formatted,
            "version 1.0\n\nimport  # fileA 1\n    \"fileA.wdl\"  # fileA 2\n    as  # fileA 3\n        bar  # fileA 4\n    alias  # fileA 5\n        qux  # fileA 6\n        as  # fileA 7\n        Qux  # fileA 8\nimport \"fileB.wdl\"\n    as foo  # fileB\nimport \"fileC.wdl\"\n\nworkflow test {\n}\n",
        );
    }

    #[test]
    fn test_format_without_comments() {
        let code = "version 1.1\nworkflow test {}";
        let formatted = format_document(code).unwrap();
        assert_eq!(formatted, "version 1.1\n\nworkflow test {\n}\n");
    }

    #[test]
    fn test_format_with_imports_and_all_comments() {
        let code = "
        version 1.1

        # this comment belongs to fileB
        import \"fileB.wdl\" as foo # also fileB
        # fileA 1.1
        import # fileA 1.2
        # fileA 2.1
        # fileA 2.2
        \"fileA.wdl\" # fileA 2.3
        # fileA 3.1
        as # fileA 3.2
        # fileA 4.1
        bar # fileA 4.2
            # fileA 5.1
            alias # fileA 5.2
            # fileA 6.1
            qux # fileA 6.2
            # fileA 7.1
            as # fileA 7.2
            # fileA 8.1
            Qux # fileA 8.2
        workflow test {}
        # this comment belongs to fileC
        import \"fileC.wdl\"";
        let formatted = format_document(code).unwrap();
        assert_eq!(
            formatted,
            "version 1.1\n\n# fileA 1.1\nimport  # fileA 1.2\n    # fileA 2.1\n    # fileA 2.2\n    \"fileA.wdl\"  # fileA 2.3\n    # fileA 3.1\n    as  # fileA 3.2\n        # fileA 4.1\n        bar  # fileA 4.2\n    # fileA 5.1\n    alias  # fileA 5.2\n        # fileA 6.1\n        qux  # fileA 6.2\n        # fileA 7.1\n        as  # fileA 7.2\n        # fileA 8.1\n        Qux  # fileA 8.2\n# this comment belongs to fileB\nimport \"fileB.wdl\"\n    as foo  # also fileB\n# this comment belongs to fileC\nimport \"fileC.wdl\"\n\nworkflow test {\n}\n"
        );
    }

    #[test]
    fn test_format_with_imports_and_no_comments() {
        let code = "
        version 1.1

        import \"fileB.wdl\" as foo
        import \"fileA.wdl\" as bar alias cows as horses
        workflow test {}
        import \"fileC.wdl\" alias qux as Qux";
        let formatted = format_document(code).unwrap();
        assert_eq!(
            formatted,
            "version 1.1\n\nimport \"fileA.wdl\"\n    as bar\n    alias cows as horses\nimport \
             \"fileB.wdl\"\n    as foo\nimport \"fileC.wdl\"\n    alias qux as Qux\n\nworkflow \
             test {\n}\n"
        );
    }

    #[test]
    fn test_format_with_meta_with_all_comments() {
        let code = "
        version 1.1

        workflow test { # workflow comment
        # meta comment
            meta # also meta comment
            # open brace
            { # open brace
        # author comment
        author: \"me\"  # author comment
        # email comment
        email: \"me@stjude.org\" # email comment
        } # trailing comment
        }";
        let formatted = format_document(code).unwrap();
        assert_eq!(
            formatted,
             "version 1.1\n\nworkflow test {  # workflow comment\n    # meta comment\n    meta  # also meta comment\n    # open brace\n    {  # open brace\n        # author comment\n        author: \"me\"  # author comment\n        # email comment\n        email: \"me@stjude.org\"  # email comment\n    }  # trailing comment\n\n}\n"
        );
    }

    #[test]
    fn test_format_with_meta_without_comments() {
        let code = "
        version 1.1

        workflow test {
            meta {
                author: \"me\"
                email: \"me@stjude.org\"
    }
    }
";
        let formatted = format_document(code).unwrap();
        assert_eq!(
            formatted,
            "version 1.1\n\nworkflow test {\n    meta {\n        author: \"me\"\n        email: \
             \"me@stjude.org\"\n    }\n\n}\n"
        );
    }
    #[test]
    fn test_format_with_parameter_metadata() {
        let code = "
        version 1.1
        # workflow comment
        workflow test {
            input {
            String foo
            }
        # parameter_meta comment
            parameter_meta { # parameter_meta comment
            foo: \"bar\" # foo comment
            }
    }
        
            ";
        let formatted = format_document(code).unwrap();
        assert_eq!(
            formatted,
             "version 1.1\n\n# workflow comment\nworkflow test {\n    # parameter_meta comment\n    parameter_meta {  # parameter_meta comment\n        foo: \"bar\"  # foo comment\n    }\n\n    input {\n        String foo\n    }\n\n}\n"
        );
    }

    #[test]
    fn test_format_with_inputs() {
        let code = "
        version 1.1

        workflow test {
        input {
        # foo comment
        String foo # another foo comment
        Int # mid-bar comment
        bar
        }
        }";
        let formatted = format_document(code).unwrap();
        assert_eq!(
            formatted,
            "version 1.1\n\nworkflow test {\n    input {\n        # foo comment\n        String \
             foo  # another foo comment\n        Int  # mid-bar comment\n            bar\n    \
             }\n\n}\n"
        );
    }

    #[test]
    fn test_format_with_calls() {
        let code = "
        version 1.1

        workflow test {
        # foo comment
        call foo
        # bar comment
        call bar as baz
        call qux # mid-qux inline comment
        # mid-qux full-line comment
        after baz # after qux
        call lorem after ipsum { input: # after input token
        bazam,
        bam = select_bam
        }
        }";
        let formatted = format_document(code).unwrap();
        assert_eq!(
            formatted,
            "version 1.1\n\nworkflow test {\n    # foo comment\n    call foo\n    # bar comment\n    call bar as baz\n    call qux  # mid-qux inline comment\n        # mid-qux full-line comment\n        after baz  # after qux\n    call lorem after ipsum { input:  # after input token\n        bazam,\n        bam = select_bam,\n    }\n}\n"
        );
    }

    #[test]
    fn test_format_with_conditionals_and_scatters() {
        let code = "
        version 1.1

        workflow test {
        if (true) {
        call foo
        scatter (abc in bar) {
        if (false) {
        call bar
        }
        if (
        a > b # expr comment
        ) {
        scatter (x in [1, 2, 3]) {
        call baz
    }}
    }}
        }";
        let formatted = format_document(code).unwrap();
        assert_eq!(
            formatted,
            "version 1.1\n\nworkflow test {\n    if (true) {\n        call foo\n        scatter (abc in bar) {\n            if (false) {\n                call bar\n            }\n            if (a > b  # expr comment\n            ) {\n                scatter (x in [1, 2, 3]) {\n                    call baz\n                }\n            }\n        }\n    }\n}\n"
        );
    }

    #[test]
    fn test_format_with_inline_comments() {
        let code = "
        # preamble one
        # preamble two
        version # 1
        1.1 # 2
        workflow # 3
        test # 4
        { # 5
        meta # 6
        { # 7
        # 8
        # 9
        description # 10
        : # 11
        \"what a nightmare\" # 12
        } # 13
        parameter_meta # 14
        { # 15
        foo # 16
        : # 17
        \"bar\" # 18
        } # 19
        input # 20
        { # 21
        String # 22
        foo # 23
        } # 24
        if # 25
        ( # 26
        true # 27
        ) # 28
        { # 29
         scatter # 30
         ( # 31
            x # 32
            in # 33
            [1,2,3] # 34
            ) # 35
            { # 36
            call # 37
            task # 38
            as # 39
            task_alias # 40
            after # 41
            cows_come_home # 42
            } # 43
    } # 44
    } # 45
        ";
        let formatted = format_document(code).unwrap();
        assert_eq!(
            formatted,
            "# preamble one\n# preamble two\n\nversion  # 1\n    1.1  # 2\n\nworkflow  # 3\n    test  # 4\n{  # 5\n    meta  # 6\n    {  # 7\n        # 8\n        # 9\n        description  # 10\n            :  # 11\n            \"what a nightmare\"  # 12\n    }  # 13\n\n    parameter_meta  # 14\n    {  # 15\n        foo  # 16\n            :  # 17\n            \"bar\"  # 18\n    }  # 19\n\n    input  # 20\n    {  # 21\n        String  # 22\n            foo  # 23\n    }  # 24\n\n    if  # 25\n    (  # 26\n        true  # 27\n    )  # 28\n    {  # 29\n        scatter  # 30\n        (  # 31\n            x  # 32\n            in  # 33\n            [1,2,3]  # 34\n        )  # 35\n        {  # 36\n            call  # 37\n                task  # 38\n                as  # 39\n                task_alias  # 40\n                after  # 41\n                cows_come_home  # 42\n        }  # 43\n    }  # 44\n}  # 45\n"
        );
    }
}
