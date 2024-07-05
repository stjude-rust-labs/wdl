//! A module for formatting WDL code.

use std::collections::HashMap;

use anyhow::bail;
use anyhow::Result;
use wdl_ast::v1::Decl;
use wdl_ast::v1::DocumentItem;
use wdl_ast::v1::ImportStatement;
use wdl_ast::v1::InputSection;
use wdl_ast::v1::MetadataSection;
use wdl_ast::v1::ParameterMetadataSection;
use wdl_ast::v1::WorkflowDefinition;
use wdl_ast::v1::WorkflowItem;
use wdl_ast::AstChildren;
use wdl_ast::AstNode;
use wdl_ast::AstToken;
use wdl_ast::Direction;
use wdl_ast::Document;
use wdl_ast::SyntaxElement;
use wdl_ast::SyntaxKind;
use wdl_ast::SyntaxNode;
use wdl_ast::Validator;
use wdl_ast::VersionStatement;
use wdl_ast::WorkflowDescriptionLanguage;

const NEWLINE: &str = "\n";
const INDENT: &str = "    ";

/// Format a version statement.
fn format_version_statement(version_statement: VersionStatement) -> String {
    let mut result = String::new();
    // Collect comments that preceed the version statement
    // Note as this must be the first element in the document,
    // the logic is simpler than the 'format_preceeding_comments' function.
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

    for child in version_statement.syntax().children_with_tokens() {
        match child.kind() {
            SyntaxKind::VersionKeyword => {
                // This should always be the first child processed
                if !result.is_empty() {
                    result.push_str(NEWLINE);
                }
                result.push_str("version ");
                result.push_str(version_statement.version().as_str());
                result.push_str(NEWLINE);
            }
            SyntaxKind::Comment => {
                // This comment is in the middle of the version statement
                // It will be moved to after the version statement
                result.push_str(child.as_token().unwrap().text().trim());
                result.push_str(NEWLINE);
            }
            SyntaxKind::Whitespace => {
                // Skip whitespace
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

/// Format comments that preceed a node.
fn format_preceeding_comments(
    node: &impl AstNode<Language = WorkflowDescriptionLanguage>,
    root_kind: SyntaxKind,
    num_indents: usize,
) -> String {
    let mut preceeding_comments = Vec::new();
    let mut processed_root = false;

    for sibling in node.syntax().siblings_with_tokens(Direction::Prev) {
        match sibling.kind() {
            SyntaxKind::Comment => {
                // Ensure this comment "belongs" to the node.
                // A preceeding comment on a blank line is considered to belong to the node.
                // Othewise, the comment "belongs" to whatever
                // else is on that line.
                if let Some(cur) = sibling.prev_sibling_or_token() {
                    match cur.kind() {
                        SyntaxKind::Whitespace => {
                            if cur.as_token().unwrap().text().contains('\n') {
                                // The 'sibling' comment is on is on its own line.
                                // It "belongs" to the current node.
                                preceeding_comments
                                    .push(sibling.as_token().unwrap().text().trim().to_string());
                            }
                        }
                        _ => {
                            // The 'sibling' comment is on the same line as this
                            // token. It "belongs"
                            // to whatever is currently being processed.
                        }
                    }
                }
            }
            SyntaxKind::Whitespace => {
                // Skip whitespace
            }
            root_kind => {
                if processed_root {
                    // This must be a element of the same kind as the root
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

    let mut result = String::new();
    for comment in preceeding_comments.iter().rev() {
        for _ in 0..num_indents {
            result.push_str(INDENT);
        }
        result.push_str(comment);
        result.push_str(NEWLINE);
    }
    result
}

/// Format a comment on the same line as a node.
fn format_inline_comment(node: &SyntaxElement) -> String {
    let mut result = String::new();
    let mut sibling = node.next_sibling_or_token();
    while let Some(cur) = sibling {
        match cur.kind() {
            SyntaxKind::Comment => {
                result.push_str("  ");
                result.push_str(cur.as_token().unwrap().text().trim());
            }
            SyntaxKind::Whitespace => {
                if cur.as_token().unwrap().text().contains('\n') {
                    // We've looked ahead past the current line, so we can stop
                    break;
                }
            }
            _ => {
                // We've looked ahead past any trivia, so we can stop
                // break;
            }
        }
        sibling = cur.next_sibling_or_token();
    }
    result
}

/// Format a list of import statements.
fn format_imports(imports: AstChildren<ImportStatement>) -> String {
    let mut import_map: HashMap<String, String> = HashMap::new();
    for import in imports {
        let key = import.syntax().to_string();
        let mut val = String::new();

        val.push_str(&format_preceeding_comments(
            &import,
            SyntaxKind::ImportStatementNode,
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

        val.push_str(&format_inline_comment(&SyntaxElement::Node(
            import.syntax().clone(),
        )));
        val.push_str(NEWLINE);

        import_map.insert(key, val);
    }

    let mut import_vec: Vec<_> = import_map.into_iter().collect();
    import_vec.sort_by(|a, b| a.0.cmp(&b.0));

    let mut result = String::new();
    for (_, val) in import_vec {
        result.push_str(&val);
    }
    if !result.is_empty() {
        result.push_str(NEWLINE);
    }
    result
}

/// Format a meta section.
fn format_meta_section(meta: Option<MetadataSection>) -> String {
    let mut result = String::new();

    if meta.is_none() {
        result.push_str(INDENT);
        result.push_str("meta {");
        result.push_str(NEWLINE);
        result.push_str(INDENT);
        result.push('}');
        result.push_str(NEWLINE);
        return result;
    }
    let meta = meta.unwrap();

    result.push_str(&format_preceeding_comments(
        &meta,
        SyntaxKind::MetadataSectionNode,
        1,
    ));

    result.push_str(INDENT);
    result.push_str("meta {");
    result.push_str(&format_inline_comment(
        &meta
            .syntax()
            .first_child_or_token()
            .expect("Metadata section should have a child"),
    ));
    result.push_str(NEWLINE);

    for item in meta.items() {
        result.push_str(&format_preceeding_comments(
            &item,
            SyntaxKind::MetadataObjectItemNode,
            2,
        ));
        result.push_str(INDENT);
        result.push_str(INDENT);
        result.push_str(item.name().as_str());
        result.push_str(": ");
        result.push_str(&item.value().syntax().to_string());
        result.push_str(&format_inline_comment(&SyntaxElement::Node(
            item.syntax().clone(),
        )));
        result.push_str(NEWLINE);
    }
    result.push_str(INDENT);
    result.push('}');
    result.push_str(&format_inline_comment(
        &meta
            .syntax()
            .last_child_or_token()
            .expect("Metadata section should have a child"),
    ));
    result.push_str(NEWLINE);
    result
}

/// Format a parameter meta section.
/// TODO: refactor to share code with `format_meta_section`.
fn format_parameter_meta_section(parameter_meta: Option<ParameterMetadataSection>) -> String {
    let mut result = String::new();

    if parameter_meta.is_none() {
        result.push_str(INDENT);
        result.push_str("parameter_meta {");
        result.push_str(NEWLINE);
        result.push_str(INDENT);
        result.push('}');
        result.push_str(NEWLINE);
        return result;
    }
    let parameter_meta = parameter_meta.unwrap();

    result.push_str(&format_preceeding_comments(
        &parameter_meta,
        SyntaxKind::ParameterMetadataSectionNode,
        1,
    ));

    result.push_str(INDENT);
    result.push_str("parameter_meta {");
    result.push_str(&format_inline_comment(
        &parameter_meta
            .syntax()
            .first_child_or_token()
            .expect("Parameter metadata section should have a child"),
    ));
    result.push_str(NEWLINE);

    for item in parameter_meta.items() {
        result.push_str(&format_preceeding_comments(
            &item,
            SyntaxKind::MetadataObjectItemNode,
            2,
        ));
        result.push_str(INDENT);
        result.push_str(INDENT);
        result.push_str(item.name().as_str());
        result.push_str(": ");
        result.push_str(&item.value().syntax().to_string());
        result.push_str(&format_inline_comment(&SyntaxElement::Node(
            item.syntax().clone(),
        )));
        result.push_str(NEWLINE);
    }
    result.push_str(INDENT);
    result.push('}');
    result.push_str(&format_inline_comment(
        &parameter_meta
            .syntax()
            .last_child_or_token()
            .expect("Parameter metadata section should have a child"),
    ));
    result.push_str(NEWLINE);
    result
}

/// Format an input section.
fn format_input_section(input: Option<InputSection>) -> String {
    let mut result = String::new();

    if input.is_none() {
        result.push_str(INDENT);
        result.push_str("input {");
        result.push_str(NEWLINE);
        result.push_str(INDENT);
        result.push('}');
        result.push_str(NEWLINE);
        return result;
    }
    let input = input.unwrap();

    result.push_str(&format_preceeding_comments(
        &input,
        SyntaxKind::InputSectionNode,
        1,
    ));

    result.push_str(INDENT);
    result.push_str("input {");
    result.push_str(&format_inline_comment(
        &input
            .syntax()
            .first_child_or_token()
            .expect("Input section should have a child"),
    ));
    result.push_str(NEWLINE);

    for item in input.declarations() {
        result.push_str(&format_preceeding_comments(&item, item.syntax().kind(), 2));
        result.push_str(INDENT);
        result.push_str(INDENT);
        result.push_str(&item.syntax().to_string()); // TODO: Format the declaration
        result.push_str(&format_inline_comment(&SyntaxElement::Node(
            item.syntax().clone(),
        )));
        result.push_str(NEWLINE);
    }
    result.push_str(INDENT);
    result.push('}');
    result.push_str(&format_inline_comment(
        &input
            .syntax()
            .last_child_or_token()
            .expect("Input section should have a child"),
    ));
    result.push_str(NEWLINE);
    result
}

/// Format a workflow definition.
fn format_workflow(workflow_def: WorkflowDefinition) -> String {
    let mut result = String::new();
    result.push_str(&format_preceeding_comments(
        &workflow_def,
        SyntaxKind::WorkflowDefinitionNode,
        0,
    ));

    result.push_str("workflow ");
    result.push_str(workflow_def.name().as_str());
    result.push_str(" {");
    result.push_str(&format_inline_comment(
        &workflow_def
            .syntax()
            .first_child_or_token()
            .expect("Workflow definition should have a child"),
    ));
    result.push_str(NEWLINE);

    result.push_str(&format_meta_section(workflow_def.metadata().next()));
    result.push_str(NEWLINE);

    result.push_str(&format_parameter_meta_section(
        workflow_def.parameter_metadata().next(),
    ));
    result.push_str(NEWLINE);

    result.push_str(&format_input_section(workflow_def.inputs().next()));
    result.push_str(NEWLINE);

    for item in workflow_def.items() {
        match item {
            WorkflowItem::Call(call) => {
                // TODO
            }
            WorkflowItem::Conditional(conditional) => {
                // TODO
            }
            WorkflowItem::Scatter(scatter) => {
                // TODO
            }
            WorkflowItem::Declaration(decl) => {
                // TODO
            }
            WorkflowItem::Metadata(_)
            | WorkflowItem::ParameterMetadata(_)
            | WorkflowItem::Input(_) => {
                // Already handled
            }
            WorkflowItem::Output(_) => {
                // TODO
            }
        }
    }

    result.push('}');
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

    ast.items().for_each(|item| {
        match item {
            DocumentItem::Import(_) => {
                // Imports have already been formatted
            }
            DocumentItem::Workflow(workflow_def) => {
                result.push_str(&format_workflow(workflow_def));
            }
            DocumentItem::Task(_task_def) => {
                // TODO: Format the task
            }
            DocumentItem::Struct(_struct_def) => {
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
        let code = "\n\n    ## preamble comment  \nversion # weird comment\n 1.1\nworkflow test {}";
        let formatted = format_document(code).unwrap();
        assert_eq!(
            formatted,
             "## preamble comment\n\nversion 1.1\n# weird comment\n\nworkflow test {\n    meta {\n    }\n\n    parameter_meta {\n    }\n\n    input {\n    }\n\n}\n"
        );
    }

    #[test]
    fn test_format_without_comments() {
        let code = "version 1.1\nworkflow test {}";
        let formatted = format_document(code).unwrap();
        assert_eq!(formatted, "version 1.1\n\nworkflow test {\n    meta {\n    }\n\n    parameter_meta {\n    }\n\n    input {\n    }\n\n}\n");
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
             comment belongs to fileC\nimport \"fileC.wdl\"\n\nworkflow test {\n    meta {\n    \
             }\n\n    parameter_meta {\n    }\n\n    input {\n    }\n\n}\n"
        );
    }

    #[test]
    fn test_format_with_metadata() {
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
            "version 1.1\n\nworkflow test {  # workflow comment\n    # meta comment\n    meta {\n        author: \"me\"  # author comment\n        # email comment\n        email: \"me@stjude.org\"\n    }\n\n    parameter_meta {\n    }\n\n    input {\n    }\n\n}\n"
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
            "version 1.1\n\n# workflow comment\nworkflow test {\n    meta {\n    }\n\n    # \
             parameter_meta comment\n    parameter_meta {  # parameter_meta comment\n        foo: \
             \"bar\"  # foo comment\n    }\n\n    input {\n        String foo\n    }\n\n}\n"
        );
    }
}
