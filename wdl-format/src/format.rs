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
use wdl_ast::Validator;
use wdl_ast::VersionStatement;

/// Newline constant used for formatting.
pub const NEWLINE: &str = "\n";
/// Indentation constant used for formatting.
pub const INDENT: &str = "    ";
/// Inline comment space constant used for formatting.
pub const INLINE_COMMENT_SPACE: &str = "  ";

mod comments;
mod import;
mod workflow;

use comments::format_inline_comment;
use comments::format_preceeding_comments;
use import::format_imports;
use workflow::format_workflow;

/// Format a version statement.
fn format_version_statement(version_statement: VersionStatement) -> String {
    // Collect comments that preceed the version statement
    // Note as this must be the first element in the document,
    // the logic is simpler than the 'format_preceeding_comments' function.
    // We are walking backwards through the syntax tree, so we must collect
    // the comments in a vector and reverse them to get them in the correct order.
    let mut preceeding_comments = Vec::new();
    for sibling in version_statement
        .syntax()
        .siblings_with_tokens(Direction::Prev)
    {
        match sibling.kind() {
            SyntaxKind::Comment => {
                preceeding_comments.push(sibling.to_string().trim().to_owned());
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
    for comment in preceeding_comments.iter().rev() {
        result.push_str(comment);
        result.push_str(NEWLINE);
    }

    let mut trailing_comment = String::new();
    for child in version_statement.syntax().children_with_tokens() {
        match child.kind() {
            SyntaxKind::VersionKeyword => {
                // This should always be the first child processed
                if !result.is_empty() {
                    // If there are preamble comments, ensure a blank line is inserted
                    result.push_str(NEWLINE);
                }
                result.push_str("version ");
                result.push_str(version_statement.version().as_str());
            }
            SyntaxKind::Comment => {
                // This comment is in the middle of the version statement
                // It will be moved to after the version statement
                trailing_comment.push_str(child.to_string().trim());
                trailing_comment.push_str(NEWLINE);
            }
            SyntaxKind::Whitespace => {
                // Ignore
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

    result.push_str(&format_inline_comment(
        &SyntaxElement::Node(version_statement.syntax().clone()),
        "",
        NEWLINE,
    ));
    result.push_str(&trailing_comment);

    result.push_str(NEWLINE);
    result
}

/// Format a meta section.
fn format_meta_section(meta: Option<MetadataSection>) -> String {
    let mut result = String::new();
    let next_indent_level = format!("{}{}", INDENT, INDENT);

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

    result.push_str(&format_preceeding_comments(
        &SyntaxElement::Node(meta.syntax().clone()),
        1,
    ));

    for child in meta.syntax().children_with_tokens() {
        // TODO this logic can be simplified
        match child.kind() {
            SyntaxKind::MetaKeyword => {
                // This should always be the first child processed
                result.push_str(INDENT);
                result.push_str("meta");
                result.push_str(&format_inline_comment(&child, INDENT, " "));
                let mut next = child.next_sibling_or_token();
                while let Some(cur) = next {
                    match cur.kind() {
                        SyntaxKind::OpenBrace => {
                            result.push('{');
                            result.push_str(&format_inline_comment(&cur, "", NEWLINE));
                        }
                        SyntaxKind::Whitespace => {
                            // Ignore
                        }
                        SyntaxKind::Comment => {
                            // This will be handled by a call to either
                            // 'format_preceeding_comments'
                            // or 'format_inline_comment'.
                        }
                        SyntaxKind::MetadataObjectItemNode => {
                            result.push_str(&format_preceeding_comments(&cur, 2));
                            result.push_str(&next_indent_level);
                            result.push_str(&cur.to_string());
                            result.push_str(&format_inline_comment(&cur, "", NEWLINE));
                        }
                        SyntaxKind::CloseBrace => {
                            // Should always be last child processed
                            result.push_str(INDENT);
                            result.push('}');
                            result.push_str(&format_inline_comment(&cur, "", NEWLINE));
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
            SyntaxKind::OpenBrace
            | SyntaxKind::MetadataObjectItemNode
            | SyntaxKind::Comment
            | SyntaxKind::CloseBrace => {
                // Handled by the meta keyword
            }
            _ => {
                unreachable!("Unexpected syntax kind: {:?}", child.kind());
            }
        }
    }

    result
}

/// Format a parameter meta section.
/// TODO: refactor to share code with `format_meta_section`.
fn format_parameter_meta_section(parameter_meta: Option<ParameterMetadataSection>) -> String {
    let mut result = String::new();
    let next_indent_level = format!("{}{}", INDENT, INDENT);

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

    result.push_str(&format_preceeding_comments(
        &SyntaxElement::Node(parameter_meta.syntax().clone()),
        1,
    ));

    for child in parameter_meta.syntax().children_with_tokens() {
        match child.kind() {
            SyntaxKind::ParameterMetaKeyword => {
                // This should always be the first child processed
                result.push_str(INDENT);
                result.push_str("parameter_meta");
                result.push_str(&format_inline_comment(&child, INDENT, " "));
                let mut next = child.next_sibling_or_token();
                while let Some(cur) = next {
                    match cur.kind() {
                        SyntaxKind::OpenBrace => {
                            result.push('{');
                            result.push_str(&format_inline_comment(&cur, "", NEWLINE));
                        }
                        SyntaxKind::Whitespace => {
                            // Ignore
                        }
                        SyntaxKind::Comment => {
                            // This will be handled by a call to either
                            // 'format_preceeding_comments'
                            // or 'format_inline_comment'.
                        }
                        SyntaxKind::MetadataObjectItemNode => {
                            result.push_str(&format_preceeding_comments(&cur, 2));
                            result.push_str(&next_indent_level);
                            result.push_str(&cur.to_string());
                            result.push_str(&format_inline_comment(&cur, "", NEWLINE));
                        }
                        SyntaxKind::CloseBrace => {
                            // Should always be last child processed
                            result.push_str(INDENT);
                            result.push('}');
                            result.push_str(&format_inline_comment(&cur, "", NEWLINE));
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
            SyntaxKind::OpenBrace
            | SyntaxKind::MetadataObjectItemNode
            | SyntaxKind::Comment
            | SyntaxKind::CloseBrace => {
                // Handled by the parameter meta keyword
            }
            _ => {
                unreachable!("Unexpected syntax kind: {:?}", child.kind());
            }
        }
    }

    result
}

/// Format an input section.
fn format_input_section(input: Option<InputSection>) -> String {
    let mut result = String::new();
    let next_indent_level = format!("{}{}", INDENT, INDENT);

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

    result.push_str(&format_preceeding_comments(
        &SyntaxElement::Node(input.syntax().clone()),
        1,
    ));

    result.push_str(INDENT);
    result.push_str("input {");
    result.push_str(&format_inline_comment(
        &input
            .syntax()
            .first_child_or_token()
            .expect("Input section should have a child"),
        "",
        NEWLINE,
    ));

    for item in input.declarations() {
        result.push_str(&format_preceeding_comments(
            &SyntaxElement::Node(item.syntax().clone()),
            2,
        ));
        result.push_str(&next_indent_level);
        result.push_str(&item.syntax().to_string()); // TODO: Format the declaration
        result.push_str(&format_inline_comment(
            &SyntaxElement::Node(item.syntax().clone()),
            "",
            NEWLINE,
        ));
    }
    result.push_str(INDENT);
    result.push('}');
    result.push_str(&format_inline_comment(
        &SyntaxElement::Node(input
            .syntax().clone()),
        "",
        NEWLINE,
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
        }
    });

    Ok(result)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_with_comments() {
        let code = "\n\n    ## preamble comment  \nversion # weird comment\n1.1 # inline \
                    comment\nworkflow test {}";
        let formatted = format_document(code).unwrap();
        assert_eq!(
            formatted,
            "## preamble comment\n\nversion 1.1  # inline comment\n# weird comment\n\nworkflow \
             test {\n}\n\n"
        );
    }

    #[test]
    fn test_format_without_comments() {
        let code = "version 1.1\nworkflow test {}";
        let formatted = format_document(code).unwrap();
        assert_eq!(formatted, "version 1.1\n\nworkflow test {\n}\n\n");
    }

    #[test]
    fn test_format_with_imports() {
        let code = "
        version 1.1

        # this comment belongs to fileB
        import \"fileB.wdl\" as foo # also fileB
        import \"fileA.wdl\" as bar # middle of fileA
            alias qux as Qux
        workflow test {}
        # this comment belongs to fileC
        import \"fileC.wdl\"";
        let formatted = format_document(code).unwrap();
        assert_eq!(
            formatted,
            "version 1.1\n\nimport \"fileA.wdl\" as bar  # middle of fileA\n     alias qux as \
             Qux\n# this comment belongs to fileB\nimport \"fileB.wdl\" as foo  # also fileB\n# \
             this comment belongs to fileC\nimport \"fileC.wdl\"\n\nworkflow test {\n}\n\n"
        );
    }

    #[test]
    fn test_format_with_meta() {
        let code = "
        version 1.1

        workflow test { # workflow comment
        # meta comment
            meta {
        author: \"me\"  # author comment
        # email comment
        email: \"me@stjude.org\"
        }
        }";
        let formatted = format_document(code).unwrap();
        assert_eq!(
            formatted,
            "version 1.1\n\nworkflow test {  # workflow comment\n    # meta comment\n    meta {\n        author: \"me\"  # author comment\n        # email comment\n        email: \"me@stjude.org\"\n    }\n\n}\n\n"
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
             "version 1.1\n\nworkflow test {\n    input {\n        # foo comment\n        String foo  # another foo comment\n        Int # mid-bar comment\n        bar\n    }\n\n}\n\n"
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
        }";
        let formatted = format_document(code).unwrap();
        assert_eq!(
            formatted,
             "# preamble one\n# preamble two\n\nversion 1.1  # 2\n# 1\n\n# 3\nworkflow  # 4\n    test  # 5\n{  # 6\n    meta  # 7\n    {  # 8\n        description # 9\n        : # 10\n        \"what a nightmare\"  # 11\n    }\n\n    parameter_meta  # 13\n    {  # 14\n        foo # 15\n        : # 16\n        \"bar\"  # 17\n    }\n\n    input {  # 19\n        String # 21\n        foo  # 22\n    }  # 23\n\n}\n\n"
        );
    }
}
