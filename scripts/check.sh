#!/bin/bash
set -e

echo "✨ Formatting code..."
bunx biome format --write src tests

echo "🔍 Linting..."
bunx biome lint src tests

echo "🏷️ Type checking..."
bunx tsc --noEmit --project tsconfig.test.json

echo "🧪 Running tests..."
bun test

echo "✅ All checks passed!"