# Testing Strategy for WTLang Tools

## Overview

This document outlines the testing strategy for the WTLang compiler, language server (LSP), and related tools. A comprehensive test suite ensures code quality, prevents regressions, and provides confidence when making changes to the language or tooling.

## Test Categories

### 1. Unit Tests

Unit tests verify individual components in isolation.

#### Lexer Tests (`wtlang-core/src/lexer.rs`)

Test the lexical analysis phase:

- **Token Recognition**: Verify each token type is correctly identified
  - Keywords: `page`, `table`, `from`, `display`, `button`, etc.
  - Identifiers: valid variable/function names
  - Literals: strings, numbers, booleans
  - Operators: `=`, `>`, `<`, `>=`, `<=`, `==`, `!=`
  - Delimiters: `{`, `}`, `(`, `)`, `,`, `:`, `->`
  
- **Error Handling**: Invalid characters, unterminated strings, malformed numbers
- **Position Tracking**: Line and column numbers are accurate
- **Whitespace**: Properly ignored except in strings

Example test structure:
```rust
#[cfg(test)]
mod lexer_tests {
    use super::*;

    #[test]
    fn test_keywords() {
        let mut lexer = Lexer::new("page table from display");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::Page);
        assert_eq!(tokens[1].kind, TokenKind::Table);
        // ... more assertions
    }

    #[test]
    fn test_string_literal() {
        let mut lexer = Lexer::new(r#""Hello, World!""#);
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens[0].kind, TokenKind::StringLiteral);
        assert_eq!(tokens[0].value, Some("Hello, World!".to_string()));
    }

    #[test]
    fn test_unterminated_string() {
        let mut lexer = Lexer::new(r#""Unterminated"#);
        let result = lexer.tokenize();
        assert!(result.is_err());
    }
}
```

#### Parser Tests (`wtlang-core/src/parser.rs`)

Test the parsing phase and AST construction:

- **Statement Parsing**: Each statement type parsed correctly
  - Page declarations
  - Table definitions
  - Variable declarations (with/without type annotations)
  - Function definitions
  - Assignments
  - Function calls
  - Control flow (if/else)
  - Loops (forall)
  - Return statements
  
- **Expression Parsing**: All expression types
  - Literals
  - Variables
  - Binary operations
  - Function calls
  - Field access (chaining)
  
- **Type Annotations**: Colon syntax parsed correctly
- **Error Recovery**: Meaningful errors for invalid syntax
- **Complex Programs**: Multi-page programs, nested structures

Example test structure:
```rust
#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn test_parse_page_declaration() {
        let tokens = vec![
            Token { kind: TokenKind::Page, value: None, line: 1, column: 1 },
            Token { kind: TokenKind::Identifier, value: Some("HomePage".to_string()), line: 1, column: 6 },
            // ... more tokens
        ];
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();
        assert_eq!(program.items.len(), 1);
        if let Item::Page(page) = &program.items[0] {
            assert_eq!(page.name, "HomePage");
        } else {
            panic!("Expected Page item");
        }
    }

    #[test]
    fn test_parse_type_annotation() {
        let source = "let count: number = 5";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let program = parser.parse().unwrap();
        // Verify type annotation is present
    }

    #[test]
    fn test_parse_error_missing_brace() {
        let source = "page Test {";
        let mut lexer = Lexer::new(source);
        let tokens = lexer.tokenize().unwrap();
        let mut parser = Parser::new(tokens);
        let result = parser.parse();
        assert!(result.is_err());
    }
}
```

#### Symbol Table Tests (`wtlang-core/src/symbols.rs`)

Test scope management and symbol resolution:

- **Scope Creation**: Global, page, function, nested scopes
- **Symbol Definition**: Variables, functions, parameters
- **Symbol Lookup**: Find symbols in current and parent scopes
- **Shadowing**: Inner scopes shadow outer scopes correctly
- **Type Tracking**: Symbol types are tracked correctly
- **Initialization State**: Variables marked as initialized/uninitialized

Example test structure:
```rust
#[cfg(test)]
mod symbol_tests {
    use super::*;

    #[test]
    fn test_global_scope() {
        let mut table = SymbolTable::new();
        table.define("global_var", Type::Number, true).unwrap();
        let symbol = table.lookup("global_var").unwrap();
        assert_eq!(symbol.ty, Type::Number);
        assert!(symbol.initialized);
    }

    #[test]
    fn test_nested_scopes() {
        let mut table = SymbolTable::new();
        table.define("outer", Type::String, true).unwrap();
        table.enter_scope(ScopeKind::Page);
        table.define("inner", Type::Number, true).unwrap();
        
        // Inner scope can see both
        assert!(table.lookup("outer").is_some());
        assert!(table.lookup("inner").is_some());
        
        table.exit_scope();
        
        // Outer scope can't see inner
        assert!(table.lookup("outer").is_some());
        assert!(table.lookup("inner").is_none());
    }

    #[test]
    fn test_shadowing() {
        let mut table = SymbolTable::new();
        table.define("x", Type::Number, true).unwrap();
        table.enter_scope(ScopeKind::Function);
        table.define("x", Type::String, true).unwrap();
        
        let symbol = table.lookup("x").unwrap();
        assert_eq!(symbol.ty, Type::String); // Inner shadows outer
    }
}
```

#### Semantic Analysis Tests (`wtlang-core/src/semantics.rs`)

Test semantic validation:

- **Undefined Variables**: Detect use of undefined variables
- **Type Checking**: Type mismatches in assignments, function calls
- **Definite Assignment**: Variables used before initialization
- **Duplicate Definitions**: Redefinition of variables/functions
- **Function Scope Isolation**: Functions have independent scopes
- **Page Scope Isolation**: Pages don't share variables

Example test structure:
```rust
#[cfg(test)]
mod semantic_tests {
    use super::*;

    #[test]
    fn test_undefined_variable() {
        let source = r#"
            page Test {
                display x
            }
        "#;
        let program = parse_source(source).unwrap();
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&program);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("undefined"));
    }

    #[test]
    fn test_type_mismatch() {
        let source = r#"
            page Test {
                let x: number = "string"
            }
        "#;
        let program = parse_source(source).unwrap();
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&program);
        assert!(result.is_err());
    }

    #[test]
    fn test_uninitialized_variable() {
        let source = r#"
            page Test {
                let result: number
                display result
            }
        "#;
        let program = parse_source(source).unwrap();
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&program);
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not initialized"));
    }

    #[test]
    fn test_valid_conditional_initialization() {
        let source = r#"
            page Test {
                let result: number
                if true {
                    result = 42
                } else {
                    result = 0
                }
                display result
            }
        "#;
        let program = parse_source(source).unwrap();
        let mut analyzer = SemanticAnalyzer::new();
        let result = analyzer.analyze(&program);
        assert!(result.is_ok());
    }
}
```

#### Code Generation Tests (`wtlang-compiler/src/codegen.rs`)

Test Python/Streamlit code generation:

- **Correct Syntax**: Generated Python is syntactically valid
- **Proper Indentation**: Python indentation rules followed
- **Streamlit API Usage**: Correct use of st.* functions
- **Variable Scoping**: Python variables scoped correctly
- **Type Handling**: WTLang types map to Python correctly
- **Edge Cases**: Empty pages, complex expressions

Example test structure:
```rust
#[cfg(test)]
mod codegen_tests {
    use super::*;

    #[test]
    fn test_generate_simple_page() {
        let source = r#"
            page Test {
                text "Hello, World!"
            }
        "#;
        let program = parse_and_analyze(source).unwrap();
        let mut generator = CodeGenerator::new();
        let code = generator.generate(&program).unwrap();
        
        assert!(code.contains("import streamlit as st"));
        assert!(code.contains("def Test():"));
        assert!(code.contains(r#"st.write("Hello, World!")"#));
    }

    #[test]
    fn test_generate_variable_declaration() {
        let source = r#"
            page Test {
                let x: number = 42
                display x
            }
        "#;
        let program = parse_and_analyze(source).unwrap();
        let mut generator = CodeGenerator::new();
        let code = generator.generate(&program).unwrap();
        
        assert!(code.contains("x = 42"));
        assert!(code.contains("st.write(x)"));
    }

    #[test]
    fn test_generate_uninitialized_variable() {
        let source = r#"
            page Test {
                let result: number
                result = 42
            }
        "#;
        let program = parse_and_analyze(source).unwrap();
        let mut generator = CodeGenerator::new();
        let code = generator.generate(&program).unwrap();
        
        assert!(code.contains("result = None"));
        assert!(code.contains("# Will be assigned later"));
    }
}
```

### 2. Integration Tests

Integration tests verify that components work together correctly.

#### End-to-End Compiler Tests

Test the complete compilation pipeline from source to Python:

Create a `tests/` directory at the workspace root with test files:

```
tests/
  integration/
    test_compiler.rs
    fixtures/
      valid/
        simple_page.wt
        type_annotations.wt
        scoping.wt
        filters.wt
      invalid/
        undefined_var.wt
        type_mismatch.wt
        syntax_error.wt
```

Example test:
```rust
// tests/integration/test_compiler.rs
use std::process::Command;
use std::fs;

#[test]
fn test_compile_simple_page() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "wtc", "--", "build", "tests/integration/fixtures/valid/simple_page.wt", "--output", "test_output"])
        .output()
        .expect("Failed to execute compiler");
    
    assert!(output.status.success());
    
    // Verify output file exists
    assert!(fs::metadata("test_output/SimplePage.py").is_ok());
    
    // Verify output is valid Python
    let py_output = Command::new("python")
        .args(&["-m", "py_compile", "test_output/SimplePage.py"])
        .output()
        .expect("Failed to check Python syntax");
    
    assert!(py_output.status.success());
    
    // Cleanup
    fs::remove_dir_all("test_output").ok();
}

#[test]
fn test_compile_with_semantic_error() {
    let output = Command::new("cargo")
        .args(&["run", "--bin", "wtc", "--", "check", "tests/integration/fixtures/invalid/undefined_var.wt"])
        .output()
        .expect("Failed to execute compiler");
    
    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("undefined") || stderr.contains("Undefined"));
}
```

#### LSP Integration Tests

Test the Language Server Protocol implementation:

```rust
// tests/integration/test_lsp.rs
use tower_lsp::lsp_types::*;
use tower_lsp::LspService;

#[tokio::test]
async fn test_lsp_initialize() {
    let (service, _) = LspService::new(|client| WTLangServer::new(client));
    
    let params = InitializeParams::default();
    let result = service.inner().initialize(params).await;
    
    assert!(result.is_ok());
    let init_result = result.unwrap();
    assert!(init_result.capabilities.hover_provider.is_some());
    assert!(init_result.capabilities.completion_provider.is_some());
}

#[tokio::test]
async fn test_lsp_diagnostics() {
    let (service, _) = LspService::new(|client| WTLangServer::new(client));
    
    // Open a file with an error
    let uri = Url::parse("file:///test.wt").unwrap();
    let params = DidOpenTextDocumentParams {
        text_document: TextDocumentItem {
            uri: uri.clone(),
            language_id: "wtlang".to_string(),
            version: 1,
            text: "page Test { display undefined_var }".to_string(),
        },
    };
    
    service.inner().did_open(params).await;
    
    // Check that diagnostics were published
    // (This requires capturing client notifications in tests)
}
```

### 3. Regression Tests

Maintain a suite of example programs that should always compile and run:

- Use all examples in `examples/` directory
- Each example should have expected output or behavior
- Run after every significant change

Example script:
```bash
#!/bin/bash
# test_examples.sh

EXAMPLES_DIR="examples"
OUTPUT_DIR="test_regression_output"

mkdir -p "$OUTPUT_DIR"

for example in "$EXAMPLES_DIR"/*.wt; do
    echo "Testing $example..."
    
    # Check syntax
    ./target/release/wtc check "$example"
    if [ $? -ne 0 ]; then
        echo "FAIL: $example failed syntax check"
        exit 1
    fi
    
    # Compile
    basename=$(basename "$example" .wt)
    ./target/release/wtc build "$example" --output "$OUTPUT_DIR/$basename"
    if [ $? -ne 0 ]; then
        echo "FAIL: $example failed to compile"
        exit 1
    fi
    
    # Verify Python syntax
    python -m py_compile "$OUTPUT_DIR/$basename"/*.py
    if [ $? -ne 0 ]; then
        echo "FAIL: Generated Python is invalid for $example"
        exit 1
    fi
    
    echo "PASS: $example"
done

echo "All regression tests passed!"
rm -rf "$OUTPUT_DIR"
```

### 4. Performance Tests

Test compilation speed and LSP responsiveness:

```rust
#[test]
fn test_compilation_performance() {
    use std::time::Instant;
    
    let large_source = generate_large_program(1000); // 1000 lines
    let start = Instant::now();
    
    let program = parse_and_analyze(&large_source).unwrap();
    let mut generator = CodeGenerator::new();
    generator.generate(&program).unwrap();
    
    let duration = start.elapsed();
    assert!(duration.as_millis() < 1000, "Compilation took too long: {:?}", duration);
}
```

## Running Tests

### Using Cargo

Cargo provides built-in test support:

```bash
# Run all tests
cargo test

# Run tests for a specific crate
cargo test -p wtlang-core

# Run tests with output
cargo test -- --nocapture

# Run a specific test
cargo test test_parse_page_declaration

# Run tests with coverage (requires cargo-tarpaulin)
cargo tarpaulin --out Html
```

### Test Organization

Tests should be organized as follows:

```
wtlang/
  crates/
    wtlang-core/
      src/
        lexer.rs          # Contains #[cfg(test)] mod lexer_tests
        parser.rs         # Contains #[cfg(test)] mod parser_tests
        symbols.rs        # Contains #[cfg(test)] mod symbol_tests
        semantics.rs      # Contains #[cfg(test)] mod semantic_tests
    wtlang-compiler/
      src/
        codegen.rs        # Contains #[cfg(test)] mod codegen_tests
        main.rs
      tests/              # Integration tests
        cli_tests.rs
  tests/                  # Workspace-level integration tests
    integration/
      test_compiler.rs
      test_lsp.rs
      fixtures/
        ...
```

### Continuous Integration

Set up CI to run tests automatically on every commit:

**GitHub Actions** (`.github/workflows/test.yml`):
```yaml
name: Tests

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    
    steps:
    - uses: actions/checkout@v3
    
    - name: Install Rust
      uses: actions-rs/toolchain@v1
      with:
        toolchain: stable
        override: true
    
    - name: Run tests
      run: cargo test --all
    
    - name: Check code formatting
      run: cargo fmt -- --check
    
    - name: Run clippy
      run: cargo clippy -- -D warnings
    
    - name: Test examples
      run: |
        cargo build --release
        ./scripts/test_examples.sh
```

## Test Fixtures

Create standardized test fixtures for common scenarios:

### Valid Programs

**`tests/fixtures/valid/simple_page.wt`**:
```wtlang
page SimplePage {
    text "Hello, World!"
}
```

**`tests/fixtures/valid/type_annotations.wt`**:
```wtlang
page TypeTest {
    let count: number = 5
    let name: string = "Test"
    let active: boolean = true
    
    display count
    display name
    display active
}
```

**`tests/fixtures/valid/scoping.wt`**:
```wtlang
let global_var: number = 100

function add(x: number, y: number) -> number {
    return x + y
}

page ScopeTest {
    let page_var: number = 10
    let result: number = add(page_var, global_var)
    display result
}
```

### Invalid Programs

**`tests/fixtures/invalid/undefined_var.wt`**:
```wtlang
page Test {
    display undefined_variable
}
```

**`tests/fixtures/invalid/type_mismatch.wt`**:
```wtlang
page Test {
    let x: number = "string"
}
```

**`tests/fixtures/invalid/uninitialized.wt`**:
```wtlang
page Test {
    let x: number
    display x
}
```

## Test Coverage

Aim for comprehensive coverage:

- **Lexer**: 100% of token types, all error conditions
- **Parser**: All statement types, all expression types, error recovery
- **Symbols**: All scope types, lookup, shadowing
- **Semantics**: All error conditions, all validation rules
- **Codegen**: All language constructs, edge cases

Use `cargo tarpaulin` to measure coverage:

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Run with coverage
cargo tarpaulin --out Html --output-dir coverage

# View report
open coverage/index.html
```

## Test-Driven Development Workflow

When adding new features:

1. **Write failing tests first**: Define expected behavior
2. **Implement the feature**: Make tests pass
3. **Refactor**: Improve code while tests remain green
4. **Add edge case tests**: Cover corner cases
5. **Update integration tests**: Ensure end-to-end works

Example workflow for adding a new feature:

```rust
// 1. Write test (fails)
#[test]
fn test_new_feature() {
    let source = "page Test { new_syntax }";
    let result = parse_and_analyze(source);
    assert!(result.is_ok());
}

// 2. Implement feature
// ... modify lexer, parser, etc.

// 3. Test passes

// 4. Add edge cases
#[test]
fn test_new_feature_with_nesting() {
    // ...
}

// 5. Integration test
#[test]
fn test_compile_with_new_feature() {
    // ...
}
```

## Test Maintenance

- **Review tests regularly**: Remove obsolete tests, add missing coverage
- **Keep tests fast**: Use mocks/stubs for expensive operations
- **Make tests readable**: Clear names, good documentation
- **Avoid test interdependence**: Each test should be independent
- **Update tests with code changes**: Keep tests in sync with implementation

## Documentation Testing

Test code examples in documentation:

```rust
/// Parse a simple page declaration
/// 
/// # Example
/// ```
/// use wtlang_core::{Lexer, Parser};
/// 
/// let source = "page Test { }";
/// let mut lexer = Lexer::new(source);
/// let tokens = lexer.tokenize().unwrap();
/// let mut parser = Parser::new(tokens);
/// let program = parser.parse().unwrap();
/// assert_eq!(program.items.len(), 1);
/// ```
pub fn parse_page() {
    // ...
}
```

Run documentation tests with:
```bash
cargo test --doc
```

## Conclusion

This testing strategy ensures:

- **Quality**: Catch bugs early through comprehensive testing
- **Confidence**: Make changes without fear of breaking existing functionality
- **Documentation**: Tests serve as executable documentation
- **Maintainability**: Well-tested code is easier to refactor and extend

Implement tests incrementally, starting with critical paths (lexer → parser → semantics → codegen), then expanding to cover edge cases and advanced features. Run tests frequently during development and always before committing changes.
