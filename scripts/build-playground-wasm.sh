#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
OUT_DIR="${1:-apps/playground/src/wasm/pkg}"
BUILD_OPTIONS=()
if [[ -n "${SESERAGI_WASM_BUILD_EVIDENCE:-}" ]]; then
  mkdir -p "$SESERAGI_WASM_BUILD_EVIDENCE"
  exec > >(tee "$SESERAGI_WASM_BUILD_EVIDENCE/build.log") 2>&1
  BUILD_OPTIONS+=(--verbose --verbose)
  export CC_ENABLE_DEBUG_OUTPUT=1
  export RUSTFLAGS="${RUSTFLAGS:+$RUSTFLAGS }--print=link-args"
  git -C "$ROOT" rev-parse HEAD 'HEAD^{tree}'
  shasum -a 256 "$ROOT/Cargo.lock" "$ROOT/Cargo.toml" "$ROOT/rust-toolchain.toml"
  rustc --version --verbose
  cargo --version --verbose
fi

if [[ "$OUT_DIR" != /* ]]; then
  OUT_DIR="$ROOT/$OUT_DIR"
fi

if ! command -v wasm-pack >/dev/null 2>&1; then
  echo "wasm-pack is required to build the Rust playground adapter" >&2
  exit 1
fi

EXPECTED_WASM_PACK_VERSION="0.15.0"
ACTUAL_WASM_PACK_VERSION="$(wasm-pack --version | awk '{print $2}')"
if [[ -n "${SESERAGI_WASM_BUILD_EVIDENCE:-}" ]]; then
  echo "wasm-pack $ACTUAL_WASM_PACK_VERSION"
fi
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

RUST_CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"
RUST_TARGET_DIR="${CARGO_TARGET_DIR:-$ROOT/target}"
if [[ "$RUST_TARGET_DIR" != /* ]]; then
  RUST_TARGET_DIR="$ROOT/$RUST_TARGET_DIR"
fi
export RUSTFLAGS="${RUSTFLAGS:+$RUSTFLAGS }--remap-path-prefix=$ROOT=/workspace --remap-path-prefix=$RUST_CARGO_HOME=/cargo --remap-path-prefix=$RUST_TARGET_DIR=/workspace/target"

bun "$ROOT/scripts/run-portable-parser.ts" wasm-pack build "$ROOT/crates/seseragi-wasm" \
  --mode no-install \
  --target web \
  --out-dir "$OUT_DIR" \
  --out-name seseragi_wasm \
  --release \
  --no-opt \
  -- "${BUILD_OPTIONS[@]}"

cp "$ROOT/runtime/unicode/LICENSE" "$OUT_DIR/UNICODE-LICENSE"

# wasm-pack 0.15 resolves wasm-opt as "latest", independently of the pinned
# wasm-pack version. Keep the committed artifact reproducible across fresh
# release runners; static hosting remains responsible for transport compression.

# wasm-pack treats publishable packages as ignored by default. The new
# playground deliberately versions this target-neutral deployment artifact so
# Vercel never needs a Rust toolchain during its static-site build.
rm -f "$OUT_DIR/.gitignore"

if [[ -n "${SESERAGI_WASM_BUILD_EVIDENCE:-}" ]]; then
  bun "$ROOT/scripts/collect-wasm-build-evidence.ts" \
    "$RUST_TARGET_DIR" "$SESERAGI_WASM_BUILD_EVIDENCE"
fi
