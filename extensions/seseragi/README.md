# Seseragi Language Support

Language support for Seseragi - a programming language that compiles to TypeScript.

## Features

- **Syntax Highlighting** - Rich syntax highlighting for `.ssrg` files
- **Error Diagnostics** - Real-time error detection and type checking
- **IntelliSense** - Code completion and hover information
- **Auto-formatting** - Format code automatically on save
- **Language Server** - Powered by Hindley-Milner type inference


## Installation

Install this extension from the [VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=seseragi-dev.seseragi-language-support).

## Requirements

- [Bun](https://bun.sh) runtime

## Extension Settings

This extension contributes the following settings:

- `seseragi.maxNumberOfProblems`: Controls the maximum number of problems produced by the server (default: 100)
- `seseragi.trace.server`: Traces the communication between VS Code and the language server

## File Extensions

This extension provides language support for files with the following extensions:

- `.ssrg` - Seseragi source files

## Commands

This extension contributes the following commands:

- `seseragi.format`: Format current Seseragi file

## Known Issues

Please report issues on [GitHub](https://github.com/KentaroMorishita/seseragi/issues).

## Contributing

See the [repository](https://github.com/KentaroMorishita/seseragi) for contribution guidelines.


**Enjoy!**