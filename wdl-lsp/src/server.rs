//! Implementation of the LSP server.

use std::ffi::OsStr;
use std::mem;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use notification::Progress;
use parking_lot::RwLock;
use request::WorkDoneProgressCreate;
use serde_json::to_value;
use tower_lsp::jsonrpc::Error as RpcError;
use tower_lsp::jsonrpc::ErrorCode;
use tower_lsp::jsonrpc::Result as RpcResult;
use tower_lsp::lsp_types::*;
use tower_lsp::Client;
use tower_lsp::LanguageServer;
use tower_lsp::LspService;
use uuid::Uuid;
use wdl_analysis::Analyzer;
use wdl_analysis::IncrementalChange;
use wdl_analysis::SourceEdit;
use wdl_analysis::SourcePosition;
use wdl_analysis::SourcePositionEncoding;
use wdl_ast::Validator;
use wdl_lint::LintVisitor;

use crate::proto;

/// LSP features supported by the client.
#[derive(Clone, Copy, Debug, Default)]
struct ClientSupport {
    /// Whether or not the client supports dynamic registration of watched
    /// files.
    pub watched_files: bool,
    /// Whether or not the client supports pull diagnostics (workspace and text
    /// document).
    pub pull_diagnostics: bool,
    /// Whether or not the client supports registering work done progress
    /// tokens.
    pub work_done_progress: bool,
}

impl ClientSupport {
    /// Creates a new client features from the given client capabilities.
    pub fn new(capabilities: &ClientCapabilities) -> Self {
        Self {
            watched_files: capabilities
                .workspace
                .as_ref()
                .map(|c| {
                    c.did_change_watched_files
                        .as_ref()
                        .map(|c| c.dynamic_registration == Some(true))
                        .unwrap_or(false)
                })
                .unwrap_or(false),
            pull_diagnostics: capabilities
                .text_document
                .as_ref()
                .map(|c| c.diagnostic.is_some())
                .unwrap_or(false),
            work_done_progress: capabilities
                .window
                .as_ref()
                .map(|c| c.work_done_progress == Some(true))
                .unwrap_or(false),
        }
    }
}

/// Represents a progress token for displaying work progress in the client.
#[derive(Debug, Clone, Default)]
struct ProgressToken(Option<String>);

impl ProgressToken {
    /// Constructs a new progress token.
    ///
    /// If progress tokens aren't supported by the client, this will return a
    /// no-op token.
    pub async fn new(client: &Client, client_supported: bool) -> Self {
        if !client_supported {
            return Self(None);
        }

        let token = Uuid::new_v4().to_string();
        if client
            .send_request::<WorkDoneProgressCreate>(WorkDoneProgressCreateParams {
                token: NumberOrString::String(token.clone()),
            })
            .await
            .is_err()
        {
            return Self(None);
        }

        Self(Some(token))
    }

    /// Starts the work progress.
    pub async fn start(
        &self,
        client: &Client,
        title: impl Into<String>,
        message: impl Into<String>,
    ) {
        if let Some(token) = &self.0 {
            client
                .send_notification::<Progress>(ProgressParams {
                    token: NumberOrString::String(token.clone()),
                    value: ProgressParamsValue::WorkDone(WorkDoneProgress::Begin(
                        WorkDoneProgressBegin {
                            title: title.into(),
                            cancellable: None,
                            message: Some(message.into()),
                            percentage: Some(0),
                        },
                    )),
                })
                .await;
        }
    }

    /// Updates the work progress.
    pub async fn update(&self, client: &Client, message: impl Into<String>, percentage: u32) {
        if let Some(token) = &self.0 {
            client
                .send_notification::<Progress>(ProgressParams {
                    token: NumberOrString::String(token.clone()),
                    value: ProgressParamsValue::WorkDone(WorkDoneProgress::Report(
                        WorkDoneProgressReport {
                            cancellable: None,
                            message: Some(message.into()),
                            percentage: Some(percentage),
                        },
                    )),
                })
                .await;
        }
    }

    /// Completes the work progress.
    pub async fn complete(self, client: &Client, message: impl Into<String>) {
        if let Some(token) = self.0 {
            client
                .send_notification::<Progress>(ProgressParams {
                    token: NumberOrString::String(token),
                    value: ProgressParamsValue::WorkDone(WorkDoneProgress::End(
                        WorkDoneProgressEnd {
                            message: Some(message.into()),
                        },
                    )),
                })
                .await;
        }
    }
}

/// Represents options for running the LSP server.
#[derive(Debug, Default)]
pub struct ServerOptions {
    /// The name of the server.
    ///
    /// Defaults to `wdl-lsp` crate name.
    pub name: Option<String>,

    /// The version of the server.
    ///
    /// Defaults to the version of the `wdl-lsp` crate.
    pub version: Option<String>,

    /// Whether or not linting is enabled.
    pub lint: bool,
}

/// Represents an LSP server for analyzing WDL documents.
#[derive(Debug)]
pub struct Server {
    /// The LSP client connected to the server.
    client: Client,
    /// The options for the server.
    options: ServerOptions,
    /// The analyzer used to analyze documents.
    analyzer: Analyzer<ProgressToken>,
    /// The features supported by the LSP client.
    client_support: Arc<RwLock<ClientSupport>>,
    /// The current set of workspace folders.
    folders: Arc<RwLock<Vec<WorkspaceFolder>>>,
}

impl Server {
    /// Runs the server until a request is received to shut down.
    pub async fn run(options: ServerOptions) -> Result<()> {
        log::debug!("running LSP server: {options:#?}");

        let (service, socket) = LspService::new(|client| {
            let lint = options.lint;
            let analyzer_client = client.clone();

            Self {
                client,
                options,
                analyzer: Analyzer::<ProgressToken>::new_with_validator(
                    move |token, kind, current, total| {
                        let client = analyzer_client.clone();
                        async move {
                            let message = format!(
                                "{kind} {current}/{total} file{s}",
                                s = if total > 1 { "s" } else { "" }
                            );
                            let percentage = ((current * 100) as f64 / total as f64) as u32;
                            token.update(&client, message, percentage).await
                        }
                    },
                    move || {
                        let mut validator = Validator::default();
                        if lint {
                            validator.add_visitor(LintVisitor::default());
                        }

                        validator
                    },
                ),
                client_support: Default::default(),
                folders: Default::default(),
            }
        });

        let stdin = tokio::io::stdin();
        let stdout = tokio::io::stdout();
        tower_lsp::Server::new(stdin, stdout, socket)
            .serve(service)
            .await;

        Ok(())
    }

    /// Gets the name of the server.
    fn name(&self) -> &str {
        self.options
            .name
            .as_deref()
            .unwrap_or(env!("CARGO_CRATE_NAME"))
    }

    /// Gets the version of the server.
    fn version(&self) -> &str {
        self.options
            .version
            .as_deref()
            .unwrap_or(env!("CARGO_PKG_VERSION"))
    }

    /// Registers a generic watcher for all files/directories in the workspace.
    async fn register_watcher(&self) {
        self.client
            .register_capability(vec![Registration {
                id: Uuid::new_v4().to_string(),
                method: "workspace/didChangeWatchedFiles".into(),
                register_options: Some(
                    to_value(DidChangeWatchedFilesRegistrationOptions {
                        watchers: vec![FileSystemWatcher {
                            // We use a generic glob so we can be notified for when directories,
                            // which might contain WDL documents, are deleted
                            glob_pattern: GlobPattern::String("**/*".to_string()),
                            kind: None,
                        }],
                    })
                    .expect("should convert to value"),
                ),
            }])
            .await
            .expect("failed to register capabilities with client");
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for Server {
    async fn initialize(&self, params: InitializeParams) -> RpcResult<InitializeResult> {
        log::debug!("received `initialize` request: {params:#?}");

        {
            let mut folders = self.folders.write();
            *folders = params
                .workspace_folders
                .unwrap_or_default()
                .into_iter()
                .collect();
        }

        {
            let mut client_support = self.client_support.write();
            *client_support = ClientSupport::new(&params.capabilities);

            if !client_support.pull_diagnostics {
                return Err(RpcError {
                    code: ErrorCode::ServerError(0),
                    message: "LSP server currently requires support for pulling diagnostics".into(),
                    data: None,
                });
            }
        }

        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::INCREMENTAL),
                        ..Default::default()
                    },
                )),
                workspace: Some(WorkspaceServerCapabilities {
                    workspace_folders: Some(WorkspaceFoldersServerCapabilities {
                        supported: Some(true),
                        change_notifications: Some(OneOf::Left(true)),
                    }),
                    ..Default::default()
                }),
                diagnostic_provider: Some(DiagnosticServerCapabilities::Options(
                    DiagnosticOptions {
                        inter_file_dependencies: true,
                        workspace_diagnostics: true,
                        // Intentionally disabled as currently VS code doesn't send a work done
                        // token on the diagnostic requests, only one for partial results; instead,
                        // we'll use a token created by the server to report progress.
                        // work_done_progress_options: WorkDoneProgressOptions {
                        //     work_done_progress: Some(true),
                        // },
                        ..Default::default()
                    },
                )),
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: self.name().to_string(),
                version: Some(self.version().to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        if self.client_support.read().watched_files {
            self.register_watcher().await;
        }

        // Process the initial workspace folders
        let folders = {
            let mut folders = self.folders.write();
            mem::take(&mut *folders)
        };

        if !folders.is_empty() {
            self.did_change_workspace_folders(DidChangeWorkspaceFoldersParams {
                event: WorkspaceFoldersChangeEvent {
                    added: folders,
                    removed: Vec::new(),
                },
            })
            .await;
        }

        log::info!(
            "{name} (v{version}) server initialized",
            name = self
                .options
                .name
                .as_deref()
                .unwrap_or(env!("CARGO_CRATE_NAME")),
            version = self
                .options
                .version
                .as_deref()
                .unwrap_or(env!("CARGO_PKG_VERSION"))
        );
    }

    async fn shutdown(&self) -> RpcResult<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        log::debug!("received `textDocument/didOpen` request: {params:#?}");

        if let Ok(path) = params.text_document.uri.to_file_path() {
            if let Err(e) = self.analyzer.add_documents(vec![path]).await {
                log::error!(
                    "failed to add document {uri}: {e}",
                    uri = params.text_document.uri
                );
                return;
            }

            if let Err(e) = self.analyzer.notify_incremental_change(
                params.text_document.uri,
                IncrementalChange {
                    version: params.text_document.version,
                    start: Some(params.text_document.text),
                    edits: Vec::new(),
                },
            ) {
                log::error!("failed to notify incremental change: {e}");
            }
        }
    }

    async fn did_change(&self, mut params: DidChangeTextDocumentParams) {
        log::debug!("received `textDocument/didChange` request: {params:#?}");

        log::debug!(
            "document `{uri}` is now client version {version}",
            uri = params.text_document.uri,
            version = params.text_document.version
        );

        // Look for the last full change (one without a range) and start there
        let (start, changes) = match params
            .content_changes
            .iter()
            .rposition(|change| change.range.is_none())
        {
            Some(idx) => (
                Some(mem::take(&mut params.content_changes[idx].text)),
                &mut params.content_changes[idx + 1..],
            ),
            None => (None, &mut params.content_changes[..]),
        };

        // Notify the analyzer that the document has changed
        if let Err(e) = self.analyzer.notify_incremental_change(
            params.text_document.uri,
            IncrementalChange {
                version: params.text_document.version,
                start,
                edits: changes
                    .iter_mut()
                    .map(|e| {
                        let range = e.range.expect("edit should be after the last full change");
                        SourceEdit::new(
                            SourcePosition::new(range.start.line, range.start.character)
                                ..SourcePosition::new(range.end.line, range.end.character),
                            SourcePositionEncoding::UTF16,
                            mem::take(&mut e.text),
                        )
                    })
                    .collect(),
            },
        ) {
            log::error!("failed to notify incremental change: {e}");
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        log::debug!("received `textDocument/didClose` request: {params:#?}");
        if let Err(e) = self.analyzer.notify_change(params.text_document.uri, true) {
            log::error!("failed to notify change: {e}");
        }
    }

    async fn diagnostic(
        &self,
        params: DocumentDiagnosticParams,
    ) -> RpcResult<DocumentDiagnosticReportResult> {
        log::debug!("received `textDocument/diagnostic` request: {params:#?}");

        let results: Vec<wdl_analysis::AnalysisResult> = self
            .analyzer
            .analyze_document(ProgressToken::default(), params.text_document.uri.clone())
            .await
            .map_err(|e| RpcError {
                code: ErrorCode::InternalError,
                message: e.to_string().into(),
                data: None,
            })?;

        proto::document_diagnostic_report(params, results, self.name())
            .ok_or_else(RpcError::request_cancelled)
    }

    async fn workspace_diagnostic(
        &self,
        params: WorkspaceDiagnosticParams,
    ) -> RpcResult<WorkspaceDiagnosticReportResult> {
        log::debug!("received `workspace/diagnostic` request: {params:#?}");

        let work_done_progress = self.client_support.read().work_done_progress;
        let progress = ProgressToken::new(&self.client, work_done_progress).await;
        progress
            .start(&self.client, self.name(), "analyzing...")
            .await;
        let results = self
            .analyzer
            .analyze(progress.clone())
            .await
            .map_err(|e| RpcError {
                code: ErrorCode::InternalError,
                message: e.to_string().into(),
                data: None,
            })?;
        progress.complete(&self.client, "analysis complete").await;

        Ok(proto::workspace_diagnostic_report(
            params,
            results,
            self.name(),
        ))
    }

    async fn did_change_workspace_folders(&self, params: DidChangeWorkspaceFoldersParams) {
        log::debug!("received `workspace/didChangeWorkspaceFolders` request: {params:#?}");

        // Process the removed folders
        if !params.event.removed.is_empty() {
            if let Err(e) = self
                .analyzer
                .remove_documents(params.event.removed.into_iter().map(|f| f.uri).collect())
                .await
            {
                log::error!("failed to remove documents from analyzer: {e}");
            }
        }

        // Progress the added folders
        if !params.event.added.is_empty() {
            if let Err(e) = self
                .analyzer
                .add_documents(
                    params
                        .event
                        .added
                        .iter()
                        .filter_map(|f| f.uri.to_file_path().ok())
                        .collect(),
                )
                .await
            {
                log::error!("failed to add documents to analyzer: {e}");
            }
        }
    }

    async fn did_change_watched_files(&self, params: DidChangeWatchedFilesParams) {
        log::debug!("received `workspace/didChangeWatchedFiles` request: {params:#?}");

        /// Converts a URI into a WDL file path.
        fn to_wdl_file_path(uri: &Url) -> Option<PathBuf> {
            if let Ok(path) = uri.to_file_path() {
                if path.is_file() && path.extension().and_then(OsStr::to_str) == Some("wdl") {
                    return Some(path);
                }
            }

            None
        }

        let mut added = Vec::new();
        let mut deleted = Vec::new();

        for event in params.changes {
            match event.typ {
                FileChangeType::CREATED => {
                    if let Some(path) = to_wdl_file_path(&event.uri) {
                        log::debug!("document `{uri}` has been created", uri = event.uri);
                        added.push(path);
                    }
                }
                FileChangeType::CHANGED => {
                    if to_wdl_file_path(&event.uri).is_some() {
                        log::debug!("document `{uri}` has been changed", uri = event.uri);
                        if let Err(e) = self.analyzer.notify_change(event.uri, false) {
                            log::error!("failed to notify change: {e}");
                        }
                    }
                }
                FileChangeType::DELETED => {
                    if to_wdl_file_path(&event.uri).is_some() {
                        log::debug!("document `{uri}` has been deleted", uri = event.uri);
                        deleted.push(event.uri);
                    }
                }
                _ => continue,
            }
        }

        // Add any documents to the analyzer
        if !added.is_empty() {
            if let Err(e) = self.analyzer.add_documents(added).await {
                log::error!("failed to add documents to analyzer: {e}");
            }
        }

        // Remove any documents from the analyzer
        if !deleted.is_empty() {
            if let Err(e) = self.analyzer.remove_documents(deleted).await {
                log::error!("failed to remove documents from analyzer: {e}");
            }
        }
    }
}
