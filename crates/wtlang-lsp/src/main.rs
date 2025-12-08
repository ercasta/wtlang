use tower_lsp::jsonrpc::Result;
use tower_lsp::lsp_types::*;
use tower_lsp::{Client, LanguageServer, LspService, Server};
use wtlang_core::{Lexer, Parser, SemanticAnalyzer, Type, SymbolKind, Severity};
use std::collections::HashMap;
use tokio::sync::Mutex;

#[derive(Debug)]
struct DocumentState {
    source: String,
    version: i32,
    // Cache parsed AST and symbol table for performance
    program: Option<wtlang_core::ast::Program>,
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

    async fn parse_and_analyze(&self, uri: &Url) -> Option<(wtlang_core::ast::Program, SemanticAnalyzer)> {
        let docs = self.documents.lock().await;
        let doc = docs.get(uri)?;
        
        let mut lexer = Lexer::new(&doc.source);
        let tokens = lexer.tokenize().ok()?;
        
        let mut parser = Parser::new(tokens);
        let program = parser.parse().ok()?;
        
        let mut analyzer = SemanticAnalyzer::new();
        let _ = analyzer.analyze(&program);
        
        Some((program, analyzer))
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
        let mut diag_bag = wtlang_core::DiagnosticBag::new();
        
        match lexer.tokenize() {
            Ok(tokens) => {
                // Parsing
                let mut parser = Parser::new(tokens);
                match parser.parse() {
                    Ok(program) => {
                        // Semantic analysis
                        let mut analyzer = SemanticAnalyzer::new();
                        if let Err(sem_errors) = analyzer.analyze(&program) {
                            for err in sem_errors {
                                // Convert semantic errors to diagnostics
                                let diagnostic = Diagnostic {
                                    range: Range {
                                        start: Position { line: 0, character: 0 },
                                        end: Position { line: 0, character: 1 },
                                    },
                                    severity: Some(DiagnosticSeverity::ERROR),
                                    code: None,
                                    code_description: None,
                                    source: Some("wtlang".to_string()),
                                    message: err.to_string(),
                                    related_information: None,
                                    tags: None,
                                    data: None,
                                };
                                diagnostics.push(diagnostic);
                            }
                        }
                    }
                    Err(e) => {
                        diag_bag = e;
                    }
                }
            }
            Err(e) => {
                diag_bag = e;
            }
        }

        // Convert DiagnosticBag to LSP diagnostics
        for diag in diag_bag.diagnostics() {
            let severity = match diag.severity {
                Severity::Error => DiagnosticSeverity::ERROR,
                Severity::Warning => DiagnosticSeverity::WARNING,
                Severity::Info => DiagnosticSeverity::INFORMATION,
                Severity::Hint => DiagnosticSeverity::HINT,
            };

            let loc = &diag.location;
            let range = Range {
                start: Position {
                    line: (loc.line.saturating_sub(1)) as u32,
                    character: (loc.column.saturating_sub(1)) as u32,
                },
                end: Position {
                    line: (loc.line.saturating_sub(1)) as u32,
                    character: loc.column as u32,
                },
            };

            let lsp_diagnostic = Diagnostic {
                range,
                severity: Some(severity),
                code: Some(NumberOrString::String(format!("{:?}", diag.code))),
                code_description: None,
                source: Some("wtlang".to_string()),
                message: diag.message.clone(),
                related_information: None,
                tags: None,
                data: None,
            };
            diagnostics.push(lsp_diagnostic);
        }

        self.client.publish_diagnostics(uri, diagnostics, Some(version)).await;
    }
    
    fn get_builtin_functions() -> Vec<(&'static str, &'static str, &'static str)> {
        vec![
            ("load_csv", "load_csv(table_type, filename: string) -> table", "Load a CSV file into a table with validation"),
            ("save_csv", "save_csv(table, filename: string)", "Save a table to a CSV file"),
            ("show", "show(table, filters?: filter[]) -> table", "Display a table with optional filters"),
            ("show_editable", "show_editable(table, filters?: filter[]) -> table", "Display an editable table with optional filters"),
            ("where", "where(table, predicate: row -> bool) -> table", "Filter table rows based on a predicate"),
            ("sort", "sort(table, column: string) -> table", "Sort table by column in ascending order"),
            ("sort_desc", "sort_desc(table, column: string) -> table", "Sort table by column in descending order"),
            ("aggregate", "aggregate(table, group_by: string, agg_func: string, column: string) -> table", "Group and aggregate table data"),
            ("sum", "sum(table, column: string) -> number", "Calculate sum of a column"),
            ("average", "average(table, column: string) -> number", "Calculate average of a column"),
            ("count", "count(table) -> int", "Count rows in a table"),
            ("min", "min(table, column: string) -> number", "Find minimum value in a column"),
            ("max", "max(table, column: string) -> number", "Find maximum value in a column"),
            ("filter", "filter(column: string, mode: single|multi) -> filter", "Create a filter for table columns"),
            ("table_from", "table_from(data: array) -> table", "Create a table from array of objects"),
        ]
    }
    
    fn get_keywords() -> Vec<(&'static str, &'static str)> {
        vec![
            ("page", "Define a new page"),
            ("table", "Define a table structure"),
            ("function", "Define a function"),
            ("external", "Declare an external function"),
            ("test", "Define a test case"),
            ("let", "Declare a variable"),
            ("if", "Conditional statement"),
            ("else", "Else branch"),
            ("forall", "Loop over collection"),
            ("return", "Return from function"),
            ("button", "Create a button"),
            ("section", "Create a section"),
            ("title", "Set page title"),
            ("subtitle", "Set page subtitle"),
            ("text", "Display text"),
            ("from", "Import/reference"),
            ("in", "Used in forall loops"),
            ("int", "Integer type"),
            ("float", "Float type"),
            ("string", "String type"),
            ("date", "Date type"),
            ("currency", "Currency type"),
            ("bool", "Boolean type"),
            ("unique", "Unique constraint"),
            ("non_null", "Non-null constraint"),
            ("validate", "Validation constraint"),
            ("references", "Foreign key reference"),
            ("single", "Single-select filter mode"),
            ("multi", "Multi-select filter mode"),
        ]
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
            program: None,
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
                doc.program = None; // Invalidate cache
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
        let position = params.text_document_position_params.position;
        
        // Get document and parse
        let (_program, analyzer) = match self.parse_and_analyze(&uri).await {
            Some(result) => result,
            None => return Ok(None),
        };
        
        let docs = self.documents.lock().await;
        let doc = match docs.get(&uri) {
            Some(d) => d,
            None => return Ok(None),
        };
        
        // Get word at position
        let lines: Vec<&str> = doc.source.lines().collect();
        let line_idx = position.line as usize;
        let char_idx = position.character as usize;
        
        if line_idx >= lines.len() {
            return Ok(None);
        }
        
        let line = lines[line_idx];
        if char_idx >= line.len() {
            return Ok(None);
        }
        
        // Find word boundaries
        let start = line[..char_idx]
            .rfind(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| i + 1)
            .unwrap_or(0);
        let end = line[char_idx..]
            .find(|c: char| !c.is_alphanumeric() && c != '_')
            .map(|i| i + char_idx)
            .unwrap_or(line.len());
        
        if start >= end {
            return Ok(None);
        }
        
        let word = &line[start..end];
        
        // Look up symbol in symbol table
        let symbol_table = analyzer.get_symbol_table();
        if let Some(symbol) = symbol_table.lookup(word) {
            let type_str = match &symbol.symbol_type {
                Type::Int => "int",
                Type::Float => "float",
                Type::String => "string",
                Type::Date => "date",
                Type::Currency => "currency",
                Type::Bool => "bool",
                Type::Table(name) => &format!("table({})", name),
                Type::Filter => "filter",
                Type::Ref(table_name) => &format!("ref {}", table_name),
            };
            
            let kind_str = match symbol.kind {
                SymbolKind::Variable => "variable",
                SymbolKind::Parameter => "parameter",
                SymbolKind::LoopVariable => "loop variable",
                SymbolKind::Table => "table",
                SymbolKind::Function => "function",
                SymbolKind::ExternalFunction => "external function",
            };
            
            let hover_text = format!("**{}** `{}`\n\n*Type:* `{}`", kind_str, word, type_str);
            
            return Ok(Some(Hover {
                contents: HoverContents::Markup(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: hover_text,
                }),
                range: Some(Range {
                    start: Position { line: position.line, character: start as u32 },
                    end: Position { line: position.line, character: end as u32 },
                }),
            }));
        }
        
        // Check if it's a built-in function
        for (name, signature, doc) in Self::get_builtin_functions() {
            if name == word {
                let hover_text = format!("**built-in function** `{}`\n\n```wtlang\n{}\n```\n\n{}", name, signature, doc);
                return Ok(Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: hover_text,
                    }),
                    range: Some(Range {
                        start: Position { line: position.line, character: start as u32 },
                        end: Position { line: position.line, character: end as u32 },
                    }),
                }));
            }
        }
        
        // Check if it's a keyword
        for (name, doc) in Self::get_keywords() {
            if name == word {
                let hover_text = format!("**keyword** `{}`\n\n{}", name, doc);
                return Ok(Some(Hover {
                    contents: HoverContents::Markup(MarkupContent {
                        kind: MarkupKind::Markdown,
                        value: hover_text,
                    }),
                    range: Some(Range {
                        start: Position { line: position.line, character: start as u32 },
                        end: Position { line: position.line, character: end as u32 },
                    }),
                }));
            }
        }
        
        Ok(None)
    }

    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        let uri = params.text_document_position.text_document.uri;
        
        let mut items = Vec::new();
        
        // Add keywords
        for (kw, doc) in Self::get_keywords() {
            items.push(CompletionItem {
                label: kw.to_string(),
                kind: Some(CompletionItemKind::KEYWORD),
                detail: Some(doc.to_string()),
                documentation: Some(Documentation::String(doc.to_string())),
                ..Default::default()
            });
        }
        
        // Add built-in functions
        for (name, signature, doc) in Self::get_builtin_functions() {
            items.push(CompletionItem {
                label: name.to_string(),
                kind: Some(CompletionItemKind::FUNCTION),
                detail: Some(signature.to_string()),
                documentation: Some(Documentation::MarkupContent(MarkupContent {
                    kind: MarkupKind::Markdown,
                    value: format!("```wtlang\n{}\n```\n\n{}", signature, doc),
                })),
                insert_text: Some(format!("{}($0)", name)),
                insert_text_format: Some(InsertTextFormat::SNIPPET),
                ..Default::default()
            });
        }
        
        // Add user-defined symbols from the document
        if let Some((program, analyzer)) = self.parse_and_analyze(&uri).await {
            let symbol_table = analyzer.get_symbol_table();
            let global_scope = symbol_table.global_scope();
            
            for (name, symbol) in global_scope.symbols() {
                let (kind, detail) = match symbol.kind {
                    SymbolKind::Table => {
                        if let Type::Table(table_name) = &symbol.symbol_type {
                            (CompletionItemKind::CLASS, format!("table({})", table_name))
                        } else {
                            (CompletionItemKind::CLASS, "table".to_string())
                        }
                    }
                    SymbolKind::Function => {
                        (CompletionItemKind::FUNCTION, format!("function -> {:?}", symbol.symbol_type))
                    }
                    SymbolKind::ExternalFunction => {
                        (CompletionItemKind::FUNCTION, format!("external function -> {:?}", symbol.symbol_type))
                    }
                    SymbolKind::Variable => {
                        (CompletionItemKind::VARIABLE, format!("{:?}", symbol.symbol_type))
                    }
                    _ => (CompletionItemKind::VARIABLE, format!("{:?}", symbol.symbol_type)),
                };
                
                items.push(CompletionItem {
                    label: name.clone(),
                    kind: Some(kind),
                    detail: Some(detail.clone()),
                    ..Default::default()
                });
            }
            
            // Add table field completions if we detect we're after a dot
            // This is a simplified approach - a full implementation would parse context
            let docs = self.documents.lock().await;
            if let Some(doc) = docs.get(&uri) {
                let position = params.text_document_position.position;
                let lines: Vec<&str> = doc.source.lines().collect();
                if let Some(line) = lines.get(position.line as usize) {
                    let before_cursor = &line[..position.character.min(line.len() as u32) as usize];
                    
                    // Check if we're after a dot (field access)
                    if let Some(dot_pos) = before_cursor.rfind('.') {
                        let before_dot = before_cursor[..dot_pos].trim_end();
                        
                        // Try to find the last identifier before the dot
                        let ident_start = before_dot
                            .rfind(|c: char| !c.is_alphanumeric() && c != '_')
                            .map(|i| i + 1)
                            .unwrap_or(0);
                        let identifier = &before_dot[ident_start..];
                        
                        // Look up the identifier in symbol table
                        if let Some(symbol) = symbol_table.lookup(identifier) {
                            if let Type::Table(table_name) = &symbol.symbol_type {
                                // Find the table definition
                                for item in &program.items {
                                    if let wtlang_core::ast::ProgramItem::TableDef(table_def) = item {
                                        if &table_def.name == table_name {
                                            // Add field completions
                                            for field in &table_def.fields {
                                                items.push(CompletionItem {
                                                    label: field.name.clone(),
                                                    kind: Some(CompletionItemKind::FIELD),
                                                    detail: Some(format!("{:?}", field.field_type)),
                                                    ..Default::default()
                                                });
                                            }
                                            break;
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        
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
    // Set up basic error handling
    env_logger::init();
    
    let stdin = tokio::io::stdin();
    let stdout = tokio::io::stdout();

    let (service, socket) = LspService::new(|client| WTLangServer::new(client));
    Server::new(stdin, stdout, socket).serve(service).await;
}
