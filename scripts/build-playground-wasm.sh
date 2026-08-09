#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="${1:-apps/playground/src/wasm/pkg}"

if [[ "$OUT_DIR" != /* ]]; then
  OUT_DIR="$ROOT/$OUT_DIR"
fi

if ! command -v wasm-pack >/dev/null 2>&1; then
  echo "wasm-pack is required to build the Rust playground adapter" >&2
  exit 1
fi

EXPECTED_WASM_PACK_VERSION="0.15.0"
ACTUAL_WASM_PACK_VERSION="$(wasm-pack --version | awk '{print $2}')"
if [[ "$ACTUAL_WASM_PACK_VERSION" != "$EXPECTED_WASM_PACK_VERSION" ]]; then
  echo "wasm-pack $EXPECTED_WASM_PACK_VERSION is required; found $ACTUAL_WASM_PACK_VERSION" >&2
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
  --out-dir "$OUT_DIR" \
  --out-name seseragi_wasm \
  --release

# wasm-pack treats publishable packages as ignored by default. The new
# playground deliberately versions this target-neutral deployment artifact so
# Vercel never needs a Rust toolchain during its static-site build.
rm -f "$OUT_DIR/.gitignore"
