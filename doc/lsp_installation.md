# WTLang Language Server and VSCode Extension

This guide provides complete instructions for installing and using the WTLang Language Server with Visual Studio Code.

## Table of Contents

1. [Prerequisites](#prerequisites)
2. [Building the Language Server](#building-the-language-server)
3. [Installing the VSCode Extension](#installing-the-vscode-extension)
4. [Configuration](#configuration)
5. [Features](#features)
6. [Troubleshooting](#troubleshooting)
7. [References](#references)

## Prerequisites

- **Rust** (1.70 or later): [https://www.rust-lang.org/tools/install](https://www.rust-lang.org/tools/install)
- **Node.js** (18.x or later): [https://nodejs.org/](https://nodejs.org/)
- **Visual Studio Code** (1.75 or later): [https://code.visualstudio.com/](https://code.visualstudio.com/)
- **cargo** and **npm** should be available in your PATH

## Building the Language Server

The WTLang project uses a Cargo workspace structure with three main components:
- `wtlang-core`: Shared library with lexer, parser, and AST
- `wtlang-compiler`: The `wtc` compiler binary
- `wtlang-lsp`: The Language Server Protocol implementation

### Step 1: Build the LSP Server

From the root of the WTLang project:

```bash
# Build the LSP server in release mode (recommended)
cargo build --release -p wtlang-lsp

# OR build in debug mode for development
cargo build -p wtlang-lsp
```

This will create the executable at:
- Release: `target/release/wtlang-lsp.exe` (Windows) or `target/release/wtlang-lsp` (Unix)
- Debug: `target/debug/wtlang-lsp.exe` (Windows) or `target/debug/wtlang-lsp` (Unix)

### Step 2: Verify the Build

Test that the server runs:

```bash
# On Windows
target\release\wtlang-lsp.exe --help

# On Linux/Mac
./target/release/wtlang-lsp --help
```

The server communicates via stdin/stdout using the LSP protocol, so it won't show interactive output when run directly.

## Installing the VSCode Extension

### Option A: Install from Source (Development)

1. **Navigate to the extension directory:**

   ```bash
   cd vscode-extension
   ```

2. **Install dependencies:**

   ```bash
   npm install
   ```

3. **Compile the extension:**

   ```bash
   npm run compile
   ```

4. **Install the extension in VSCode:**

   ```bash
   # Create a symbolic link (recommended for development)
   # On Windows (run as Administrator):
   mklink /D "%USERPROFILE%\.vscode\extensions\wtlang-0.1.0" "%CD%"

   # On Linux/Mac:
   ln -s "$(pwd)" ~/.vscode/extensions/wtlang-0.1.0
   ```

   **OR package and install:**

   ```bash
   # Package the extension
   npm run package

   # This creates wtlang-0.1.0.vsix
   # Install in VSCode: Extensions > ... menu > Install from VSIX
   ```

5. **Reload VSCode:**

   Press `Ctrl+Shift+P` (or `Cmd+Shift+P` on Mac) and run "Developer: Reload Window"

### Option B: Package and Install (Production)

1. **Install vsce (if not already installed):**

   ```bash
   npm install -g @vscode/vsce
   ```

2. **Navigate to the extension directory and package:**

   ```bash
   cd vscode-extension
   npm install
   npm run compile
   vsce package
   ```

3. **Install the .vsix file in VSCode:**
   - Open VSCode
   - Go to Extensions view (`Ctrl+Shift+X`)
   - Click the "..." menu at the top
   - Select "Install from VSIX..."
   - Choose `wtlang-0.1.0.vsix`

## Configuration

### Setting the LSP Server Path

By default, the extension looks for `wtlang-lsp` in:
1. The workspace's `target/release/wtlang-lsp.exe`
2. The workspace's `target/debug/wtlang-lsp.exe`
3. Your system PATH

To specify a custom path:

1. Open VSCode Settings (`Ctrl+,`)
2. Search for "wtlang"
3. Set "WTLang: Server Path" to the full path of your `wtlang-lsp` executable

**Example (Windows):**
```
C:\Users\YourName\creazioni\wtlang\target\release\wtlang-lsp.exe
```

**Example (Linux/Mac):**
```
/home/yourname/wtlang/target/release/wtlang-lsp
```

### VSCode settings.json

Alternatively, add to `.vscode/settings.json` in your workspace:

```json
{
  "wtlang.server.path": "C:\\Users\\YourName\\creazioni\\wtlang\\target\\release\\wtlang-lsp.exe",
  "wtlang.trace.server": "messages"
}
```

The `trace.server` setting can be:
- `"off"`: No tracing (default)
- `"messages"`: Log LSP messages
- `"verbose"`: Detailed logging

## Features

The WTLang Language Server provides the following features:

### 1. Syntax Highlighting

Automatic syntax highlighting for `.wt` files, including:
- Keywords: `page`, `table`, `from`, `display`, `button`, `input`, `if`, `else`, `test`
- Types: `number`, `text`, `date`, `boolean`
- Operators: `->`, `==`, `!=`, `<=`, `>=`, `+`, `-`, `*`, `/`, etc.
- Functions: `filter`, `sort`, `aggregate`, `unique`
- Comments: `// comment`
- Strings and numbers

### 2. Real-time Diagnostics

As you type, the server analyzes your code and reports:
- **Lexical errors**: Invalid tokens, malformed strings
- **Syntax errors**: Missing semicolons, unmatched braces, incorrect grammar
- Errors appear in the Problems panel (`Ctrl+Shift+M`)

### 3. Auto-completion

Trigger with `Ctrl+Space` to see:
- Language keywords
- Built-in types
- Standard library functions

### 4. Hover Information

Hover over code elements to see:
- Type information (coming soon)
- Function signatures (coming soon)
- Documentation (coming soon)

### 5. Go to Definition

Right-click on a symbol and select "Go to Definition" or press `F12` to jump to:
- Table definitions
- Page declarations
- Function definitions (coming soon)

## Troubleshooting

### Extension Not Activating

**Problem:** The extension doesn't activate when opening `.wt` files.

**Solutions:**
1. Check the Output panel: View > Output, then select "WTLang Language Server" from the dropdown
2. Verify the extension is installed: Extensions view (`Ctrl+Shift+X`), search for "WTLang"
3. Reload VSCode: `Ctrl+Shift+P` > "Developer: Reload Window"

### Server Not Found

**Problem:** Error message "wtlang-lsp not found" or server fails to start.

**Solutions:**
1. Verify the server is built: `cargo build --release -p wtlang-lsp`
2. Check the path in settings: VSCode Settings > "WTLang: Server Path"
3. Ensure the executable has proper permissions (Unix: `chmod +x target/release/wtlang-lsp`)
4. Try using an absolute path in the settings

### No Diagnostics Appearing

**Problem:** Errors in code are not highlighted.

**Solutions:**
1. Check the trace output: Set `"wtlang.trace.server": "verbose"` in settings
2. Verify the file is saved with `.wt` extension
3. Check the Problems panel (`Ctrl+Shift+M`) for any server errors
4. Restart the language server: `Ctrl+Shift+P` > "Developer: Reload Window"

### Compilation Errors

**Problem:** TypeScript or Rust compilation errors when building.

**Solutions:**
1. **Rust:** Ensure you have Rust 1.70+ installed: `rustc --version`
2. **Node:** Ensure you have Node.js 18+ installed: `node --version`
3. Delete build artifacts and rebuild:
   ```bash
   # Rust
   cargo clean
   cargo build --release
   
   # VSCode extension
   cd vscode-extension
   rm -rf node_modules out
   npm install
   npm run compile
   ```

## References

### Official Documentation

1. **Language Server Protocol (LSP) Specification:**
   - [LSP Overview](https://microsoft.github.io/language-server-protocol/overview)
   - [LSP Specification](https://microsoft.github.io/language-server-protocol/specifications/lsp/3.17/specification/)
   - Understanding the protocol that powers modern IDE features

2. **VSCode Extension API:**
   - [Extension API Overview](https://code.visualstudio.com/api)
   - [Language Extensions Guide](https://code.visualstudio.com/api/language-extensions/overview)
   - [Programmatic Language Features](https://code.visualstudio.com/api/language-extensions/programmatic-language-features)

3. **VSCode Extension Development:**
   - [Your First Extension](https://code.visualstudio.com/api/get-started/your-first-extension)
   - [Extension Anatomy](https://code.visualstudio.com/api/get-started/extension-anatomy)
   - [Publishing Extensions](https://code.visualstudio.com/api/working-with-extensions/publishing-extension)

4. **Language Server Implementation:**
   - [LSP Sample (TypeScript)](https://github.com/microsoft/vscode-extension-samples/tree/main/lsp-sample)
   - [Language Server Extension Guide](https://code.visualstudio.com/api/language-extensions/language-server-extension-guide)

5. **Rust tower-lsp Library:**
   - [tower-lsp Documentation](https://docs.rs/tower-lsp/)
   - [tower-lsp GitHub](https://github.com/ebkalderon/tower-lsp)
   - The async LSP framework used by WTLang

### WTLang-Specific Documentation

- See `doc/language_design.md` for language specification
- See `doc/compiler_tools_design.md` for architecture details
- See `doc/tutorial.md` for language tutorial

### Additional Resources

- [TextMate Grammar Guide](https://macromates.com/manual/en/language_grammars)
- [VSCode API Reference](https://code.visualstudio.com/api/references/vscode-api)
- [Language Configuration Guide](https://code.visualstudio.com/api/language-extensions/language-configuration-guide)

## Development Workflow

For contributors working on the LSP:

1. **Make changes to the server:**
   ```bash
   # Edit files in crates/wtlang-lsp/src/
   cargo build -p wtlang-lsp
   ```

2. **Make changes to the extension:**
   ```bash
   cd vscode-extension
   # Edit src/extension.ts or other files
   npm run compile
   ```

3. **Test changes:**
   - Reload VSCode window: `Ctrl+Shift+P` > "Developer: Reload Window"
   - Or press `F5` to launch Extension Development Host

4. **View logs:**
   - Server logs: Output panel > "WTLang Language Server"
   - Extension logs: Help > Toggle Developer Tools > Console

## Next Steps

- Implement hover information with type details
- Add go-to-definition for tables and pages
- Implement semantic highlighting
- Add code formatting support
- Implement refactoring actions
- Add workspace symbols support

For issues or contributions, see the project repository.
