#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

BIOME="$ROOT/node_modules/.bin/biome"
PLAYGROUND_TSC="$ROOT/apps/playground/node_modules/.bin/tsc"
PLAYGROUND_VITE="$ROOT/apps/playground/node_modules/.bin/vite"
EXTENSION_ESBUILD="$ROOT/extensions/seseragi-spec-preview/node_modules/.bin/esbuild"
EXTENSION_VSCE="$ROOT/extensions/seseragi-spec-preview/node_modules/.bin/vsce"

usage() {
  cat <<'EOF'
Usage: scripts/check-scoped.sh <sample|playground|rust|conformance|wasm|extension|release|full> [args...]

Scoped lanes:
  sample       Native CLI samples, every sample compile/format, and manifest freshness
  playground   Sample checks plus Playground lint, tests, typecheck, and Vite build
  rust         Rust format and workspace (or forwarded target) tests
  conformance  Canonical conformance fixtures (optional path arguments)
  wasm         Regenerate committed Playground WASM and require no diff
  extension    Extension lint, tests, and local-platform package verification
  release      Version source, generated package metadata, and release contract tests
  full         Repository-wide integration gate
EOF
}

require_executable() {
  local executable="$1"
  local bootstrap="$2"
  if [[ ! -x "$executable" ]]; then
    echo "required local tool is missing: $executable" >&2
    echo "bootstrap it first with: $bootstrap" >&2
    exit 2
  fi
}

require_root_tools() {
  require_executable "$BIOME" "bun install --frozen-lockfile"
}

require_playground_tools() {
  require_executable "$PLAYGROUND_TSC" "cd apps/playground && bun install --frozen-lockfile"
  require_executable "$PLAYGROUND_VITE" "cd apps/playground && bun install --frozen-lockfile"
}

require_extension_tools() {
  require_executable "$EXTENSION_ESBUILD" "cd extensions/seseragi-spec-preview && bun install --frozen-lockfile"
  require_executable "$EXTENSION_VSCE" "cd extensions/seseragi-spec-preview && bun install --frozen-lockfile"
}

run_native_sample_checks() {
  echo "Checking runnable samples through the native CLI..."
  bun run test:samples:cli
}

run_sample_manifest_check() {
  echo "Checking the committed Playground sample manifest..."
  (
    cd apps/playground
    bun run samples:check
  )
}

run_sample_compiler_checks() {
  require_playground_tools
  echo "Compiling and formatting every canonical sample through committed WASM..."
  (
    cd apps/playground
    bun test tests/sample-compilation.test.ts
  )
}

run_sample_base_checks() {
  run_native_sample_checks
  run_sample_manifest_check
}

run_sample_checks() {
  run_sample_base_checks
  run_sample_compiler_checks
}

run_playground_lint() {
  require_root_tools
  echo "Linting Playground and sample catalog sources..."
  "$BIOME" lint \
    apps/playground/index.html \
    apps/playground/tour/index.html \
    apps/playground/vite.config.ts \
    apps/playground/playwright.config.ts \
    apps/playground/src/*.ts \
    apps/playground/src/compiler \
    apps/playground/src/diagnostics \
    apps/playground/src/editor \
    apps/playground/src/generated/tour-manifest.ts \
    apps/playground/src/generated/sample-manifest.ts \
    apps/playground/src/runtime \
    apps/playground/src/tour \
    apps/playground/src/ui \
    apps/playground/src/workspace \
    apps/playground/e2e \
    apps/playground/tests \
    scripts/check-samples-cli.ts \
    scripts/generate-playground-samples.ts \
    scripts/generate-playground-tour.ts \
    scripts/tour-curriculum.ts \
    scripts/tour-lessons.ts \
    scripts/release-contract.ts \
    scripts/release-contract.test.ts
}

run_playground_checks() {
  require_playground_tools
  run_sample_base_checks
  run_playground_lint

  echo "Running Playground tests..."
  (
    cd apps/playground
    bun run tour:check
    bun test tests
  )

  echo "Type-checking Playground TypeScript..."
  (
    cd apps/playground
    "$PLAYGROUND_TSC" --noEmit
  )

  echo "Building the Playground bundle..."
  (
    cd apps/playground
    "$PLAYGROUND_VITE" build
  )
}

run_rust_checks() {
  echo "Checking Rust formatting..."
  cargo fmt --all -- --check

  echo "Testing the Rust workspace..."
  if (($# == 0)); then
    cargo test --workspace
  else
    cargo test "$@"
  fi
}

run_conformance_checks() {
  require_root_tools
  require_playground_tools
  echo "Running canonical conformance fixtures..."
  if (($# == 0)); then
    cargo run -p seseragi-conformance -- .
  else
    cargo run -p seseragi-conformance -- "$@"
  fi
}

run_wasm_checks() {
  echo "Checking committed WASM freshness..."
  ./scripts/build-playground-wasm.sh apps/playground/src/wasm/pkg
  git diff --exit-code -- apps/playground/src/wasm/pkg
}

run_release_contract_check() {
  require_root_tools
  echo "Checking the canonical release contract..."
  bun scripts/release-contract.ts check

  echo "Testing release contract tooling..."
  "$BIOME" lint scripts/release-contract.ts scripts/release-contract.test.ts
  bun test scripts/release-contract.test.ts
}

run_extension_lint() {
  require_root_tools
  echo "Linting VS Code extension sources..."
  "$BIOME" lint \
    extensions/seseragi-spec-preview/extension.js \
    extensions/seseragi-spec-preview/extension-core.js \
    extensions/seseragi-spec-preview/scripts \
    extensions/seseragi-spec-preview/tests
}

run_extension_checks() {
  require_extension_tools
  run_extension_lint

  echo "Testing the VS Code extension..."
  (
    cd extensions/seseragi-spec-preview
    bun test tests
  )

  echo "Packaging and verifying the VS Code extension..."
  (
    cd extensions/seseragi-spec-preview
    bun scripts/package-extension.ts
  )
}

run_full_checks() {
  echo "Installing frozen root dependencies for the full gate..."
  bun install --frozen-lockfile

  echo "Installing frozen Playground dependencies for the full gate..."
  (
    cd apps/playground
    bun install --frozen-lockfile
  )

  require_root_tools
  require_playground_tools

  echo "Checking Rust formatting..."
  cargo fmt --all -- --check

  echo "Linting active TypeScript and HTML sources..."
  "$BIOME" lint \
    apps/playground/index.html \
    apps/playground/tour/index.html \
    apps/playground/vite.config.ts \
    apps/playground/playwright.config.ts \
    apps/playground/src/*.ts \
    apps/playground/src/compiler \
    apps/playground/src/diagnostics \
    apps/playground/src/editor \
    apps/playground/src/generated/tour-manifest.ts \
    apps/playground/src/runtime \
    apps/playground/src/tour \
    apps/playground/src/ui \
    apps/playground/src/workspace \
    apps/playground/e2e \
    apps/playground/tests \
    extensions/seseragi-spec-preview/extension.js \
    extensions/seseragi-spec-preview/extension-core.js \
    extensions/seseragi-spec-preview/scripts \
    extensions/seseragi-spec-preview/tests \
    scripts/check-samples-cli.ts \
    scripts/generate-playground-samples.ts \
    scripts/generate-playground-tour.ts \
    scripts/tour-curriculum.ts \
    scripts/tour-lessons.ts \
    scripts/release-contract.ts \
    scripts/release-contract.test.ts \
    runtime/ts/src

  echo "Testing Rust workspace..."
  cargo test --workspace

  run_conformance_checks
  run_native_sample_checks
  run_wasm_checks
  run_release_contract_check

  echo "Checking Playground catalog and Tour manifests..."
  (
    cd apps/playground
    bun run samples:check
    bun run tour:check
    bun test tests
    "$PLAYGROUND_TSC" --noEmit
    "$PLAYGROUND_VITE" build
  )

  echo "Packaging the VS Code extension..."
  bun run build:extension

  echo "All checks passed."
}

lane="${1:-}"
shift || true

case "$lane" in
  sample)
    (($# == 0)) || {
      echo "sample lane does not accept arguments" >&2
      exit 2
    }
    run_sample_checks
    ;;
  playground)
    (($# == 0)) || {
      echo "playground lane does not accept arguments" >&2
      exit 2
    }
    run_playground_checks
    ;;
  rust)
    run_rust_checks "$@"
    ;;
  conformance)
    run_conformance_checks "$@"
    ;;
  wasm)
    (($# == 0)) || {
      echo "wasm lane does not accept arguments" >&2
      exit 2
    }
    run_wasm_checks
    ;;
  extension)
    (($# == 0)) || {
      echo "extension lane does not accept arguments" >&2
      exit 2
    }
    run_extension_checks
    ;;
  release)
    (($# == 0)) || {
      echo "release lane does not accept arguments" >&2
      exit 2
    }
    run_release_contract_check
    ;;
  full)
    (($# == 0)) || {
      echo "full lane does not accept arguments" >&2
      exit 2
    }
    run_full_checks
    ;;
  -h|--help)
    usage
    ;;
  *)
    usage >&2
    exit 2
    ;;
esac
