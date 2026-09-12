#!/usr/bin/env bash
set -euo pipefail

repository_root="$(cd "$(dirname "$0")/../../.." && pwd)"
docs_origin="${SESERAGI_DOCS_ORIGIN:?Set SESERAGI_DOCS_ORIGIN to the canonical HTTPS origin}"
docs_base="${SESERAGI_DOCS_BASE:-/}"
docs_output="${SESERAGI_DOCS_OUTPUT:-$repository_root/target/docs-site}"

cd "$repository_root"
CARGO_INCREMENTAL=0 cargo build --locked --release -p seseragi-cli
SESERAGI_BIN="$repository_root/target/release/seseragi" \
  bun apps/docs/scripts/production.ts "$docs_output" "$docs_origin" "$docs_base"
