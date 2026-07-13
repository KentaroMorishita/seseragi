#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"

if ! command -v wasm-pack >/dev/null 2>&1; then
  echo "wasm-pack is required to build the Rust playground adapter" >&2
  exit 1
fi

if command -v brew >/dev/null 2>&1; then
  RUSTUP_PREFIX="$(brew --prefix rustup 2>/dev/null || true)"
  if [[ -x "$RUSTUP_PREFIX/bin/rustup" ]]; then
    export PATH="$RUSTUP_PREFIX/bin:$PATH"
  fi
fi

if command -v rustup >/dev/null 2>&1; then
  rustup target add wasm32-unknown-unknown >/dev/null
fi

wasm-pack build "$ROOT/crates/seseragi-wasm" \
  --target web \
  --out-dir "$ROOT/playground/src/wasm/pkg" \
  --out-name seseragi_wasm \
  --release
