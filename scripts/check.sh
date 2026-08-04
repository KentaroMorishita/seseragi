#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

bun scripts/check-readme.ts
exec "$ROOT/scripts/check-scoped.sh" full "$@"
