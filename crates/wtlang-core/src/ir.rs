// Intermediate Representation (IR) for WTLang
// 
// The IR serves as a bridge between the AST and code generation,
// providing a platform-independent representation that enables:
// - Advanced type checking and semantic analysis
// - Cross-platform optimizations
// - Better tooling support (LSP, debugger)
// - Multiple backend targets

pub mod types;
pub mod nodes;
pub mod module;
pub mod builder;

// Re-export commonly used types
pub use types::*;
pub use nodes::*;
pub use module::*;
pub use builder::*;

