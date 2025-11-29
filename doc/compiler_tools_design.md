# WTLang Compiler and Tools Design Considerations

This document evaluates implementation alternatives for the WTLang compiler and supporting tools, providing rationale for the chosen approaches.

## 1. Compiler Implementation Language

### Alternatives Considered

**A. Python**
- Pros: Easy prototyping, rich ecosystem, same as target platform
- Cons: Slower execution, packaging challenges for distribution

**B. Rust (Chosen)**
- Pros: Excellent performance, memory safety, great tooling (cargo), modern language features
- Cons: Steeper learning curve, longer compile times

**C. C++**
- Pros: Maximum performance, mature ecosystem
- Cons: Memory management complexity, slower development, fragmented tooling

**D. Go**
- Pros: Simple, fast compilation, good tooling
- Cons: Less sophisticated type system, limited metaprogramming

**E. OCaml/Haskell**
- Pros: Excellent for compilers (pattern matching, ADTs), proven track record
- Cons: Smaller ecosystem, less mainstream

### Rationale
Rust was chosen for the compiler implementation because:
1. **Performance**: Critical for fast compilation in large projects
2. **Safety**: Memory safety without garbage collection prevents entire classes of bugs
3. **Tooling**: Cargo provides excellent dependency management and build system
4. **Modern Features**: Pattern matching, enums, and trait system are ideal for compiler construction
5. **Ecosystem**: Libraries like `nom` (parsing), `lalrpop` (parser generators), and `tower-lsp` (LSP) are mature

The performance requirement is non-negotiable for developer experience—slow compilation kills productivity.

## 2. Parser Technology

### Alternatives Considered

**A. Hand-Written Recursive Descent Parser**
- Pros: Full control, easy to debug, good error messages
- Cons: Time-consuming, error-prone, harder to maintain

**B. Parser Generator (LALRPOP/Yacc-style) (Chosen for Production)**
```rust
grammar;
pub Program: Program = {
    <pages:Page*> => Program { pages }
};
Page: Page = {
    "page" <name:Ident> "{" <content:Statement*> "}" => Page { name, content }
};
```
- Pros: Declarative, proven approach, less code
- Cons: Learning curve, sometimes cryptic errors

**C. Parser Combinator (nom/combine)**
```rust
fn parse_page(input: &str) -> IResult<&str, Page> {
    let (input, _) = tag("page")(input)?;
    let (input, name) = identifier(input)?;
    // ... more combinators
}
```
- Pros: Composable, type-safe, flexible
- Cons: Can be verbose, performance tradeoffs

**D. PEG Parser (pest)**
- Pros: Simple grammar syntax, good error recovery
- Cons: No left-recursion, less control over precedence

### Rationale
**Two-phase approach:**
1. **Prototyping**: Parser combinators (nom) for rapid iteration
2. **Production**: LALRPOP for performance and maintainability

LALRPOP provides the best balance of expressiveness and performance for a production compiler, with excellent error reporting capabilities that can be customized for user-friendly messages.

## 3. Compiler Architecture

### Alternatives Considered

**A. Single-Pass Compiler**
- Pros: Fast, simple
- Cons: Limited optimization, hard to extend

**B. Multi-Pass Pipeline (Chosen)**
```
Source Code
    ↓ [Lexer]
Tokens
    ↓ [Parser]
AST (Abstract Syntax Tree)
    ↓ [Semantic Analysis]
Typed AST
    ↓ [IR Generation]
Intermediate Representation
    ↓ [Optimization]
Optimized IR
    ↓ [Code Generation]
Target Code (Python)
```
- Pros: Separation of concerns, extensible, allows optimization
- Cons: More complex, slower (mitigated by Rust performance)

**C. Query-based (Salsa-style)**
- Pros: Incremental compilation, caching, IDE-friendly
- Cons: Paradigm shift, complexity

### Rationale
A traditional multi-pass pipeline provides the clearest architecture and easiest maintenance. Each phase has well-defined responsibilities:

1. **Lexing/Parsing**: Convert text to AST
2. **Semantic Analysis**: Type checking, symbol resolution, validation
3. **IR Generation**: Platform-agnostic intermediate form
4. **Optimization**: Simplify code, eliminate dead code
5. **Code Generation**: Emit Python (or other targets)

This architecture naturally supports multiple target platforms—only the final code generation step changes.

## 4. Type Checker Implementation

### Alternatives Considered

**A. Hindley-Milner Type Inference**
- Pros: Powerful, minimal annotations
- Cons: Complex implementation, confusing error messages

**B. Bidirectional Type Checking (Chosen)**
```rust
fn check_expr(expr: &Expr, expected: &Type) -> Result<(), TypeError> {
    match expr {
        Expr::Lit(n) => unify(&Type::Int, expected),
        Expr::Var(x) => unify(&lookup_var(x), expected),
        // ...
    }
}

fn infer_expr(expr: &Expr) -> Result<Type, TypeError> {
    match expr {
        Expr::Lit(n) => Ok(Type::Int),
        Expr::Add(e1, e2) => {
            check_expr(e1, &Type::Int)?;
            check_expr(e2, &Type::Int)?;
            Ok(Type::Int)
        }
        // ...
    }
}
```
- Pros: Good error messages, predictable, works well with annotations
- Cons: Requires more type annotations

**C. Constraint-based**
- Pros: Flexible, supports advanced features
- Cons: Complex, harder to implement

### Rationale
Bidirectional type checking is the sweet spot for WTLang. Since the language is explicitly typed, users provide type annotations for tables and function signatures. The type checker:
- **Checks**: Verifies expressions against expected types
- **Infers**: Derives types for local variables and intermediate expressions
- **Reports**: Produces clear, actionable error messages

This approach is predictable and produces errors where users expect them.

## 5. Error Reporting

### Alternatives Considered

**A. Simple Error Messages**
```
Error: Type mismatch at line 42
```
- Pros: Easy to implement
- Cons: Poor developer experience

**B. Rich Diagnostic Messages (Chosen)**
```
Error: Type mismatch in function call
  ┌─ src/main.wt:42:5
  │
42│     filter(users, age > "18")
  │                   ^^^ expected function(row) -> bool, found comparison
  │
  = note: "18" is a string, but age is an integer
  = help: did you mean age > 18 (without quotes)?
```
- Pros: Excellent UX, guides users to fixes
- Cons: More implementation effort

**C. Compiler as IDE Service**
- Pros: Real-time feedback
- Cons: Requires Language Server

### Rationale
Using the `codespan-reporting` crate, WTLang can produce beautiful error messages with:
- Source location with context
- Color-coded severity (error/warning/note)
- Suggestions for fixes
- Multiple related errors grouped together

This is essential for a DSL targeting non-expert users.

## 6. Source Code Structure for Multi-Tool Support

### Overview

The WTLang toolchain is designed to support multiple tools beyond just the compiler: Language Server, debugger, documentation generator, formatter, and more. To achieve this, the codebase must be structured to share core components while allowing tool-specific functionality.

### Alternatives Considered

**A. Monolithic Single Binary**
```
wtlang/
└── src/
    └── main.rs  // Everything in one file
```
- Pros: Simple, everything in one place
- Cons: Cannot reuse components, becomes unmaintainable, slow compile times

**B. Separate Projects for Each Tool**
```
wtlang-compiler/
wtlang-lsp/
wtlang-debugger/
wtlang-formatter/
```
- Pros: Clear separation, independent versioning
- Cons: Code duplication, synchronization issues, inconsistent behavior

**C. Library-Based Architecture with Multiple Binaries (Chosen)**
```
wtlang/
├── crates/
│   ├── wtlang-core/        # Shared library
│   ├── wtlang-compiler/    # Compiler binary
│   ├── wtlang-lsp/         # LSP server binary
│   ├── wtlang-debugger/    # Debugger
│   └── wtlang-formatter/   # Code formatter
└── Cargo.toml              # Workspace configuration
```
- Pros: Code reuse, consistent behavior, independent tools
- Cons: More complex project structure, workspace management

**D. Plugin Architecture**
- Pros: Extensible, community contributions
- Cons: Complex, version management, performance overhead

### Rationale: Library-Based Architecture

The library-based architecture provides the best balance:
1. **Core library** (`wtlang-core`) contains shared components
2. **Tool binaries** use the core library for specific purposes
3. **Cargo workspace** manages all crates together
4. **Consistent behavior** across all tools
5. **Efficient development** - changes to core propagate to all tools

### Detailed Source Code Structure

```
wtlang/
├── Cargo.toml                      # Workspace root
│
├── crates/
│   │
│   ├── wtlang-core/                # Core library (shared by all tools)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── lib.rs              # Library entry point
│   │       ├── syntax/             # Syntax analysis
│   │       │   ├── mod.rs
│   │       │   ├── lexer.rs        # Tokenization
│   │       │   ├── token.rs        # Token definitions
│   │       │   ├── parser.rs       # AST construction
│   │       │   └── ast.rs          # AST node definitions
│   │       ├── semantics/          # Semantic analysis
│   │       │   ├── mod.rs
│   │       │   ├── types.rs        # Type system
│   │       │   ├── checker.rs      # Type checker
│   │       │   ├── symbols.rs      # Symbol table
│   │       │   └── validator.rs    # Constraint validation
│   │       ├── ir/                 # Intermediate representation
│   │       │   ├── mod.rs
│   │       │   ├── nodes.rs        # IR node types
│   │       │   └── builder.rs      # AST → IR transformation
│   │       ├── diagnostics/        # Error reporting
│   │       │   ├── mod.rs
│   │       │   ├── error.rs        # Error types
│   │       │   ├── reporter.rs     # Diagnostic reporter
│   │       │   └── spans.rs        # Source location tracking
│   │       ├── query/              # Query system (Salsa-style)
│   │       │   ├── mod.rs
│   │       │   └── database.rs     # Incremental computation
│   │       └── util/               # Utilities
│   │           ├── mod.rs
│   │           ├── string_interner.rs
│   │           └── file_cache.rs
│   │
│   ├── wtlang-compiler/            # Compiler binary
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs             # CLI entry point
│   │       ├── cli.rs              # Command-line interface
│   │       ├── codegen/            # Code generation
│   │       │   ├── mod.rs
│   │       │   ├── python.rs       # Python/Streamlit generator
│   │       │   ├── typescript.rs   # Future: TypeScript generator
│   │       │   └── common.rs       # Shared codegen utilities
│   │       ├── optimize/           # Optimization passes
│   │       │   ├── mod.rs
│   │       │   ├── dead_code.rs
│   │       │   └── constant_fold.rs
│   │       └── project/            # Project management
│   │           ├── mod.rs
│   │           └── config.rs       # wtlang.toml handling
│   │
│   ├── wtlang-lsp/                 # Language Server
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs             # LSP server entry
│   │       ├── server.rs           # LSP protocol implementation
│   │       ├── features/           # LSP features
│   │       │   ├── mod.rs
│   │       │   ├── completion.rs   # Auto-completion
│   │       │   ├── hover.rs        # Type information on hover
│   │       │   ├── definition.rs   # Go to definition
│   │       │   ├── references.rs   # Find references
│   │       │   ├── rename.rs       # Symbol renaming
│   │       │   ├── diagnostics.rs  # Real-time error checking
│   │       │   ├── formatting.rs   # Code formatting
│   │       │   └── code_actions.rs # Quick fixes
│   │       ├── state/              # Server state management
│   │       │   ├── mod.rs
│   │       │   └── workspace.rs    # Workspace files tracking
│   │       └── analysis/           # Code analysis for IDE
│   │           ├── mod.rs
│   │           ├── document.rs     # Document analysis
│   │           └── project.rs      # Project-wide analysis
│   │
│   ├── wtlang-formatter/           # Code formatter
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       ├── format.rs           # Formatting logic
│   │       └── config.rs           # Formatting options
│   │
│   ├── wtlang-debugger/            # Debugger (future)
│   │   ├── Cargo.toml
│   │   └── src/
│   │       ├── main.rs
│   │       └── debug_adapter.rs    # DAP implementation
│   │
│   └── wtlang-doc/                 # Documentation generator
│       ├── Cargo.toml
│       └── src/
│           ├── main.rs
│           └── generator.rs        # API doc generation
│
├── examples/                       # Example WTLang programs
├── tests/                          # Integration tests
│   ├── compiler/
│   ├── lsp/
│   └── formatter/
│
└── docs/                           # Project documentation
    ├── design/
    ├── tutorial/
    └── api/
```

### Core Library Design Principles

**1. Incremental and Query-Based**
Use a Salsa-like query system for incremental computation:
```rust
// wtlang-core/src/query/database.rs
#[salsa::query_group(CompilerDatabase)]
pub trait WTLangDatabase {
    #[salsa::input]
    fn source_text(&self, file: FileId) -> Arc<String>;
    
    fn parse(&self, file: FileId) -> Arc<Program>;
    fn check_types(&self, file: FileId) -> Arc<TypedProgram>;
    fn diagnostics(&self, file: FileId) -> Arc<Vec<Diagnostic>>;
}
```

Benefits:
- **Incremental**: Only recompute what changed
- **Cached**: Automatic memoization
- **IDE-friendly**: Perfect for Language Server
- **Consistent**: Same logic for compiler and LSP

**2. Position-Aware AST**
Every AST node tracks its source location:
```rust
// wtlang-core/src/syntax/ast.rs
#[derive(Debug, Clone)]
pub struct Spanned<T> {
    pub node: T,
    pub span: Span,
}

#[derive(Debug, Clone, Copy)]
pub struct Span {
    pub start: Position,
    pub end: Position,
}

pub type Expr = Spanned<ExprKind>;
pub type Statement = Spanned<StatementKind>;
```

Benefits:
- **Error reporting**: Precise error locations
- **IDE features**: Hover, go-to-definition work correctly
- **Refactoring**: Know exact code ranges to modify

**3. Immutable Data Structures**
Use persistent data structures with Arc/Rc:
```rust
// wtlang-core/src/syntax/ast.rs
pub struct Program {
    pub items: Arc<[ProgramItem]>,
    pub source: FileId,
}

pub struct TypedProgram {
    pub ast: Arc<Program>,
    pub types: Arc<TypeMap>,
    pub symbols: Arc<SymbolTable>,
}
```

Benefits:
- **Thread-safe**: Can be shared across threads (LSP needs this)
- **Efficient cloning**: Cheap to clone and pass around
- **Caching**: Safe to cache without mutation concerns

**4. Error Resilience**
Parser continues after errors for better IDE experience:
```rust
// wtlang-core/src/syntax/parser.rs
pub struct ParseResult {
    pub program: Program,
    pub errors: Vec<ParseError>,
}

impl Parser {
    fn parse(&mut self) -> ParseResult {
        let mut items = Vec::new();
        let mut errors = Vec::new();
        
        while !self.is_at_end() {
            match self.parse_item() {
                Ok(item) => items.push(item),
                Err(err) => {
                    errors.push(err);
                    self.recover(); // Skip to next item
                }
            }
        }
        
        ParseResult { program: Program { items }, errors }
    }
}
```

Benefits:
- **IDE tolerance**: Show multiple errors, don't stop at first
- **Better UX**: Users see all issues at once
- **Partial analysis**: Can still provide some features with errors

### Tool-Specific Components

**Compiler-Specific:**
- Code generation backends
- Optimization passes
- Build system integration
- Output file management

**LSP-Specific:**
- Real-time document synchronization
- Incremental re-parsing
- Symbol caching
- Workspace management
- Quick fix suggestions

**Formatter-Specific:**
- Pretty-printing logic
- Configuration file parsing
- Comment preservation
- Whitespace handling

**Debugger-Specific:**
- Debug adapter protocol
- Breakpoint management
- Variable inspection
- Step execution

### Shared vs. Tool-Specific Code

| Component | Core Library | Compiler | LSP | Formatter | Debugger |
|-----------|-------------|----------|-----|-----------|----------|
| Lexer | ✓ | | | | |
| Parser | ✓ | | | | |
| AST | ✓ | | | | |
| Type Checker | ✓ | | | | |
| Symbol Table | ✓ | | | | |
| Diagnostics | ✓ | | | | |
| IR Generation | ✓ | | | | |
| Code Generation | | ✓ | | | |
| Optimization | | ✓ | | | |
| Auto-completion | | | ✓ | | |
| Hover Info | | | ✓ | | |
| Pretty Printing | | | | ✓ | |
| Debug Adapter | | | | | ✓ |

### Cargo Workspace Configuration

```toml
# Root Cargo.toml
[workspace]
members = [
    "crates/wtlang-core",
    "crates/wtlang-compiler",
    "crates/wtlang-lsp",
    "crates/wtlang-formatter",
    "crates/wtlang-debugger",
    "crates/wtlang-doc",
]

[workspace.package]
version = "0.1.0"
edition = "2021"
authors = ["WTLang Contributors"]
license = "MIT"

[workspace.dependencies]
# Shared dependencies
salsa = "0.16"
tower-lsp = "0.20"
codespan-reporting = "0.11"
thiserror = "1.0"
anyhow = "1.0"
```

### Build Process

```bash
# Build everything
cargo build --workspace

# Build only compiler
cargo build -p wtlang-compiler

# Build only LSP
cargo build -p wtlang-lsp

# Run tests for all crates
cargo test --workspace

# Run only core tests
cargo test -p wtlang-core
```

### Benefits of This Structure

1. **Code Reuse**: Core library shared by all tools (no duplication)
2. **Consistency**: Same parsing, type checking across tools
3. **Maintainability**: Changes to core automatically affect all tools
4. **Performance**: Incremental compilation via Salsa
5. **IDE Integration**: LSP has everything it needs
6. **Extensibility**: Easy to add new tools
7. **Testing**: Can test core independently
8. **Modularity**: Clear boundaries between components

### Migration Path

**Phase 1** (Current - Basic Compiler):
- Flat structure with all code in `src/`
- Single binary `wtc`

**Phase 2** (Refactor to Library):
- Move core code to `wtlang-core` crate
- Compiler uses core library
- Maintain backward compatibility

**Phase 3** (Add LSP):
- Create `wtlang-lsp` crate
- Implement LSP using core library
- Add incremental computation (Salsa)

**Phase 4** (Additional Tools):
- Add formatter, debugger, doc generator
- All use core library
- Share testing infrastructure

This structure ensures the WTLang toolchain can grow from a simple compiler to a comprehensive development environment while maintaining code quality and consistency.

## 7. Language Server Protocol (LSP) Implementation

### Alternatives Considered

**A. No IDE Support**
- Pros: No extra work
- Cons: Poor developer experience, major adoption barrier

**B. VSCode Extension Only**
- Pros: Simpler, targets most popular editor
- Cons: Locks users into VSCode

**C. LSP Server (Chosen)**
```rust
use tower_lsp::{LspService, Server};

#[tower_lsp::async_trait]
impl LanguageServer for WTLangServer {
    async fn completion(&self, params: CompletionParams) -> Result<Option<CompletionResponse>> {
        // Provide autocomplete
    }
    
    async fn hover(&self, params: HoverParams) -> Result<Option<Hover>> {
        // Show type information
    }
    
    async fn goto_definition(&self, params: GotoDefinitionParams) -> Result<...> {
        // Navigate to definitions
    }
}
```
- Pros: Works with any LSP-compatible editor (VSCode, Vim, Emacs, etc.)
- Cons: More complex than editor-specific extension

### Rationale
Implementing the Language Server Protocol ensures WTLang works in any modern editor. The `tower-lsp` crate makes this straightforward in Rust. Key features to implement:

**Phase 1 (MVP):**
- Syntax highlighting
- Error diagnostics
- Go to definition
- Find references

**Phase 2:**
- Autocomplete (keywords, functions, table fields)
- Hover for type information
- Rename refactoring
- Format document

**Phase 3:**
- Code actions (quick fixes)
- Semantic tokens
- Call hierarchy
- **Test running integration**: Run tests from editor with inline results
- **Test debugging**: Set breakpoints in test blocks
- **Test coverage visualization**: Show which lines are tested

## 8. External Function Autocompletion

### Alternatives Considered

**A. Manual Declaration Files**
```wtlang
// analytics.wtd (WTLang Declaration)
external module analytics {
    function analyze_sentiment(text: string) -> float
    function classify(text: string) -> string
}
```
- Pros: Explicit, type-safe
- Cons: Maintenance burden, duplication

**B. Python Introspection (Chosen)**
```rust
// LSP server inspects Python modules at development time
fn analyze_python_module(module_path: &str) -> Vec<FunctionSignature> {
    // Use Python's inspect module via PyO3
    Python::with_gil(|py| {
        let module = py.import(module_path)?;
        extract_signatures(module)
    })
}
```
- Pros: No duplication, always in sync with actual code
- Cons: Requires Python runtime, may be slow

**C. Stub Files (.pyi)**
```python
# analytics.pyi
def analyze_sentiment(text: str) -> float: ...
def classify(text: str) -> str: ...
```
- Pros: Standard Python approach, type hints
- Cons: Maintenance burden

### Rationale
**Hybrid approach:**
1. LSP server uses PyO3 to introspect Python modules when available
2. Falls back to .pyi stub files for type information
3. Caches results for performance

This provides the best developer experience—users get autocomplete for external functions without manual declaration overhead, while maintaining type safety.

## 9. Build System and Tooling

### Alternatives Considered

**A. Custom Build Scripts**
- Pros: Full control
- Cons: Reinventing the wheel, platform-specific

**B. Make/CMake**
- Pros: Standard, powerful
- Cons: Complex syntax, not modern

**C. Cargo-based (Chosen)**
```toml
[package]
name = "wtlang"
version = "0.1.0"
edition = "2021"

[[bin]]
name = "wtc"  # WTLang compiler
path = "src/main.rs"

[[bin]]
name = "wtlang-lsp"
path = "src/lsp/main.rs"

[dependencies]
lalrpop-util = "0.19"
tower-lsp = "0.20"
codespan-reporting = "0.11"
```
- Pros: Modern, excellent dependency management, cross-platform
- Cons: Rust-specific

**D. Dedicated CLI (Clap)**
```rust
#[derive(Parser)]
#[command(name = "wtc")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

enum Commands {
    Build { input: PathBuf, output: PathBuf },
    Check { input: PathBuf },
    Format { input: PathBuf },
    Test { 
        input: PathBuf,
        #[arg(long)]
        watch: bool,
        #[arg(long)]
        coverage: bool,
    },
    Lsp,
}
```
- Pros: User-friendly, self-documenting
- Cons: Additional dependency

### Rationale
Use Cargo as the build system for the compiler itself, and provide a user-friendly CLI using Clap:

```bash
wtc build src/main.wt --output dist/
wtc check src/main.wt
wtc format src/main.wt
wtc test src/ --watch          # Run tests in watch mode
wtc test src/ --coverage       # Generate coverage report
wtc lsp  # Start Language Server
wtc init my-project  # Scaffold new project
```

This provides familiar tooling for developers while leveraging Rust's excellent build infrastructure.

## 10. Testing Strategy

### Alternatives Considered

**A. Unit Tests Only**
- Pros: Fast, focused
- Cons: Misses integration issues

**B. Comprehensive Test Suite (Chosen)**
```rust
// Unit tests
#[test]
fn test_parse_page() {
    let src = "page Home { }";
    let ast = parse(src).unwrap();
    assert_eq!(ast.pages.len(), 1);
}

// Integration tests
#[test]
fn test_compile_simple_program() {
    let compiled = compile_file("tests/fixtures/simple.wt");
    assert!(compiled.is_ok());
}

// Golden tests
#[test]
fn test_code_generation() {
    let output = compile("tests/input.wt");
    assert_snapshot!(output);  // Using insta crate
}

// Property-based tests
#[quickcheck]
fn parse_then_print_is_identity(prog: Program) -> bool {
    parse(&print(prog)) == prog
}
```
- Pros: High confidence, catches regressions
- Cons: More maintenance

**C. Fuzzing**
```rust
#[cfg(test)]
fn fuzz_parser(data: &[u8]) {
    let _ = parse(data);  // Should never crash
}
```
- Pros: Finds edge cases
- Cons: Requires setup, CI integration

### Rationale
Multi-level testing strategy for the compiler itself:
1. **Unit tests**: Parser, type checker, code generator components
2. **Integration tests**: End-to-end compilation of sample programs
3. **Snapshot tests**: Verify generated code doesn't change unexpectedly (using `insta`)
4. **Fuzzing**: Ensure parser robustness (using `cargo-fuzz`)
5. **Regression tests**: Archive of bug-reproducing test cases

**Additional Testing for User Code Compilation:**
```rust
// Test that WTLang test blocks compile to pytest
#[test]
fn test_wtlang_test_compilation() {
    let wtlang_src = r#"
        test "example" {
            let x = 5
            assert x == 5
        }
    "#;
    
    let python_output = compile(wtlang_src).unwrap();
    
    // Verify pytest-compatible output
    assert!(python_output.contains("def test_example():"));
    assert!(python_output.contains("assert x == 5"));
}

// Test mock function generation
#[test]
fn test_mock_external_function() {
    let wtlang_src = r#"
        mock external process(x: int) -> int {
            return x * 2
        }
    "#;
    
    let python_output = compile(wtlang_src).unwrap();
    assert!(python_output.contains("@patch"));
}
```

**WTLang User Testing Support:**
The compiler must support the `test` keyword and compile it to pytest:

```wtlang
// user_code.wt
test "sorting works" {
    let data = table_from([{id: 2}, {id: 1}])
    let sorted = data -> sort(_, "id")
    assert sorted[0].id == 1
}
```

Compiles to:
```python
# user_code_test.py
def test_sorting_works():
    data = pd.DataFrame([{"id": 2}, {"id": 1}])
    sorted_data = data.sort_values("id")
    assert sorted_data.iloc[0]["id"] == 1
```

The `wtc test` command:
1. Compiles WTLang test blocks to Python pytest functions
2. Runs pytest with appropriate fixtures and mocks
3. Reports results in WTLang-friendly format
4. Integrates with LSP for in-editor test running

This catches bugs at all levels while maintaining fast development iteration.

## 11. Documentation and Developer Experience

### Alternatives Considered

**A. README Only**
- Pros: Simple
- Cons: Insufficient for real use

**B. Comprehensive Documentation Site (Chosen)**
```
docs/
  ├── getting-started/
  │   ├── installation.md
  │   ├── hello-world.md
  │   └── tutorial.md
  ├── language-reference/
  │   ├── types.md
  │   ├── functions.md
  │   ├── tables.md
  │   └── syntax.md
  ├── guides/
  │   ├── external-functions.md
  │   ├── deployment.md
  │   └── best-practices.md
  └── api/
      └── standard-library.md
```
Built with: mdBook or Docusaurus
- Pros: Professional, searchable, versioned
- Cons: More maintenance

**C. In-IDE Documentation**
- Pros: Contextual, convenient
- Cons: Limited scope

### Rationale
Multi-channel documentation approach:
1. **CLI help**: `wtc --help` for quick reference
2. **LSP hover**: Show function signatures and documentation in editor
3. **Website**: Comprehensive tutorials and reference (using mdBook)
4. **Examples**: Repository of example projects
5. **Playground**: Web-based REPL for experimentation (future)

Focus on making the first 5 minutes delightful—clear installation, simple "hello world", immediate value.

## 12. Release and Distribution

### Alternatives Considered

**A. Source-only Distribution**
- Pros: Simple, always up-to-date
- Cons: Users must have Rust toolchain

**B. Binary Releases (Chosen)**
```yaml
# GitHub Actions workflow
name: Release
on:
  push:
    tags: ['v*']
jobs:
  build:
    strategy:
      matrix:
        os: [ubuntu-latest, macos-latest, windows-latest]
    steps:
      - uses: actions/checkout@v2
      - run: cargo build --release
      - run: cargo test
      - uses: actions/upload-artifact@v2
```
- Pros: Easy installation, no toolchain required
- Cons: More CI complexity

**C. Package Managers**
- Homebrew (macOS): `brew install wtlang`
- Chocolatey (Windows): `choco install wtlang`
- Cargo (all platforms): `cargo install wtlang`
- Pros: Familiar installation
- Cons: Package submission overhead

### Rationale
**Multi-platform distribution:**
1. **GitHub Releases**: Pre-built binaries for Windows, macOS, Linux
2. **Cargo**: `cargo install wtlang` for developers
3. **Package managers**: Homebrew and Chocolatey for mainstream users
4. **Docker**: Container image for CI/CD integration
5. **VSCode Extension Marketplace**: One-click install for IDE support

Automated via GitHub Actions for zero-friction releases.

## 13. Performance Optimization

### Alternatives Considered

**A. No Optimization**
- Pros: Simpler compiler
- Cons: Slow compilation, poor user experience

**B. Incremental Compilation (Chosen)**
```rust
// Cache parsed and type-checked files
struct CompilerCache {
    ast_cache: HashMap<PathBuf, (Timestamp, AST)>,
    type_cache: HashMap<PathBuf, (Timestamp, TypedAST)>,
}

fn compile_incremental(file: PathBuf, cache: &mut CompilerCache) -> Result<Output> {
    if let Some((timestamp, ast)) = cache.ast_cache.get(&file) {
        if file_modified_time(&file) <= *timestamp {
            return Ok(use_cached(ast));
        }
    }
    // Re-compile
}
```
- Pros: Fast iteration, better developer experience
- Cons: Cache invalidation complexity

**C. Parallel Compilation**
```rust
use rayon::prelude::*;

fn compile_modules(modules: Vec<Module>) -> Vec<Result<CompiledModule>> {
    modules.par_iter()
           .map(|m| compile_module(m))
           .collect()
}
```
- Pros: Utilizes multi-core CPUs
- Cons: More complex, potential race conditions

### Rationale
**Three-level optimization:**
1. **Rust performance**: Base language is already fast
2. **Incremental compilation**: Only recompile changed files
3. **Parallel module compilation**: Independent modules compile in parallel

Target: <100ms for recompilation of single module, <1s for full project rebuild of typical application (10-20 files).

## Summary

The compiler and tools strategy for WTLang:

**Compiler Core:**
- **Language**: Rust for performance and safety
- **Parser**: LALRPOP for production-grade parsing
- **Architecture**: Multi-pass pipeline with clear IR
- **Type System**: Bidirectional type checking
- **Errors**: Rich diagnostics with codespan-reporting

**Developer Tools:**
- **LSP Server**: Universal IDE support via tower-lsp
- **CLI**: User-friendly interface with clap
- **Autocomplete**: Python introspection for external functions
- **Testing**: Comprehensive unit, integration, and snapshot tests

**Distribution:**
- **Binaries**: Cross-platform pre-built releases
- **Package Managers**: Cargo, Homebrew, Chocolatey
- **Documentation**: mdBook-based comprehensive docs
- **VSCode Extension**: One-click installation and setup

This tooling strategy prioritizes developer experience while maintaining the performance and reliability expected from a production compiler. The Rust ecosystem provides battle-tested libraries for each component, reducing implementation risk and accelerating development.
