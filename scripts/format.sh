#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "Formatting Rust workspace..."
cargo fmt --all

echo "Formatting active TypeScript and HTML sources..."
bunx biome format --write \
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
