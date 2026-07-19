#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "Building Rust workspace..."
cargo build --workspace

echo "Building Playground..."
(
  cd apps/playground
  bun install --frozen-lockfile
  bun run build
)

echo "Build complete."
