#!/bin/bash
set -e

echo "🔍 Checking (format + lint)..."
bunx biome check --write src tests playground/src

echo "🏷️ Type checking..."
bunx tsc --noEmit --project tsconfig.test.json

echo "🧪 Running tests..."
bun test

echo "✅ All checks passed!"