#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "Checking Rust formatting..."
cargo fmt --all -- --check

echo "Linting active TypeScript and HTML sources..."
bunx biome lint \
  apps/playground/index.html \
  apps/playground/vite.config.ts \
  apps/playground/src/*.ts \
  apps/playground/src/compiler \
  apps/playground/src/diagnostics \
  apps/playground/src/editor \
  apps/playground/src/runtime \
  apps/playground/src/ui \
  apps/playground/tests \
  scripts/check-samples-cli.ts \
  scripts/generate-playground-samples.ts \
  runtime/ts/src

echo "Testing Rust workspace..."
cargo test --workspace

echo "Running canonical conformance fixtures..."
cargo run -p seseragi-conformance -- .

echo "Checking runnable samples through the native CLI..."
bun run test:samples:cli

echo "Installing frozen Playground dependencies..."
(
  cd apps/playground
  bun install --frozen-lockfile
)

echo "Checking committed WASM and Playground tests..."
bun run test:playground:wasm

echo "Type-checking and building Playground..."
(
  cd apps/playground
  bun run samples:check
  bun run typecheck
  bun run build
)

echo "Packaging the VS Code extension..."
bun run build:extension

echo "All checks passed."
