//! Handlers for code completion requests.
//!
//! This module implements the LSP `textDocument/completion` functionality for
//! WDL files. It provides context-aware completions for various WDL language
//! constructs including:
//!
//! - Keywords appropriate to the current context (task, workflow and
//!   root-level)
//! - Variables and declarations visible in the current scope
//! - Standard library functions with signatures and documentation
//! - User-defined structs and their members
//! - Callable items (tasks and workflows) from local and imported namespaces
//! - Member access completions for struct fields, call outputs, and pair
//!   elements
//! - Import namespace identifiers
//!
//! See: https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/#textDocument_completion

use anyhow::Result;
use anyhow::bail;
use lsp_types::CompletionItem;
use lsp_types::CompletionItemKind;
use lsp_types::Documentation;
use lsp_types::MarkupContent;
use rowan::TextSize;
use url::Url;
use wdl_ast::AstNode;
use wdl_ast::SyntaxKind;
use wdl_ast::SyntaxNode;
use wdl_ast::TreeNode;
use wdl_ast::lexer::TokenSet;
use wdl_ast::lexer::v1::Token;
use wdl_ast::v1::Expr;
use wdl_ast::v1::StructDefinition;
use wdl_ast::v1::TASK_FIELD_ATTEMPT;
use wdl_ast::v1::TASK_FIELD_CONTAINER;
use wdl_ast::v1::TASK_FIELD_CPU;
use wdl_ast::v1::TASK_FIELD_DISKS;
use wdl_ast::v1::TASK_FIELD_END_TIME;
use wdl_ast::v1::TASK_FIELD_EXT;
use wdl_ast::v1::TASK_FIELD_FPGA;
use wdl_ast::v1::TASK_FIELD_GPU;
use wdl_ast::v1::TASK_FIELD_ID;
use wdl_ast::v1::TASK_FIELD_MEMORY;
use wdl_ast::v1::TASK_FIELD_META;
use wdl_ast::v1::TASK_FIELD_NAME;
use wdl_ast::v1::TASK_FIELD_PARAMETER_META;
use wdl_ast::v1::TASK_FIELD_RETURN_CODE;
use wdl_ast::v1::TASK_HINT_DISKS;
use wdl_ast::v1::TASK_HINT_FPGA;
use wdl_ast::v1::TASK_HINT_GPU;
use wdl_ast::v1::TASK_HINT_INPUTS;
use wdl_ast::v1::TASK_HINT_LOCALIZATION_OPTIONAL;
use wdl_ast::v1::TASK_HINT_MAX_CPU;
use wdl_ast::v1::TASK_HINT_MAX_MEMORY;
use wdl_ast::v1::TASK_HINT_OUTPUTS;
use wdl_ast::v1::TASK_HINT_SHORT_TASK;
use wdl_ast::v1::TASK_REQUIREMENT_CONTAINER;
use wdl_ast::v1::TASK_REQUIREMENT_CPU;
use wdl_ast::v1::TASK_REQUIREMENT_DISKS;
use wdl_ast::v1::TASK_REQUIREMENT_FPGA;
use wdl_ast::v1::TASK_REQUIREMENT_GPU;
use wdl_ast::v1::TASK_REQUIREMENT_MAX_RETRIES;
use wdl_ast::v1::TASK_REQUIREMENT_MEMORY;
use wdl_ast::v1::TASK_REQUIREMENT_RETURN_CODES;
use wdl_ast::v1::TaskDefinition;
use wdl_ast::v1::WORKFLOW_HINT_ALLOW_NESTED_INPUTS;
use wdl_ast::v1::WorkflowDefinition;
use wdl_grammar::grammar::v1::TASK_ITEM_EXPECTED_SET;
use wdl_grammar::grammar::v1::TOP_RECOVERY_SET;
use wdl_grammar::grammar::v1::TYPE_EXPECTED_SET;
use wdl_grammar::grammar::v1::WORKFLOW_ITEM_EXPECTED_SET;
use wdl_grammar::parser::ParserToken;

use crate::Document;
use crate::SourcePosition;
use crate::SourcePositionEncoding;
use crate::document::ScopeRef;
use crate::document::Struct;
use crate::document::TASK_VAR_NAME;
use crate::document::Task;
use crate::document::Workflow;
use crate::graph::DocumentGraph;
use crate::graph::ParseState;
use crate::handlers::TypeEvalContext;
use crate::handlers::position_to_offset;
use crate::stdlib::Function;
use crate::stdlib::STDLIB;
use crate::stdlib::TypeParameters;
use crate::types::CompoundType;
use crate::types::Type;
use crate::types::v1::ExprTypeEvaluator;
use crate::types::v1::task_member_type;

/// Provides code completion suggestions for the given position in a document.
///
/// Analyzes the context at the specified position and returns appropriate
/// completion items based on the surrounding syntax and scope. The completions
/// are filtered by any partial word already typed at the cursor position.
///
/// Provides context-aware suggestions by:
/// 1. Determining if the cursor is in a member access context (i.e. after a `.`
///    dot)
/// 2. Walking up the CST to find the appropriate completion context
/// 3. Adding relevant completions based on the context (keywords, scope items,
///    etc.)
/// 4. Filtering results by any partially typed identifier
pub fn completion(
    graph: &DocumentGraph,
    document_uri: &Url,
    position: SourcePosition,
    encoding: SourcePositionEncoding,
) -> Result<Vec<CompletionItem>> {
    let Some(index) = graph.get_index(document_uri) else {
        bail!("document `{document_uri}` not found in graph")
    };
    let node = graph.get(index);
    let (root, lines) = match node.parse_state() {
        ParseState::Parsed { lines, root, .. } => {
            (SyntaxNode::new_root(root.clone()), lines.clone())
        }
        _ => bail!("document `{uri}` has not been parsed", uri = document_uri),
    };

    let Some(document) = node.document() else {
        bail!("document analysis data not available for {}", document_uri);
    };

    let offset = position_to_offset(&lines, position, encoding)?;
    let token = root.token_at_offset(offset).left_biased();

    let mut items = Vec::new();

    let partial_word = token
        .as_ref()
        .filter(|t| t.kind() == SyntaxKind::Ident && t.text_range().contains_inclusive(offset))
        .map(|t| {
            let start = t.text_range().start();
            let len = offset - start;
            t.text()[..len.into()].to_string()
        });

    let parent = token
        .as_ref()
        .and_then(|t| t.parent())
        .unwrap_or_else(|| root.clone());

    // Trigger member access completions if the cursor is on a dot, or on an
    // identifier immediately following a dot.
    let is_member_access = if let Some(t) = &token {
        if t.kind() == SyntaxKind::Dot {
            true
        } else if t.kind() == SyntaxKind::Ident {
            t.prev_token()
                .filter(|prev| !prev.kind().is_trivia())
                .is_some_and(|prev| prev.kind() == SyntaxKind::Dot)
        } else {
            false
        }
    } else {
        false
    };

    if is_member_access {
        add_member_access_completions(document, &parent, &mut items)?;
    } else {
        let mut current = Some(parent);
        while let Some(node) = current {
            match node.kind() {
                SyntaxKind::WorkflowDefinitionNode => {
                    add_keyword_completions(&WORKFLOW_ITEM_EXPECTED_SET, &mut items);
                    if let Some(scope) = document.find_scope_by_position(offset.into()) {
                        add_scope_completions(scope, &mut items);
                    }
                    add_stdlib_completions(&mut items);
                    add_struct_completions(document, &mut items);
                    add_namespace_completions(document, &mut items);
                    add_callable_completions(document, &mut items);
                    break;
                }
                SyntaxKind::ScatterStatementNode | SyntaxKind::ConditionalStatementNode => {
                    const NESTED_WORKFLOW_KEYWORDS: TokenSet = TokenSet::new(&[
                        Token::CallKeyword as u8,
                        Token::ScatterKeyword as u8,
                        Token::IfKeyword as u8,
                    ]);
                    add_keyword_completions(
                        &TYPE_EXPECTED_SET.union(NESTED_WORKFLOW_KEYWORDS),
                        &mut items,
                    );
                    if let Some(scope) = document.find_scope_by_position(offset.into()) {
                        add_scope_completions(scope, &mut items);
                    }
                    add_stdlib_completions(&mut items);
                    add_struct_completions(document, &mut items);
                    add_namespace_completions(document, &mut items);
                    add_callable_completions(document, &mut items);
                    break;
                }

                SyntaxKind::TaskDefinitionNode => {
                    add_keyword_completions(&TASK_ITEM_EXPECTED_SET, &mut items);
                    if let Some(scope) = document.find_scope_by_position(offset.into()) {
                        add_scope_completions(scope, &mut items);
                    }
                    add_stdlib_completions(&mut items);
                    add_struct_completions(document, &mut items);
                    break;
                }

                SyntaxKind::StructDefinitionNode => {
                    add_struct_completions(document, &mut items);
                    add_keyword_completions(
                        &TYPE_EXPECTED_SET.union(TokenSet::new(&[
                            Token::MetaKeyword as u8,
                            Token::ParameterMetaKeyword as u8,
                        ])),
                        &mut items,
                    );
                    break;
                }

                SyntaxKind::RuntimeSectionNode => {
                    add_runtime_key_completions(&mut items);
                    break;
                }

                SyntaxKind::RequirementsSectionNode => {
                    add_requirements_key_completions(&mut items);
                    break;
                }
                SyntaxKind::TaskHintsSectionNode => {
                    add_task_hints_key_completions(&mut items);
                    break;
                }

                SyntaxKind::WorkflowHintsSectionNode => {
                    add_workflow_hints_key_completions(&mut items);
                    break;
                }

                SyntaxKind::RootNode => {
                    add_keyword_completions(
                        &TOP_RECOVERY_SET.union(TokenSet::new(&[Token::VersionKeyword as u8])),
                        &mut items,
                    );
                    add_struct_completions(document, &mut items);
                    add_namespace_completions(document, &mut items);
                    break;
                }
                _ => current = node.parent(),
            }
        }
    }

    match partial_word {
        Some(partial) => {
            let items = items
                .into_iter()
                .filter(|item| item.label.starts_with(&partial))
                .collect();
            Ok(items)
        }
        None => Ok(items),
    }
}

/// Generates completion items for WDL keywords based on the provided token set.
///
/// Converts raw token values to completion items with appropriate labels,
/// kinds, and descriptions.
fn add_keyword_completions(token_set: &TokenSet, items: &mut Vec<CompletionItem>) {
    items.extend(token_set.iter().map(|raw| {
        let token = Token::from_raw(raw);
        let label = token
            .describe()
            .trim_start_matches("`")
            .split("`")
            .next()
            .unwrap();

        CompletionItem {
            label: label.to_string(),
            kind: Some(CompletionItemKind::KEYWORD),
            ..Default::default()
        }
    }))
}

/// Adds completions for member access expressions.
///
/// Takes a syntax node containing the member access expression (parent of the
/// `.` token) and handles different types of member access completions:
///
/// - Namespace access
/// - Struct member access
/// - Call output access
/// - Pair element access (when accessing `.left` and `.right` of pair types)
///
/// For namespace access, it directly looks up the identifier before the dot.
/// For other types, it evaluates the expression type to determine available
/// members.
///
/// The node is the parent of the `.` token. For incomplete document, it might
/// not be fully-formed `AccessExprNode`. We find the expression to the left
/// of the dot.
fn add_member_access_completions(
    document: &Document,
    node: &SyntaxNode,
    items: &mut Vec<CompletionItem>,
) -> Result<()> {
    let Some(dot_token) = node
        .children_with_tokens()
        .find(|t| t.kind() == SyntaxKind::Dot)
    else {
        return Ok(());
    };

    let Some(target_element) = dot_token.prev_sibling_or_token() else {
        return Ok(());
    };

    match &target_element {
        rowan::NodeOrToken::Node(n) => {
            if n.text() == TASK_VAR_NAME
                && document.version()
                    >= Some(wdl_ast::SupportedVersion::V1(wdl_ast::version::V1::Two))
            {
                if let Some(parent) = n.parent() {
                    if matches!(
                        parent.kind(),
                        SyntaxKind::CommandSectionNode | SyntaxKind::OutputSectionNode
                    ) {
                        add_task_variable_completions(items);
                    }
                }

                return Ok(());
            }
        }
        rowan::NodeOrToken::Token(t) => {
            if t.kind() == SyntaxKind::Ident {
                if let Some(ns) = document.namespace(t.text()) {
                    let ns_root = ns.document().root();
                    for task in ns.document().tasks() {
                        items.push(CompletionItem {
                            label: task.name().to_string(),
                            kind: Some(CompletionItemKind::FUNCTION),
                            detail: Some(format!("task {}", task.name())),
                            documentation: provide_task_documentation(task, &ns_root)
                                .and_then(make_md_docs),
                            ..Default::default()
                        })
                    }

                    if let Some(workflow) = ns.document().workflow() {
                        items.push(CompletionItem {
                            label: workflow.name().to_string(),
                            kind: Some(CompletionItemKind::FUNCTION),
                            detail: Some(format!("workflow {}", workflow.name())),
                            documentation: provide_workflow_documentation(workflow, &ns_root)
                                .and_then(make_md_docs),
                            ..Default::default()
                        });
                    }

                    return Ok(());
                }
            }
        }
    }

    // NOTE: we do type evaluation only for non namespaces or complex types

    let Some(target_node) = target_element.as_node() else {
        return Ok(());
    };

    let Some(target_expr) = Expr::cast(target_node.clone()) else {
        return Ok(());
    };

    let Some(scope) = document.find_scope_by_position(node.span().start()) else {
        bail!("could not find scope for access expression")
    };

    let mut ctx = TypeEvalContext { scope, document };
    let mut evaluator = ExprTypeEvaluator::new(&mut ctx);
    let target_type = evaluator.evaluate_expr(&target_expr).unwrap_or(Type::Union);

    match target_type {
        Type::Compound(CompoundType::Struct(s), _) => {
            for (name, ty) in s.members() {
                items.push(CompletionItem {
                    label: name.to_string(),
                    kind: Some(CompletionItemKind::FIELD),
                    detail: Some(ty.to_string()),
                    ..Default::default()
                });
            }
        }
        Type::Call(call) => {
            for (name, output) in call.outputs() {
                items.push(CompletionItem {
                    label: name.to_string(),
                    kind: Some(CompletionItemKind::FIELD),
                    detail: Some(output.ty().to_string()),
                    ..Default::default()
                });
            }
        }
        Type::Compound(CompoundType::Pair(p), _) => {
            items.push(CompletionItem {
                label: "left".to_string(),
                kind: Some(CompletionItemKind::FIELD),
                detail: Some(p.left_type().to_string()),
                ..Default::default()
            });

            items.push(CompletionItem {
                label: "right".to_string(),
                kind: Some(CompletionItemKind::FIELD),
                detail: Some(p.right_type().to_string()),
                ..Default::default()
            });
        }
        _ => {}
    }

    Ok(())
}

/// Adds completions for callable items available in the current document.
///
/// Includes both local and imported tasks and workflows.
fn add_callable_completions(document: &Document, items: &mut Vec<CompletionItem>) {
    let root_node = document.root();

    for task in document.tasks() {
        items.push(CompletionItem {
            label: task.name().to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some(format!("task {}", task.name())),
            documentation: provide_task_documentation(task, &root_node).and_then(make_md_docs),
            ..Default::default()
        });
    }
    if let Some(workflow) = document.workflow() {
        items.push(CompletionItem {
            label: workflow.name().to_string(),
            kind: Some(CompletionItemKind::FUNCTION),
            detail: Some(format!("workflow {}", workflow.name())),
            documentation: provide_workflow_documentation(workflow, &root_node)
                .and_then(make_md_docs),
            ..Default::default()
        });
    }

    for (ns_name, ns) in document.namespaces() {
        let ns_root = ns.document().root();

        for task in ns.document().tasks() {
            let label = format!("{ns_name}.{}", task.name());
            items.push(CompletionItem {
                label,
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("task".to_string()),
                documentation: provide_task_documentation(task, &ns_root).and_then(make_md_docs),
                ..Default::default()
            });
        }
        if let Some(workflow) = ns.document().workflow() {
            let label = format!("{ns_name}.{}", workflow.name());
            items.push(CompletionItem {
                label,
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some("workflow".to_string()),
                documentation: provide_workflow_documentation(workflow, &ns_root)
                    .and_then(make_md_docs),
                ..Default::default()
            });
        }
    }
}

/// Adds completions for variables and declarations visible in the current
/// scope.
fn add_scope_completions(scope: ScopeRef<'_>, items: &mut Vec<CompletionItem>) {
    let mut current_scope = Some(scope);
    while let Some(s) = current_scope {
        for (name, name_info) in s.names() {
            if !items.iter().any(|i| i.label == name) {
                let (kind, detail) = match name_info.ty() {
                    Type::Call(_) => (
                        Some(CompletionItemKind::FIELD),
                        Some(format!("call output: {}", name_info.ty())),
                    ),
                    _ => (
                        Some(CompletionItemKind::VARIABLE),
                        Some(name_info.ty().to_string()),
                    ),
                };

                items.push(CompletionItem {
                    label: name.to_string(),
                    kind,
                    detail,
                    ..Default::default()
                });
            }
        }
        current_scope = s.parent();
    }
}

/// Adds completions for all WDL standard library functions.
fn add_stdlib_completions(items: &mut Vec<CompletionItem>) {
    for (name, func) in STDLIB.functions() {
        match func {
            Function::Monomorphic(m) => {
                let sig = m.signature();
                let params = TypeParameters::new(sig.type_parameters());
                let detail = Some(format!("{name}{}", sig.display(&params)));
                let docs = sig.definition().and_then(|d| make_md_docs(d.to_string()));
                items.push(CompletionItem {
                    label: name.to_string(),
                    kind: Some(CompletionItemKind::FUNCTION),
                    detail,
                    documentation: docs,
                    ..Default::default()
                })
            }
            Function::Polymorphic(p) => {
                for sig in p.signatures() {
                    let params = TypeParameters::new(sig.type_parameters());
                    let detail = Some(format!("{name}{}", sig.display(&params)));
                    let docs = sig.definition().and_then(|d| make_md_docs(d.to_string()));
                    items.push(CompletionItem {
                        label: name.to_string(),
                        kind: Some(CompletionItemKind::FUNCTION),
                        detail,
                        documentation: docs,
                        ..Default::default()
                    });
                }
            }
        };
    }
}

/// Adds completions for user-defined structs in the document.
fn add_struct_completions(document: &Document, items: &mut Vec<CompletionItem>) {
    let root = document.root();
    for (name, s) in document.structs() {
        items.push(CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::STRUCT),
            detail: Some(format!("struct {name}")),
            documentation: provide_struct_documentation(s, &root).and_then(make_md_docs),
            ..Default::default()
        })
    }
}

/// Adds completions for imported namespaces (aliases).
fn add_namespace_completions(document: &Document, items: &mut Vec<CompletionItem>) {
    for (name, _) in document.namespaces() {
        items.push(CompletionItem {
            label: name.to_string(),
            kind: Some(CompletionItemKind::MODULE),
            detail: Some(format!("import alias {name}")),
            ..Default::default()
        });
    }
}

/// Adds completions for the members of the implicit `task` variable.
fn add_task_variable_completions(items: &mut Vec<CompletionItem>) {
    const TASK_FIELDS: &[&str] = &[
        TASK_FIELD_NAME,
        TASK_FIELD_ID,
        TASK_FIELD_CONTAINER,
        TASK_FIELD_CPU,
        TASK_FIELD_MEMORY,
        TASK_FIELD_ATTEMPT,
        TASK_FIELD_GPU,
        TASK_FIELD_FPGA,
        TASK_FIELD_DISKS,
        TASK_FIELD_END_TIME,
        TASK_FIELD_RETURN_CODE,
        TASK_FIELD_META,
        TASK_FIELD_PARAMETER_META,
        TASK_FIELD_EXT,
    ];

    for field in TASK_FIELDS {
        if let Some(ty) = task_member_type(field) {
            items.push(CompletionItem {
                label: field.to_string(),
                kind: Some(CompletionItemKind::FIELD),
                detail: Some(ty.to_string()),
                ..Default::default()
            });
        }
    }
}

/// Adds completions for `runtime` section keys.
fn add_runtime_key_completions(items: &mut Vec<CompletionItem>) {
    const RUNTIME_KEYS: &[&str] = &[
        TASK_REQUIREMENT_CONTAINER,
        TASK_REQUIREMENT_CPU,
        TASK_REQUIREMENT_MEMORY,
        TASK_REQUIREMENT_DISKS,
        TASK_REQUIREMENT_GPU,
    ];

    for key in RUNTIME_KEYS {
        items.push(CompletionItem {
            label: key.to_string(),
            kind: Some(CompletionItemKind::PROPERTY),
            ..Default::default()
        });
    }
}

/// Adds completions for `requirements` section keys.
fn add_requirements_key_completions(items: &mut Vec<CompletionItem>) {
    const REQUIREMENTS_KEY: &[&str] = &[
        TASK_REQUIREMENT_CONTAINER,
        TASK_REQUIREMENT_CPU,
        TASK_REQUIREMENT_MEMORY,
        TASK_REQUIREMENT_GPU,
        TASK_REQUIREMENT_FPGA,
        TASK_REQUIREMENT_DISKS,
        TASK_REQUIREMENT_MAX_RETRIES,
        TASK_REQUIREMENT_RETURN_CODES,
    ];

    for key in REQUIREMENTS_KEY {
        items.push(CompletionItem {
            label: key.to_string(),
            kind: Some(CompletionItemKind::PROPERTY),
            ..Default::default()
        });
    }
}

/// Adds completions for `task hints` section keys.
fn add_task_hints_key_completions(items: &mut Vec<CompletionItem>) {
    const HINTS_KEY: &[&str] = &[
        TASK_HINT_DISKS,
        TASK_HINT_GPU,
        TASK_HINT_FPGA,
        TASK_HINT_INPUTS,
        TASK_HINT_LOCALIZATION_OPTIONAL,
        TASK_HINT_MAX_CPU,
        TASK_HINT_MAX_MEMORY,
        TASK_HINT_OUTPUTS,
        TASK_HINT_SHORT_TASK,
    ];

    for key in HINTS_KEY {
        items.push(CompletionItem {
            label: key.to_string(),
            kind: Some(CompletionItemKind::PROPERTY),
            ..Default::default()
        });
    }
}

/// Adds completions for `workflow hints` section keys.
fn add_workflow_hints_key_completions(items: &mut Vec<CompletionItem>) {
    const HINTS_KEY: &[&str] = &[WORKFLOW_HINT_ALLOW_NESTED_INPUTS];

    for key in HINTS_KEY {
        items.push(CompletionItem {
            label: key.to_string(),
            kind: Some(CompletionItemKind::PROPERTY),
            ..Default::default()
        });
    }
}

/// Makes a LSP documentation from a definition text.
fn make_md_docs(definition: String) -> Option<Documentation> {
    Some(Documentation::MarkupContent(MarkupContent {
        kind: lsp_types::MarkupKind::Markdown,
        value: definition,
    }))
}

/// Provides documentation for tasks which includes `inputs`, `outputs`,
/// `metadata`, `runtime`
fn provide_task_documentation(task: &Task, root: &wdl_ast::Document) -> Option<String> {
    match TextSize::try_from(task.name_span().start()) {
        Ok(offset) => root
            .inner()
            .token_at_offset(offset)
            .left_biased()
            .and_then(|t| t.parent_ancestors().find_map(TaskDefinition::cast))
            .as_ref()
            .and_then(|n| {
                let mut s = String::new();
                n.markdown_description(&mut s).ok()?;
                Some(s)
            }),
        Err(_) => None,
    }
}

/// Provides documentation for workflows which includes `inputs`, `outputs`,
/// `metadata`
fn provide_workflow_documentation(workflow: &Workflow, root: &wdl_ast::Document) -> Option<String> {
    match TextSize::try_from(workflow.name_span().start()) {
        Ok(offset) => root
            .inner()
            .token_at_offset(offset)
            .left_biased()
            .and_then(|t| t.parent_ancestors().find_map(WorkflowDefinition::cast))
            .as_ref()
            .and_then(|n| {
                let mut s = String::new();
                n.markdown_description(&mut s).ok()?;
                Some(s)
            }),
        Err(_) => None,
    }
}

/// Provides documentation for structs.
fn provide_struct_documentation(struct_info: &Struct, root: &wdl_ast::Document) -> Option<String> {
    match TextSize::try_from(struct_info.name_span().start()) {
        Ok(offset) => root
            .inner()
            .token_at_offset(offset)
            .left_biased()
            .and_then(|t| t.parent_ancestors().find_map(StructDefinition::cast))
            .as_ref()
            .and_then(|n| {
                let mut s = String::new();
                n.markdown_description(&mut s).ok()?;
                Some(s)
            }),
        Err(_) => None,
    }
}
