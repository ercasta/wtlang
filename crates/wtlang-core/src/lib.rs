// WTLang Core Library
// Shared components for compiler, LSP, and other tools

pub mod lexer;
pub mod ast;
pub mod parser;
pub mod symbols;
pub mod semantics;
pub mod errors;

// Re-export commonly used types
pub use lexer::{Lexer, Token, TokenType};
pub use ast::*;
pub use parser::Parser;
pub use symbols::{Symbol, SymbolTable, SymbolKind, SymbolError, ScopeKind};
pub use semantics::{SemanticAnalyzer, SemanticError};
pub use errors::{ErrorCode, Diagnostic, DiagnosticBag, Location, Severity};
