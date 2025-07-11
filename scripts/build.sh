#!/bin/bash
set -e

echo "🧹 Cleaning..."
rm -rf dist

echo "🏗️ Building CLI..."
bun build src/cli.ts --outdir dist --target node

echo "🔧 Building LSP server..."
bun build src/lsp/main.ts --outdir dist/lsp --target node

echo "🔨 Building VS Code extension..."
./scripts/vscode.sh

echo "✅ All builds complete!"