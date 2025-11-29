# WTLang VSCode Extension

Visual Studio Code extension for the WTLang programming language.

## Features

- **Syntax Highlighting**: Full syntax highlighting for `.wt` files
- **Real-time Diagnostics**: Instant error detection as you type
- **Auto-completion**: Keyword and function completion
- **Hover Information**: Type and documentation tooltips (coming soon)
- **Go to Definition**: Jump to table and page definitions (coming soon)

## Installation

See the main [LSP Installation Guide](../doc/lsp_installation.md) for complete setup instructions.

### Quick Start

1. Build the language server:
   ```bash
   cd ..
   cargo build --release -p wtlang-lsp
   ```

2. Install extension dependencies:
   ```bash
   npm install
   ```

3. Compile the extension:
   ```bash
   npm run compile
   ```

4. Package and install:
   ```bash
   npm run package
   # Then install the .vsix file in VSCode
   ```

## Configuration

Configure the extension in VSCode settings:

- **wtlang.server.path**: Path to the `wtlang-lsp` executable
- **wtlang.trace.server**: Set to "messages" or "verbose" for debugging

## Development

To work on the extension:

1. Install dependencies: `npm install`
2. Make changes to `src/extension.ts`
3. Compile: `npm run compile`
4. Press `F5` to launch Extension Development Host
5. Test your changes

## File Structure

- `src/extension.ts`: Extension entry point and LSP client setup
- `syntaxes/wtlang.tmLanguage.json`: TextMate grammar for syntax highlighting
- `language-configuration.json`: Language configuration (brackets, comments, etc.)
- `package.json`: Extension manifest

## Requirements

- Node.js 18.x or later
- TypeScript 5.x
- VSCode 1.75 or later

## License

Same as the WTLang project.
