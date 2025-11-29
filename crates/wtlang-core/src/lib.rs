// WTLang Core Library
// Shared components for compiler, LSP, and other tools

pub mod lexer;
pub mod ast;
pub mod parser;

// Re-export commonly used types
pub use lexer::{Lexer, Token, TokenType};
pub use ast::*;
pub use parser::Parser;
