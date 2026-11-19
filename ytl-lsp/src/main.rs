// Модули, которые остались в ytl-lsp:
mod hover;
mod completion;
mod code_action;
mod inlay_hints;

// Импортируем всё из ytl-core:
use ytl_core::{
    lexer, cst_parser, syntax, locales, cst_validator, fs_utils, symbols
};

use tower_lsp_server::{LspService, Server};
use tower_lsp_server::jsonrpc::Result;
use tower_lsp_server::ls_types::*;
use tower_lsp_server::Client;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::Mutex;
use std::path::{Path, PathBuf};

#[derive(Debug)]
struct Backend {
    client: Client,
    documents: Arc<Mutex<HashMap<Uri, String>>>,
    root_path: Arc<Mutex<Option<PathBuf>>>,
    language: Arc<Mutex<String>>, 
}

impl Backend {
    fn parse_document(text: &str) -> crate::syntax::SyntaxNode{
        let tokens = lexer::tokenize(text);
        let cst = cst_parser::parse_to_cst(&tokens);
        rowan::SyntaxNode::<crate::syntax::YtlLanguage>::new_root(cst)
    }
    // Рекурсивный обход директории и проверка файлов
    async fn scan_workspace(&self, dir: PathBuf) {
        let client = self.client.clone();
        let root_path = dir.clone();
        let locale = {
            let lang = self.language.lock().await;
            locales::load_locale(&lang)
        };
        
        tokio::task::spawn_blocking(move || {
            let mut paths = Vec::new();
            Self::collect_ytl_files(&dir, &mut paths);
            
            for path in paths {
                if let Ok(content) = std::fs::read_to_string(&path) {
                    let uri = Uri::from_file_path(&path).unwrap_or_else(|| "file:///unknown".parse().unwrap());
                    
                    let tokens = lexer::tokenize(&content);
                    let cst = cst_parser::parse_to_cst(&tokens);
                    let syntax_node = rowan::SyntaxNode::<crate::syntax::YtlLanguage>::new_root(cst);
                    // Передаем locale
                    let diagnostics = cst_validator::validate(&syntax_node, &content, &uri, Some(&root_path), &locale);
                    
                    let client_clone = client.clone();
                    let uri_clone = uri.clone();
                    tokio::spawn(async move {
                        client_clone.publish_diagnostics(uri_clone, diagnostics, None).await;
                    });
                }
            }
        }).await.unwrap();
    }

    fn collect_ytl_files(dir: &Path, paths: &mut Vec<PathBuf>) {
        if let Ok(entries) = std::fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.is_dir() {
                    Self::collect_ytl_files(&path, paths);
                } else if path.extension().map_or(false, |ext| ext == "ytl") {
                    paths.push(path);
                }
            }
        }
    }
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
        } else {
            #[allow(deprecated)]
            if let Some(root_uri) = &params.root_uri {
                if let Some(path) = root_uri.to_file_path() {
                    root_path = Some(path.into_owned());
                }
            }
        }

        {
            let mut root = self.root_path.lock().await;
            *root = root_path;
        }

        // --- ЧТЕНИЕ ЯЗЫКА ИЗ VS CODE ---
        let mut lang = "en";
        if let Some(options) = &params.initialization_options {
            if let Some(lang_opt) = options.get("ytl_language").and_then(|v| v.as_str()) {
                lang = lang_opt;
            }
        }
        {
            let mut current_lang = self.language.lock().await;
            *current_lang = lang.to_string();
        }

        Ok(InitializeResult {
            server_info: Some(ServerInfo {
                name: "YTL Language Server".to_string(),
                version: Some("0.4.1".to_string()), // Обновил версию!
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
        self.client.log_message(MessageType::INFO, "YTL Server 0.4.1 (CST) is ready").await;
        
        let root_path_opt = {
            let root = self.root_path.lock().await;
            root.clone()
        };
        
        if let Some(root) = root_path_opt {
            self.client.log_message(MessageType::INFO, format!("Scanning workspace: {:?}", root)).await;
            self.scan_workspace(root).await;
        }
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

        let locale = {
            let lang = self.language.lock().await;
            locales::load_locale(&lang)
        };

        let diagnostics = {
            let syntax_node = Self::parse_document(&text);
            cst_validator::validate(&syntax_node, &text, &uri, root_path_opt.as_deref(), &locale)
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

            let locale = {
                let lang = self.language.lock().await;
                locales::load_locale(&lang)
            };

            let diagnostics = {
                let syntax_node = Self::parse_document(&text);
                cst_validator::validate(&syntax_node, &text, &uri, root_path_opt.as_deref(), &locale)
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
            let syntax_node = Self::parse_document(&text);

            let items = completion::get_completions(&syntax_node, position, text.as_str(), &uri, root_path_ref);
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
            let syntax_node = Self::parse_document(&text);

            let actions = code_action::get_code_actions(&syntax_node, text.as_str(), &uri, params.range.start);
            
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
            let syntax_node = Self::parse_document(&text);

            let hints = inlay_hints::get_inlay_hints(&syntax_node, text.as_str(), params.range, &uri, root_path_ref);
            
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
        language: Arc::new(Mutex::new("en".to_string())), // <-- Инициализация
    });
    Server::new(stdin, stdout, socket).serve(service).await;
}