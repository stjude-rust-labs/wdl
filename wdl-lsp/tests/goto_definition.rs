//! Integration tests for the `textDocument/gotoDefinition` request.

mod common;
use core::panic;

use common::TestContext;
use pretty_assertions::assert_eq;
use tower_lsp::lsp_types::GotoDefinitionParams;
use tower_lsp::lsp_types::GotoDefinitionResponse;
use tower_lsp::lsp_types::Position;
use tower_lsp::lsp_types::Range;
use tower_lsp::lsp_types::TextDocumentIdentifier;
use tower_lsp::lsp_types::TextDocumentPositionParams;
use tower_lsp::lsp_types::request::GotoDefinition;

async fn goto_definition_request(
    ctx: &mut TestContext,
    path: &str,
    position: Position,
) -> Option<GotoDefinitionResponse> {
    ctx.request::<GotoDefinition>(GotoDefinitionParams {
        text_document_position_params: TextDocumentPositionParams {
            text_document: TextDocumentIdentifier {
                uri: ctx.doc_uri(path),
            },
            position,
        },
        work_done_progress_params: Default::default(),
        partial_result_params: Default::default(),
    })
    .await
}

async fn setup() -> TestContext {
    let mut ctx = TestContext::new("goto_definition");
    ctx.initialize().await;
    ctx
}

#[tokio::test]
async fn should_goto_local_variable_definition() {
    let mut ctx = setup().await;

    // Position of `name` in `call greet { input: to = name }`
    let response = goto_definition_request(&mut ctx, "source.wdl", Position::new(20, 29))
        .await
        .unwrap();

    let GotoDefinitionResponse::Scalar(location) = response else {
        panic!("expected a single location response, got {:?}", response);
    };

    assert_eq!(location.uri, ctx.doc_uri("source.wdl"));
    assert_eq!(
        location.range,
        Range::new(Position::new(16, 15), Position::new(16, 19)) // `String name`
    );
}

#[tokio::test]
async fn should_goto_local_task_definition() {
    let mut ctx = setup().await;

    // Position of `greet` in `call greet`
    let response = goto_definition_request(&mut ctx, "source.wdl", Position::new(20, 9)).await;
    let Some(GotoDefinitionResponse::Scalar(location)) = response else {
        panic!("expected a single location response, got {:?}", response);
    };

    assert_eq!(location.uri, ctx.doc_uri("source.wdl"));
    assert_eq!(
        location.range,
        Range::new(Position::new(4, 5), Position::new(4, 10)) // `task greet`
    );
}

#[tokio::test]
async fn should_goto_imported_task_definition() {
    let mut ctx = setup().await;

    // Position of `add` in `call lib.add`
    let response = goto_definition_request(&mut ctx, "source.wdl", Position::new(22, 13)).await;
    let Some(GotoDefinitionResponse::Scalar(location)) = response else {
        panic!("expected a single location response, got {:?}", response);
    };

    assert_eq!(location.uri, ctx.doc_uri("lib.wdl"));
    assert_eq!(
        location.range,
        Range::new(Position::new(2, 5), Position::new(2, 8)) // `task add` in lib.wdl
    );
}

#[tokio::test]
async fn should_goto_imported_struct_definition() {
    let mut ctx = setup().await;

    // Position of `Person` in `Person p`
    let response = goto_definition_request(&mut ctx, "source.wdl", Position::new(27, 4)).await;
    let Some(GotoDefinitionResponse::Scalar(location)) = response else {
        panic!("expected a single location response, got {:?}", response);
    };

    assert_eq!(location.uri, ctx.doc_uri("lib.wdl"));
    assert_eq!(
        location.range,
        Range::new(Position::new(17, 7), Position::new(17, 13)) // `task add` in lib.wdl
    );
}

#[tokio::test]
async fn should_goto_struct_member_definition() {
    let mut ctx = setup().await;

    // Position of `name` in `p.name`
    let response = goto_definition_request(&mut ctx, "source.wdl", Position::new(33, 22)).await;

    let Some(GotoDefinitionResponse::Scalar(location)) = response else {
        panic!("expected a single location response, got {:?}", response);
    };

    assert_eq!(location.uri, ctx.doc_uri("lib.wdl"));
    assert_eq!(
        location.range,
        Range::new(Position::new(18, 11), Position::new(18, 15)) // `String name` in Person struct
    );
}

#[tokio::test]
async fn should_goto_call_output_definition() {
    let mut ctx = setup().await;

    // Position of `result` in `t1.result`
    let response = goto_definition_request(&mut ctx, "source.wdl", Position::new(36, 24)).await;

    let Some(GotoDefinitionResponse::Scalar(location)) = response else {
        panic!("expected a single location response, got {:?}", response);
    };

    assert_eq!(location.uri, ctx.doc_uri("lib.wdl"));
    assert_eq!(
        location.range,
        Range::new(Position::new(13, 12), Position::new(13, 18)) // `Int result` in add's output
    );
}

#[tokio::test]
async fn should_goto_import_namespace_definition() {
    let mut ctx = setup().await;

    // Position of `lib` in `call lib.add`
    let response = goto_definition_request(&mut ctx, "source.wdl", Position::new(22, 9)).await;

    let Some(GotoDefinitionResponse::Scalar(location)) = response else {
        panic!("expected a single location response, got {:?}", response);
    };

    assert_eq!(location.uri, ctx.doc_uri("source.wdl"));
    assert_eq!(
        location.range,
        Range::new(Position::new(2, 20), Position::new(2, 23)) // `as lib` in import statement
    );
}
