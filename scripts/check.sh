#!/bin/bash
set -e

# フォーマット実行
./scripts/format.sh

echo "🔍 Linting..."
bunx biome lint src tests

echo "🏷️ Type checking..."
bunx tsc --noEmit --project tsconfig.test.json

echo "🧪 Running tests..."
bun test

echo "✅ All checks passed!"