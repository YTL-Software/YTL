mod lexer;
mod validators;
mod hover;
mod completion;
mod code_action;
mod syntax;
mod cst_parser;
mod cst_validator;
mod inlay_hints;
mod symbols;
mod fs_utils;

use tower_lsp_server::{LspService, Server};
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::*;
use tower_lsp_server::Client;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::path::PathBuf;

#[derive(Debug)]
struct Backend {
    client: Client,
    documents: Arc<Mutex<HashMap<Uri, String>>>,
    root_path: Arc<Mutex<Option<PathBuf>>>,
}

impl tower_lsp_server::LanguageServer for Backend {
    async fn initialize(&self, params: InitializeParams) -> Result<InitializeResult> {
        let mut root_path: Option<PathBuf> = None;
        
        if let Some(folders) = &params.workspace_folders {
            if let Some(first) = folders.first() {
                if let Some(path) = first.uri.to_file_path() {
                    root_path = Some(path.into_owned());
                }
            }
        }

        {
            let mut root = self.root_path.lock().await;
            *root = root_path;
        }

        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "YTL Language Server".to_string(),
                version: Some("0.4.0".to_string()),
            }),
            offset_encoding: None,
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Options(
                    TextDocumentSyncOptions {
                        open_close: Some(true),
                        change: Some(TextDocumentSyncKind::FULL),
                        save: Some(TextDocumentSyncSaveOptions::SaveOptions(SaveOptions {
                            include_text: Some(true),
                        })),
                        ..Default::default()
                    }
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![
                        "{".to_string(),
                        ",".to_string(),
                        " ".to_string(),
                        "/".to_string(),
                    ]),
                    ..Default::default()
                }),
                code_action_provider: Some(CodeActionProviderCapability::Simple(true)),
                inlay_hint_provider: Some(OneOf::Left(true)),
                ..ServerCapabilities::default()
            },
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client.log_message(MessageType::INFO, "YTL Server 0.4.0 (CST) is ready").await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        self.client.log_message(MessageType::INFO, "did_open called!").await;
        let uri = params.text_document.uri;
        let text = params.text_document.text;

        {
            let mut docs = self.documents.lock().await;
            docs.insert(uri.clone(), text.clone());
        }

        let root_path_opt = {
            let root = self.root_path.lock().await;
            root.clone()
        };

        let diagnostics = {
            let tokens = lexer::tokenize(&text);
            let cst = cst_parser::parse_to_cst(&tokens);
            let syntax_node = rowan::SyntaxNode::<crate::syntax::YtlLanguage>::new_root(cst);
            cst_validator::validate(&syntax_node, &text, &uri, root_path_opt.as_deref())
        };

        self.client.publish_diagnostics(uri, diagnostics, None).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        self.client.log_message(MessageType::INFO, "did_change called!").await;
        if let Some(change) = params.content_changes.last() {
            let uri = params.text_document.uri;
            let text = change.text.clone();

            {
                let mut docs = self.documents.lock().await;
                docs.insert(uri.clone(), text.clone());
            }

            let root_path_opt = {
                let root = self.root_path.lock().await;
                root.clone()
            };

            let diagnostics = {
                let tokens = lexer::tokenize(&text);
                let cst = cst_parser::parse_to_cst(&tokens);
                let syntax_node = rowan::SyntaxNode::<crate::syntax::YtlLanguage>::new_root(cst);
                cst_validator::validate(&syntax_node, &text, &uri, root_path_opt.as_deref())
            };

            self.client.publish_diagnostics(uri, diagnostics, None).await;
        }
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        let position = params.text_document_position_params.position;

        let docs = self.documents.lock().await;
        if let Some(text) = docs.get(&uri) {
            let tokens = lexer::tokenize(text);
            let cst = cst_parser::parse_to_cst(&tokens);
            let syntax_node = rowan::SyntaxNode::<crate::syntax::YtlLanguage>::new_root(cst);

            if let Some(hover) = hover::get_hover(&syntax_node, position) {
                return Ok(Some(hover));
            }
        }
        Ok(None)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        let position = params.text_document_position.position;

        let root_path_opt = {
            let root = self.root_path.lock().await;
            root.clone()
        };
        let root_path_ref = root_path_opt.as_deref();

        let docs = self.documents.lock().await;
        if let Some(text) = docs.get(&uri) {
            let tokens = lexer::tokenize(text);
            let cst = cst_parser::parse_to_cst(&tokens);
            let syntax_node = rowan::SyntaxNode::<crate::syntax::YtlLanguage>::new_root(cst);

            let items = completion::get_completions(&syntax_node, position, text, &uri, root_path_ref);
            if !items.is_empty() {
                return Ok(Some(CompletionResponse::Array(items)));
            }
        }
        Ok(None)
    }

    async fn code_action(&self, params: CodeActionParams) -> Result<Option<CodeActionResponse>> {
        let uri = params.text_document.uri;

        let docs = self.documents.lock().await;
        
        if let Some(text) = docs.get(&uri) {
            let tokens = lexer::tokenize(text);
            let cst = cst_parser::parse_to_cst(&tokens);
            let syntax_node = rowan::SyntaxNode::<crate::syntax::YtlLanguage>::new_root(cst);

            let actions = code_action::get_code_actions(&syntax_node, text, &uri, params.range.start);
            
            if !actions.is_empty() {
                return Ok(Some(actions));
            }
        }
        Ok(None)
    }

    async fn inlay_hint(&self, params: InlayHintParams) -> Result<Option<Vec<InlayHint>>> {
        let uri = params.text_document.uri;

        let root_path_opt = {
            let root = self.root_path.lock().await;
            root.clone()
        };
        let root_path_ref = root_path_opt.as_deref();

        let docs = self.documents.lock().await;
        
        if let Some(text) = docs.get(&uri) {
            let tokens = lexer::tokenize(text);
            let cst = cst_parser::parse_to_cst(&tokens);
            let syntax_node = rowan::SyntaxNode::<crate::syntax::YtlLanguage>::new_root(cst);

            let hints = inlay_hints::get_inlay_hints(&syntax_node, text, params.range, &uri, root_path_ref);
            
            if !hints.is_empty() {
                return Ok(Some(hints));
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
        root_path: Arc::new(Mutex::new(None)),
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}