# WTLang - Web Table Language

**A Domain Specific Language for creating web-based table applications**

WTLang allows you to quickly build interactive web applications for displaying and editing tabular data, compiling to Python/Streamlit for easy deployment.

## Features

- 📊 **Table-First Design**: Define table structures with types and constraints
- 🔄 **Function Chaining**: Intuitive data transformations with the `->` operator
- 🎨 **Declarative UI**: Simple syntax for creating pages, buttons, and sections
- 🔒 **Type Safe**: Strong static typing prevents runtime errors
- 🚀 **Compiles to Streamlit**: Generates production-ready Python code
- 📦 **Immutable Data**: Functional programming principles for predictable behavior
- 🛠️ **IDE Support**: Full Language Server Protocol (LSP) implementation with VSCode extension

## Project Structure

This is a Cargo workspace containing:

- **`crates/wtlang-core`**: Core library with lexer, parser, and AST
- **`crates/wtlang-compiler`**: The `wtc` compiler (WTLang → Python/Streamlit)
- **`crates/wtlang-lsp`**: Language Server Protocol implementation
- **`vscode-extension/`**: Visual Studio Code extension for WTLang
- **`examples/`**: Example WTLang programs
- **`doc/`**: Comprehensive documentation

## Quick Start

### Installation

```bash
# Build the compiler
cargo build --release -p wtlang-compiler

# The compiler binary will be at target/release/wtc
```

### Optional: Install Language Server & VSCode Extension

For IDE support with syntax highlighting, diagnostics, and auto-completion:

```bash
# Build the language server
cargo build --release -p wtlang-lsp

# Install and configure the VSCode extension
# See doc/lsp_installation.md for complete instructions
cd vscode-extension
npm install
npm run compile
npm run package
```

### Hello World

Create a file `hello.wt`:

```wtlang
page Home {
  title "Hello WTLang!"
  subtitle "My First Application"
  
  text "Welcome to WTLang - a language for creating table-based web applications."
}
```

Compile and run:

```bash
# Compile
wtc build hello.wt --output output

# Run the generated Streamlit app
cd output
pip install -r requirements.txt
streamlit run Home.py
```

## Example: User Management

```wtlang
table User {
  id: int [unique, non_null]
  name: string [non_null]
  email: string
  age: int
}

page UserList {
  title "User Management"
  subtitle "View and manage users"
  
  let users = load_csv("users.csv")
  
  section "All Users" {
    show users
    
    let total_users = count(users)
    text "Total: {total_users}"
  }
  
  section "Actions" {
    button "Export to Excel" {
      export_excel(users, "users_export.xlsx")
    }
  }
}
```

## Language Features

### Tables

Define typed table structures:

```wtlang
table Product {
  sku: string [unique, non_null]
  name: string [non_null]
  price: currency [validate(x => x > 0)]
  stock: int
}
```

### Function Chaining

Transform data with intuitive pipeline syntax:

```wtlang
let analysis = products
  -> filter(_, row => row.price > 100)
  -> sort_desc(_, "price")
  -> limit(_, 10)
```

### Multiple Pages

Create multi-page applications:

```wtlang
page Dashboard {
  title "Dashboard"
  // ...
}

page Settings {
  title "Settings"
  // ...
}
```

### Conditional Logic

```wtlang
let total = sum(orders, "amount")

if total > 10000 {
  text "🎉 Excellent sales!"
} else {
  text "Keep going!"
}
```

## Standard Library

WTLang includes functions for common data operations:

- **Loading**: `load_csv()`, `load_excel()`
- **Saving**: `save_csv()`, `export_excel()`
- **Filtering**: `filter()`, `where()`
- **Sorting**: `sort()`, `sort_desc()`
- **Aggregation**: `sum()`, `average()`, `count()`, `min()`, `max()`
- **Grouping**: `group_by()`
- **Joining**: `join()`, `left_join()`, `inner_join()`
- **Transformation**: `select()`, `add_column()`, `drop_column()`

## Documentation

- [Language Tutorial](doc/tutorial.md) - Complete guide to WTLang
- [Language Design](doc/language_design.md) - Design decisions and rationale
- [Target Platform](doc/target_platform_design.md) - Why Streamlit and deployment options
- [Compiler Design](doc/compiler_tools_design.md) - Compiler architecture and tooling
- [Implementation Notes](doc/IMPLEMENTATION.md) - Current implementation status
- [Examples](examples/) - Sample programs

## Project Structure

```
wtlang/
├── src/              # Compiler source code (Rust)
│   ├── main.rs       # CLI entry point
│   ├── lexer.rs      # Lexical analyzer
│   ├── parser.rs     # Parser
│   ├── ast.rs        # AST definitions
│   └── codegen.rs    # Code generator
├── examples/         # Example WTLang programs
├── doc/              # Documentation
└── Cargo.toml        # Rust project configuration
```

## Documentation

- **[Language Design](doc/language_design.md)**: Complete language specification
- **[Target Platform Design](doc/target_platform_design.md)**: Streamlit mapping details
- **[Compiler & Tools Design](doc/compiler_tools_design.md)**: Architecture and implementation
- **[Tutorial](doc/tutorial.md)**: Step-by-step guide to WTLang
- **[LSP Installation](doc/lsp_installation.md)**: Complete LSP and VSCode setup guide

## CLI Usage

### Build Command

Compile WTLang source to Python/Streamlit:

```bash
wtc build <input.wt> [-o <output-dir>]
```

Options:
- `-o, --output <DIR>`: Output directory (default: `output/`)

### Check Command

Check source for errors without generating code:

```bash
wtc check <input.wt>
```

## Examples

See the `examples/` directory:

- **01_hello.wt**: Simple "Hello World" page
- **02_tables.wt**: Loading and displaying CSV data
- **03_chaining.wt**: Function chaining and filtering
- **04_multi_page.wt**: Multi-page application
- **05_external_functions.wt**: External function declarations and imports

Each example includes sample data in `examples/data/`.

## Development Status

**Version 0.1** - First Implementation

✅ Completed:
- Lexer and tokenizer
- Parser with full AST support
- Code generator (Streamlit output)
- CLI with build and check commands
- Core language features (tables, pages, functions, chaining)
- Multi-tool architecture with shared core library
- **Language Server Protocol (LSP) implementation**
- **VSCode extension with syntax highlighting and diagnostics**
- Example programs and documentation

🚧 In Progress:
- Type checker and semantic analysis
- Error reporting improvements
- Extended standard library

📋 Planned:
- Testing framework (`test` blocks → pytest)
- Module/import system
- Advanced optimizations
- Hover information and go-to-definition in LSP
- Additional target platforms

## Requirements

**To build the compiler:**
- Rust 1.70+ and Cargo

**To run generated applications:**
- Python 3.8+
- Streamlit 1.28+
- pandas 2.0+

## Contributing

We welcome contributions! Areas where help is needed:

- Implementing additional standard library functions
- Improving error messages
- Adding more examples
- Writing tests
- Documentation improvements
- IDE/editor support

## License

MIT License - see LICENSE file for details

## Acknowledgments

WTLang is inspired by:
- Streamlit's simple approach to data apps
- Pandas' powerful data manipulation
- F#'s pipeline operator
- SQL's declarative data queries

## Contact

- GitHub Issues: [Report bugs or request features](https://github.com/yourusername/wtlang/issues)
- Documentation: [Full docs](doc/)
- Examples: [See examples/](examples/)

---

**Build data applications in minutes, not hours.** 🚀
