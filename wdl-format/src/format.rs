//! A module for formatting WDL code.

use anyhow::bail;
use anyhow::Result;
use wdl_ast::v1::DocumentItem;
use wdl_ast::v1::InputSection;
use wdl_ast::v1::MetadataSection;
use wdl_ast::v1::ParameterMetadataSection;
use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::Direction;
use wdl_ast::Document;
use wdl_ast::SyntaxElement;
use wdl_ast::SyntaxKind;
use wdl_ast::SyntaxTree;
use wdl_ast::Validator;
use wdl_ast::VersionStatement;

/// Newline constant used for formatting.
pub const NEWLINE: &str = "\n";
/// Indentation constant used for formatting.
pub const INDENT: &str = "    ";

mod comments;
mod import;
mod workflow;

use comments::format_inline_comment;
use comments::format_preceding_comments;
use import::format_imports;
use workflow::format_workflow;

/// Format a version statement.
fn format_version_statement(version_statement: VersionStatement) -> String {
    // Collect comments that preceed the version statement
    // Note as this must be the first element in the document,
    // the logic is simpler than the 'format_preceding_comments' function.
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
        false,
        false,
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

    result.push_str(NEWLINE);
    result
}

/// Format the inner portion of a meta/parameter_meta section.
fn format_metadata_children(item: &SyntaxElement) -> String {
    let mut result = String::new();
    let cur_indent_level = INDENT;
    let next_indent_level = format!("{}{}", INDENT, INDENT);

    match item.kind() {
        SyntaxKind::OpenBrace => {
            result.push_str(&format_preceding_comments(&item, 1, false, true));
            if !result.ends_with(INDENT) {
                result.push(' ');
            }
            result.push('{');
            result.push_str(&format_inline_comment(&item, true));
        }
        SyntaxKind::Whitespace => {
            // Ignore
        }
        SyntaxKind::Comment => {
            // This will be handled by a call to either
            // 'format_preceding_comments'
            // or 'format_inline_comment'.
        }
        SyntaxKind::MetadataObjectItemNode => {
            result.push_str(&format_preceding_comments(&item, 2, false, false));
            result.push_str(&next_indent_level);
            result.push_str(&item.to_string());
            result.push_str(&format_inline_comment(&item, true));
        }
        SyntaxKind::CloseBrace => {
            // Should always be last child processed
            result.push_str(&format_preceding_comments(&item, 1, false, true));
            result.push_str(cur_indent_level);
            result.push('}');
            result.push_str(&format_inline_comment(&item, true));
        }
        _ => {
            unreachable!("Unexpected syntax kind: {:?}", item.kind());
        }
    }
    result
}

/// Format a meta section.
fn format_meta_section(meta: Option<MetadataSection>) -> String {
    let mut result = String::new();

    if meta.is_none() {
        // result.push_str(INDENT);
        // result.push_str("meta {");
        // result.push_str(NEWLINE);
        // result.push_str(INDENT);
        // result.push('}');
        // result.push_str(NEWLINE);
        return result;
    }
    let meta = meta.unwrap();
    let cur_indent_level = INDENT;

    result.push_str(&format_preceding_comments(
        &SyntaxElement::Node(meta.syntax().clone()),
        1,
        false,
        false,
    ));

    result.push_str(cur_indent_level);
    result.push_str("meta");
    let meta_keyword = meta.syntax().first_token().unwrap();
    result.push_str(&format_inline_comment(
        &SyntaxElement::Token(meta_keyword.clone()),
        false,
    ));

    let mut next = meta_keyword.next_sibling_or_token();
    while let Some(cur) = next {
        result.push_str(&format_metadata_children(&cur));
        next = cur.next_sibling_or_token();
    }

    result
}

/// Format a parameter meta section.
fn format_parameter_meta_section(parameter_meta: Option<ParameterMetadataSection>) -> String {
    let mut result = String::new();

    if parameter_meta.is_none() {
        // result.push_str(INDENT);
        // result.push_str("parameter_meta {");
        // result.push_str(NEWLINE);
        // result.push_str(INDENT);
        // result.push('}');
        // result.push_str(NEWLINE);
        return result;
    }
    let parameter_meta = parameter_meta.unwrap();
    let cur_indent_level = INDENT;

    result.push_str(&format_preceding_comments(
        &SyntaxElement::Node(parameter_meta.syntax().clone()),
        1,
        false,
        false,
    ));

    result.push_str(cur_indent_level);
    result.push_str("parameter_meta");
    let parameter_meta_keyword = parameter_meta.syntax().first_token().unwrap();
    result.push_str(&format_inline_comment(&SyntaxElement::Token(parameter_meta_keyword.clone()), false));

    let mut next = parameter_meta_keyword.next_sibling_or_token();
    while let Some(cur) = next {
        result.push_str(&format_metadata_children(&cur));
        next = cur.next_sibling_or_token();
    }

    result
}

/// Format an input section.
fn format_input_section(input: Option<InputSection>) -> String {
    let mut result = String::new();

    if input.is_none() {
        // result.push_str(INDENT);
        // result.push_str("input {");
        // result.push_str(NEWLINE);
        // result.push_str(INDENT);
        // result.push('}');
        // result.push_str(NEWLINE);
        return result;
    }
    let input = input.unwrap();
    let cur_indent_level = INDENT;
    let next_indent_level = format!("{}{}", INDENT, INDENT);

    result.push_str(&format_preceding_comments(
        &SyntaxElement::Node(input.syntax().clone()),
        1,
        false,
        false,
    ));

    result.push_str(cur_indent_level);
    result.push_str("input");
    let input_token = input
        .syntax()
        .first_token()
        .expect("input section should have a token");
    result.push_str(&format_inline_comment(
        &SyntaxElement::Token(input_token.clone()),
        false,
    ));
    let open_brace = input
        .syntax()
        .children_with_tokens()
        .find(|c| c.kind() == SyntaxKind::OpenBrace)
        .expect("input section should have an open brace");
    result.push_str(&format_preceding_comments(&open_brace, 1, false, true));
    if !result.ends_with(INDENT) {
        result.push(' ');
    }
    result.push('{');
    result.push_str(&format_inline_comment(&open_brace, true));

    for item in input.declarations() {
        result.push_str(&format_preceding_comments(
            &SyntaxElement::Node(item.syntax().clone()),
            2,
            false,
            false,
        ));
        result.push_str(&next_indent_level);
        result.push_str(&item.syntax().to_string()); // TODO: Format the declaration
        result.push_str(&format_inline_comment(
            &SyntaxElement::Node(item.syntax().clone()),
            true,
        ));
    }
    result.push_str(&format_preceding_comments(
        &SyntaxElement::Token(
            input
                .syntax()
                .last_token()
                .expect("input section should have a token"),
        ),
        0,
        false,
        false,
    ));
    result.push_str(cur_indent_level);
    result.push('}');
    result.push_str(&format_inline_comment(
        &SyntaxElement::Node(input.syntax().clone()),
        true,
    ));

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

    ast.items().for_each(|item| {
        match item {
            DocumentItem::Import(_) => {
                // Imports have already been formatted
            }
            DocumentItem::Workflow(workflow_def) => {
                result.push_str(&format_workflow(workflow_def));
            }
            DocumentItem::Task(task_def) => {
                // TODO: Format the task
            }
            DocumentItem::Struct(struct_def) => {
                // TODO: Format the struct type
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
            "version 1.1\n\n# fileA 1\nimport\n    # fileA 2.1\n    # fileA 2.2\n    \"fileA.wdl\"\n    # fileA 3\n    as\n        # fileA 4\n        bar\n    # fileA 5\n    alias\n        # fileA 6\n        qux\n        # fileA 7\n        as\n        # fileA 8\n        Qux\n# this comment belongs to fileB\nimport \"fileB.wdl\"\n    as foo\n# this comment belongs to fileC\nimport \"fileC.wdl\"\n\nworkflow test {\n}\n\n"
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
            "version 1.0\n\nimport  # fileA 1\n    \"fileA.wdl\"  # fileA 2\n    as  # fileA 3\n        bar  # fileA 4\n    alias  # fileA 5\n        qux  # fileA 6\n        as  # fileA 7\n        Qux  # fileA 8\nimport \"fileB.wdl\"\n    as foo\nimport \"fileC.wdl\"\n\nworkflow test {\n}\n\n",
        );
    }

    #[test]
    fn test_format_without_comments() {
        let code = "version 1.1\nworkflow test {}";
        let formatted = format_document(code).unwrap();
        assert_eq!(formatted, "version 1.1\n\nworkflow test {\n}\n\n");
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
            "version 1.1\n\n# fileA 1.1\nimport  # fileA 1.2\n    # fileA 2.1\n    # fileA 2.2\n    \"fileA.wdl\"  # fileA 2.3\n    # fileA 3.1\n    as  # fileA 3.2\n        # fileA 4.1\n        bar  # fileA 4.2\n    # fileA 5.1\n    alias  # fileA 5.2\n        # fileA 6.1\n        qux  # fileA 6.2\n        # fileA 7.1\n        as  # fileA 7.2\n        # fileA 8.1\n        Qux  # fileA 8.2\n# this comment belongs to fileB\nimport \"fileB.wdl\"\n    as foo\n# this comment belongs to fileC\nimport \"fileC.wdl\"\n\nworkflow test {\n}\n\n"
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
             test {\n}\n\n"
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
             "version 1.1\n\nworkflow test {  # workflow comment\n    # meta comment\n    meta  # also meta comment\n    # open brace\n    {  # open brace\n        # author comment\n        author: \"me\"  # author comment\n        # email comment\n        email: \"me@stjude.org\"  # email comment\n    }\n\n}\n\n"
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
            "version 1.1\n\nworkflow test {\n    meta {\n        author: \"me\"\n        email: \"me@stjude.org\"\n    }\n\n}\n\n"
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
             "version 1.1\n\n# workflow comment\nworkflow test {\n    # parameter_meta comment\n    parameter_meta {  # parameter_meta comment\n        foo: \"bar\"  # foo comment\n    }\n\n    input {\n        String foo\n    }\n\n}\n\n"
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
             foo  # another foo comment\n        Int # mid-bar comment\n        bar\n    \
             }\n\n}\n\n"
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
        }
        }";
        let formatted = format_document(code).unwrap();
        assert_eq!(
            formatted,
            "version 1.1\n\nworkflow test {\n    # foo comment\n    call foo\n    # bar comment\n    call bar as baz\n    call qux  # mid-qux inline comment\n        after baz  # mid-qux full-line comment\n    call lorem after ipsum { input:  # after input token\n    }\n\n}\n\n"
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
        if (a >
        b) {
        scatter (x in [
        1, 2, 3
        ]) {
        call baz
    }}
    }}
        }";
        let formatted = format_document(code).unwrap();
        assert_eq!(
            formatted,
            "version 1.1\n\nworkflow test {\n    if (true) {\n        call foo\n        scatter (abc in bar) {\n            if (false) {\n                call bar\n            }\n            if (a >\n        b\n            ) {\n                scatter (x in [\n        1, 2, 3\n        ]\n                ) {\n                    call baz\n                }\n            }\n        }\n    }\n\n}\n\n"
        );
    }

    #[test]
    fn test_format_with_comment_between_every_token() {
        let code = "
        # preamble one
        # preamble two
        version # 1
        1.1 # 2
        # 3
        workflow # 4
        test # 5
        { # 6
        meta # 7
        { # 8
        description # 9
        : # 10
        \"what a nightmare\" # 11
        } # 12
        parameter_meta # 13
        { # 14
        foo # 15
        : # 16
        \"bar\" # 17
        } # 18
        input # 19
        { # 20
        String # 21
        foo # 22
        } # 23
        if # 24
        ( # 25
        true # 26
        ) # 27
        { # 28
         scatter # 29
         ( # 30
            x # 31
            in # 32
            [1,2,3] # 33
            ) # 34
            { # 35
            call # 36
            task # 37
            as # 38
            task_alias # 39
            after # 40
            cows_come_home # 41
            } # 42
    } # 43
    } # 44
        ";
        let formatted = format_document(code).unwrap();
        assert_eq!(
            formatted,
            "# preamble one\n# preamble two\n\nversion 1.1  # 2\n# 1\n\n# 3\nworkflow  # 4\n    \
             test  # 5\n{  # 6\n    meta  # 7\n    {  # 8\n        description # 9\n        : # \
             10\n        \"what a nightmare\"  # 11\n    }\n\n    parameter_meta  # 13\n    {  # \
             14\n        foo # 15\n        : # 16\n        \"bar\"  # 17\n    }\n\n    input {  # \
             19\n        String # 21\n        foo  # 22\n    }  # 23\n\n}\n\n"
        );
    }
}
