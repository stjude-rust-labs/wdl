//! Integration tests for the `textDocument/completion` request.

mod common;

use common::TestContext;
use tower_lsp::lsp_types::CompletionContext;
use tower_lsp::lsp_types::CompletionItem;
use tower_lsp::lsp_types::CompletionParams;
use tower_lsp::lsp_types::CompletionResponse;
use tower_lsp::lsp_types::CompletionTriggerKind;
use tower_lsp::lsp_types::Position;
use tower_lsp::lsp_types::TextDocumentIdentifier;
use tower_lsp::lsp_types::TextDocumentPositionParams;
use tower_lsp::lsp_types::request::Completion;

const TYPES: &[&str] = &[
    "Boolean",
    "Int",
    "Float",
    "String",
    "File",
    "Directory",
    "Array",
    "Map",
    "Object",
    "Pair",
];
const IO_KEYWORDS: &[&str] = &["input", "output"];
const META_KEYWORDS: &[&str] = &["meta", "parameter_meta"];

const TOP_LEVEL_KEYWORDS: &[&str] = &["version", "task", "workflow", "struct", "import"];

const TASK_SPECIFIC_KEYWORDS: &[&str] = &["command", "requirements", "hints", "runtime"];

const WORKFLOW_SPECIFIC_KEYWORDS: &[&str] = &["call", "scatter", "if"];

async fn completion_request(
    ctx: &mut TestContext,
    path: &str,
    position: Position,
) -> Option<CompletionResponse> {
    ctx.request::<Completion>(CompletionParams {
        text_document_position: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier {
                uri: ctx.doc_uri(path),
            },
            position,
        },
        work_done_progress_params: Default::default(),
        partial_result_params: Default::default(),
        context: Some(CompletionContext {
            trigger_kind: CompletionTriggerKind::INVOKED,
            trigger_character: None,
        }),
    })
    .await
}

fn assert_contains(items: &[CompletionItem], expected_label: &str) {
    assert!(
        items.iter().any(|item| item.label == expected_label),
        "completion items should have contained '{expected_label}'"
    );
}

fn assert_not_contains(items: &[CompletionItem], unexpected_label: &str) {
    assert!(
        !items.iter().any(|item| item.label == unexpected_label),
        "completion items should NOT have contained '{unexpected_label}'"
    );
}

fn assert_keywords(items: &[CompletionItem], keywords: &[&str], should_contain: bool) {
    for &keyword in keywords {
        if should_contain {
            assert_contains(items, keyword);
        } else {
            assert_not_contains(items, keyword);
        }
    }
}

async fn setup() -> TestContext {
    let mut ctx = TestContext::new("completions");
    ctx.initialize().await;
    ctx
}

#[tokio::test]
async fn should_complete_top_level_keywords() {
    let mut ctx = setup().await;
    let response = completion_request(&mut ctx, "source.wdl", Position::new(1, 0)).await;
    let Some(CompletionResponse::Array(items)) = response else {
        panic!("expected a response, got none");
    };

    assert_keywords(&items, TOP_LEVEL_KEYWORDS, true);
    assert_keywords(&items, TASK_SPECIFIC_KEYWORDS, false);
    assert_keywords(&items, WORKFLOW_SPECIFIC_KEYWORDS, false);
    assert_keywords(&items, TYPES, false);
    assert_keywords(&items, IO_KEYWORDS, false);
    assert_keywords(&items, META_KEYWORDS, false);

    // `stdout` is a standard library function
    assert_not_contains(&items, "stdout");
}

#[tokio::test]
async fn should_complete_workflow_keywords() {
    let mut ctx = setup().await;
    let response = completion_request(&mut ctx, "source.wdl", Position::new(13, 0)).await;
    let Some(CompletionResponse::Array(items)) = response else {
        panic!("expected a response, got none");
    };

    assert_keywords(&items, TOP_LEVEL_KEYWORDS, false);
    assert_keywords(&items, TASK_SPECIFIC_KEYWORDS, false);
    assert_keywords(&items, WORKFLOW_SPECIFIC_KEYWORDS, true);
    assert_keywords(&items, TYPES, true);
    assert_keywords(&items, IO_KEYWORDS, true);
    assert_keywords(&items, META_KEYWORDS, true);

    // `stdout` is a standard library function
    assert_contains(&items, "stdout");
}

#[tokio::test]
async fn should_complete_task_keywords() {
    let mut ctx = setup().await;
    let response = completion_request(&mut ctx, "lib.wdl", Position::new(3, 0)).await;
    let Some(CompletionResponse::Array(items)) = response else {
        panic!("expected a response, got none");
    };

    assert_keywords(&items, TOP_LEVEL_KEYWORDS, false);
    assert_keywords(&items, TASK_SPECIFIC_KEYWORDS, true);
    assert_keywords(&items, WORKFLOW_SPECIFIC_KEYWORDS, false);
    assert_keywords(&items, TYPES, true);
    assert_keywords(&items, IO_KEYWORDS, true);
    assert_keywords(&items, META_KEYWORDS, true);

    // `stdout` is a standard library function
    assert_contains(&items, "stdout");
}

#[tokio::test]
async fn should_complete_struct_keywords() {
    let mut ctx = setup().await;
    let response = completion_request(&mut ctx, "source.wdl", Position::new(9, 0)).await;
    let Some(CompletionResponse::Array(items)) = response else {
        panic!("expected a response, got none");
    };

    assert_keywords(&items, TOP_LEVEL_KEYWORDS, false);
    assert_keywords(&items, TASK_SPECIFIC_KEYWORDS, false);
    assert_keywords(&items, WORKFLOW_SPECIFIC_KEYWORDS, false);
    assert_keywords(&items, TYPES, true);
    assert_keywords(&items, IO_KEYWORDS, false);
    assert_keywords(&items, META_KEYWORDS, true);

    // `stdout` is a standard library function
    assert_not_contains(&items, "stdout");
}

#[tokio::test]
async fn should_complete_struct_members_access() {
    let mut ctx = setup().await;

    // Position of cursor `String n = my_foo.`
    let response = completion_request(&mut ctx, "source.wdl", Position::new(21, 22)).await;
    let Some(CompletionResponse::Array(items)) = response else {
        panic!("expected a response, got none");
    };

    assert_eq!(items.len(), 1, "should only complete the single member");
    assert_contains(&items, "bar");
    assert_not_contains(&items, "baz");
}

#[tokio::test]
async fn should_complete_with_partial_word() {
    let mut ctx = setup().await;
    // Position of cursor at `Int out = qux.n`
    let response = completion_request(&mut ctx, "partial.wdl", Position::new(13, 23)).await;
    let Some(CompletionResponse::Array(items)) = response else {
        panic!("expected a response, got none");
    };

    assert_eq!(items.len(), 1, "should only have a single item");
    assert_contains(&items, "num");
}

#[tokio::test]
async fn should_complete_namespace_members() {
    let mut ctx = setup().await;
    // Position of cursor at `call lib.`
    let response = completion_request(&mut ctx, "namespaces.wdl", Position::new(5, 13)).await;
    let Some(CompletionResponse::Array(items)) = response else {
        panic!("expected a response, got none");
    };

    assert_eq!(items.len(), 1);
    assert_contains(&items, "greet");
}

#[tokio::test]
async fn should_complete_scope_variables() {
    let mut ctx = setup().await;

    // Workflow scope
    let response = completion_request(&mut ctx, "scopes.wdl", Position::new(10, 0)).await;
    let Some(CompletionResponse::Array(items)) = response else {
        panic!("expected a response, got none");
    };

    // Struct
    assert_contains(&items, "Person");
    // task from current file
    assert_contains(&items, "A");
    // task from imported file
    assert_contains(&items, "lib.greet");
    // Namespace
    assert_contains(&items, "lib");
    // Stdlib function
    assert_contains(&items, "floor");
    assert_contains(&items, "min");
    assert_contains(&items, "stdout");
    assert_contains(&items, "stderr");

    // Workflow specific keywords
    assert_contains(&items, "call");
    assert_contains(&items, "hints");
    assert_contains(&items, "input");
    assert_contains(&items, "output");
    assert_contains(&items, "meta");
    assert_contains(&items, "parameter_meta");
    assert_not_contains(&items, "runtime");
    assert_not_contains(&items, "requirements");

    // Task scope
    let response = completion_request(&mut ctx, "scopes.wdl", Position::new(17, 0)).await;
    let Some(CompletionResponse::Array(items)) = response else {
        panic!("expected a response, got none");
    };

    // Variable
    assert_contains(&items, "number");
    // Struct
    assert_contains(&items, "Person");
    // Stdlib function
    assert_contains(&items, "floor");

    // Task specific keywords
    assert_contains(&items, "hints");
    assert_contains(&items, "input");
    assert_contains(&items, "output");
    assert_contains(&items, "meta");
    assert_contains(&items, "parameter_meta");
    assert_contains(&items, "runtime");
    assert_contains(&items, "requirements");
    assert_not_contains(&items, "call");
}
