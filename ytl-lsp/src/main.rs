mod lexer;
mod parser;
mod validators;
mod hover;
mod completion;
mod code_action; // Оставляем только code_action

use tower_lsp_server::{LspService, Server};
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::*;
use tower_lsp_server::Client;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;

#[derive(Debug)]
struct Backend {
    client: Client,
    documents: Arc<Mutex<HashMap<Uri, String>>>,
}

impl tower_lsp_server::LanguageServer for Backend {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "YTL Language Server".to_string(),
                version: Some("0.3.0".to_string()),
            }),
            offset_encoding: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions::default()),
                // inlay_hint_provider: Some(OneOf::Left(true)), <--- УДАЛЕНО
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client.log_message(MessageType::INFO, "YTL Server 0.3.0 is ready!").await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;

        self.documents.lock().await.insert(uri.clone(), text.clone());

        let tokens = lexer::tokenize(&text);
        let diagnostics = parser::parse(&tokens);

        self.client.publish_diagnostics(uri, diagnostics, None).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        if let Some(change) = params.content_changes.last() {
            let uri = params.text_document.uri;
            let text = change.text.clone();

            self.documents.lock().await.insert(uri.clone(), text.clone());

            let tokens = lexer::tokenize(&text);
            let diagnostics = parser::parse(&tokens);

            self.client.publish_diagnostics(uri, diagnostics, None).await;
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let docs = self.documents.lock().await;
        if let Some(text) = docs.get(&uri) {
            if let Some(hover) = hover::get_hover(text, position) {
                return Ok(Some(hover));
            }
        }
        Ok(None)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let docs = self.documents.lock().await;
        if let Some(text) = docs.get(&uri) {
            let items = completion::get_completions(text, position);
            if !items.is_empty() {
                return Ok(Some(CompletionResponse::Array(items)));
            }
        }
        Ok(None)
    }

    // МЕТОД inlay_hint ПОЛНОСТЬЮ УДАЛЕН

    // Лампочка (автовставка полей)
    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = params.text_document.uri;
        let docs = self.documents.lock().await;
        if let Some(text) = docs.get(&uri) {
            let actions = code_action::get_code_actions(text, &uri, params.range.start);
            if !actions.is_empty() {
                return Ok(Some(actions));
            }
        }
        Ok(None)
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| Backend { 
        client,
        documents: Arc::new(Mutex::new(HashMap::new())),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}