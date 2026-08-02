#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

echo "Formatting Rust workspace..."
cargo fmt --all

echo "Formatting active TypeScript and HTML sources..."
bunx biome format --write \
  apps/playground/index.html \
  apps/playground/tour/index.html \
  apps/playground/vite.config.ts \
  apps/playground/src/*.ts \
  apps/playground/src/compiler \
  apps/playground/src/diagnostics \
  apps/playground/src/editor \
  apps/playground/src/generated/tour-manifest.ts \
  apps/playground/src/runtime \
  apps/playground/src/tour \
  apps/playground/src/ui \
  apps/playground/src/workspace \
  apps/playground/tests \
  extensions/seseragi/extension.js \
  extensions/seseragi/extension-core.js \
  extensions/seseragi/scripts \
  extensions/seseragi/tests \
  extensions/seseragi-legacy/extension.js \
  scripts/check-samples-cli.ts \
  scripts/generate-playground-samples.ts \
  scripts/generate-playground-tour.ts \
  scripts/tour-curriculum.ts \
  scripts/tour-lessons.ts \
  scripts/check-extension-identity.ts \
  runtime/ts/src
