use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use wtlang_core::{Lexer, Parser};
use std::collections::HashMap;
use tokio::sync::Mutex;

#[derive(Debug)]
struct DocumentState {
    source: String,
    version: i32,
}

pub struct WTLangServer {
    client: Client,
    documents: Mutex<HashMap<Url, DocumentState>>,
}

impl WTLangServer {
    pub fn new(client: Client) -> Self {
        WTLangServer {
            client,
            documents: Mutex::new(HashMap::new()),
        }
    }

    async fn parse_document(&self, uri: &Url) -> Option<wtlang_core::ast::Program> {
        let docs = self.documents.lock().await;
        let doc = docs.get(uri)?;
        
        let mut lexer = Lexer::new(&doc.source);
        let tokens = lexer.tokenize().ok()?;
        
        let mut parser = Parser::new(tokens);
        parser.parse().ok()
    }

    async fn publish_diagnostics(&self, uri: Url) {
        let docs = self.documents.lock().await;
        let doc = match docs.get(&uri) {
            Some(d) => d,
            None => return,
        };

        let mut diagnostics = Vec::new();
        let source = doc.source.clone();
        let version = doc.version;
        drop(docs);

        // Lexical analysis
        let mut lexer = Lexer::new(&source);
        match lexer.tokenize() {
            Ok(tokens) => {
                // Parsing
                let mut parser = Parser::new(tokens);
                if let Err(e) = parser.parse() {
                    // Parser error - create diagnostic
                    let diagnostic = Diagnostic {
                        range: Range {
                            start: Position { line: 0, character: 0 },
                            end: Position { line: 0, character: 1 },
                        },
                        severity: Some(DiagnosticSeverity::ERROR),
                        code: None,
                        code_description: None,
                        source: Some("wtlang".to_string()),
                        message: e,
                        related_information: None,
                        tags: None,
                        data: None,
                    };
                    diagnostics.push(diagnostic);
                }
            }
            Err(e) => {
                // Lexer error
                let diagnostic = Diagnostic {
                    range: Range {
                        start: Position { line: 0, character: 0 },
                        end: Position { line: 0, character: 1 },
                    },
                    severity: Some(DiagnosticSeverity::ERROR),
                    code: None,
                    code_description: None,
                    source: Some("wtlang".to_string()),
                    message: e,
                    related_information: None,
                    tags: None,
                    data: None,
                };
                diagnostics.push(diagnostic);
            }
        }

        self.client.publish_diagnostics(uri, diagnostics, Some(version)).await;
    }
}

#[tower_lsp::async_trait]
impl LanguageServer for WTLangServer {
    async fn initialize(&self, _: InitializeParams) -> Result<InitializeResult> {
        Ok(InitializeResult {
            capabilities: ServerCapabilities {
                text_document_sync: Some(TextDocumentSyncCapability::Kind(
                    TextDocumentSyncKind::FULL,
                )),
                hover_provider: Some(HoverProviderCapability::Simple(true)),
                completion_provider: Some(CompletionOptions {
                    trigger_characters: Some(vec![".".to_string(), ">".to_string()]),
                    ..Default::default()
                }),
                definition_provider: Some(OneOf::Left(true)),
                // We use push diagnostics (publish_diagnostics), not pull diagnostics
                ..Default::default()
            },
            server_info: Some(ServerInfo {
                name: "WTLang Language Server".to_string(),
                version: Some("0.1.0".to_string()),
            }),
        })
    }

    async fn initialized(&self, _: InitializedParams) {
        self.client
            .log_message(MessageType::INFO, "WTLang Language Server initialized")
            .await;
    }

    async fn shutdown(&self) -> Result<()> {
        Ok(())
    }

    async fn did_open(&self, params: DidOpenTextDocumentParams) {
        let uri = params.text_document.uri;
        let text = params.text_document.text;
        let version = params.text_document.version;

        let mut docs = self.documents.lock().await;
        docs.insert(uri.clone(), DocumentState {
            source: text,
            version,
        });
        drop(docs);

        self.publish_diagnostics(uri).await;
    }

    async fn did_change(&self, params: DidChangeTextDocumentParams) {
        let uri = params.text_document.uri;
        let version = params.text_document.version;

        if let Some(change) = params.content_changes.first() {
            let mut docs = self.documents.lock().await;
            if let Some(doc) = docs.get_mut(&uri) {
                doc.source = change.text.clone();
                doc.version = version;
            }
            drop(docs);

            self.publish_diagnostics(uri).await;
        }
    }

    async fn did_close(&self, params: DidCloseTextDocumentParams) {
        let mut docs = self.documents.lock().await;
        docs.remove(&params.text_document.uri);
    }

    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        let uri = params.text_document_position_params.text_document.uri;
        
        // For now, return basic hover info
        // TODO: Implement position-based hover using AST
        let _program = self.parse_document(&uri).await;
        
        Ok(Some(Hover {
            contents: HoverContents::Scalar(MarkedString::String(
                "WTLang - Hover information coming soon".to_string(),
            )),
            range: None,
        }))
    }

    async fn completion(&self, _params: CompletionParams) -> Result<Option<CompletionResponse>> {
        // Basic keyword completion
        let keywords = vec![
            "page", "table", "from", "display", "button", "input", "if", "else",
            "filter", "sort", "aggregate", "number", "text", "date", "boolean",
        ];

        let items: Vec<CompletionItem> = keywords
            .iter()
            .map(|kw| CompletionItem {
                label: kw.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some("WTLang keyword".to_string()),
                ..Default::default()
            })
            .collect();

        Ok(Some(CompletionResponse::Array(items)))
    }

    async fn goto_definition(
        &self,
        _params: GotoDefinitionParams,
    ) -> Result<Option<GotoDefinitionResponse>> {
        // TODO: Implement go-to-definition using AST and symbol resolution
        Ok(None)
    }
}

#[tokio::main]
async fn main() {
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| WTLangServer::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
}
