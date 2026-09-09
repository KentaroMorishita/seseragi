#!/usr/bin/env bash
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

BIOME="$ROOT/node_modules/.bin/biome"
PLAYGROUND_TSC="$ROOT/apps/playground/node_modules/.bin/tsc"
EXTENSION_ESBUILD="$ROOT/extensions/seseragi/node_modules/.bin/esbuild"
EXTENSION_VSCE="$ROOT/extensions/seseragi/node_modules/.bin/vsce"

# Full gates collect independent failures; scoped lanes retain fail-fast behavior.
CHECK_FAILURES=()
COLLECT_CHECK_FAILURES=0
check_step() {
  if [[ "$COLLECT_CHECK_FAILURES" != 1 ]]; then
    "$@"
    return
  fi
  local result=0
  "$@" || result=$?
  if ((result != 0)); then
    CHECK_FAILURES+=("$* (exit $result)")
    printf 'FAILED: %s (exit %s)\n' "$*" "$result" >&2
  fi
  return 0
}

usage() {
  cat <<'EOF'
Usage: scripts/check-scoped.sh <sample|playground|rust|conformance|production|wasm|extension|release|release-gate|release-gate-after-wasm|full> [args...]

Scoped lanes:
  sample       Native CLI samples, every sample compile/format, and manifest freshness
  playground   Sample checks plus Playground lint, tests, typecheck, and Vite build
  rust         Rust format and workspace (or forwarded target) tests
  conformance  Canonical conformance fixtures (optional path arguments)
  production   First-party production artifact shape, budgets, and identity
  wasm         Regenerate committed Playground WASM and require no diff
  extension    Extension lint, tests, and local-platform package verification
  release      Version source, generated package metadata, and release contract tests
  release-gate Repository-wide source gate; release artifact packaging stays in matrix jobs
  release-gate-after-wasm
               Release source gate after the same job verified committed WASM freshness
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
}

require_extension_tools() {
  require_executable "$EXTENSION_ESBUILD" "cd extensions/seseragi && bun install --frozen-lockfile"
  require_executable "$EXTENSION_VSCE" "cd extensions/seseragi && bun install --frozen-lockfile"
}

run_native_sample_checks() {
  echo "Checking runnable samples through the native CLI..."
  check_step bun run test:samples:cli
}

run_sample_manifest_check() {
  echo "Checking the committed Playground sample manifest..."
  pushd apps/playground >/dev/null
    check_step bun run samples:check
  popd >/dev/null
}

run_sample_compiler_checks() {
  require_playground_tools
  echo "Compiling and formatting every canonical sample through committed WASM..."
  pushd apps/playground >/dev/null
    check_step bun test tests/sample-compilation.test.ts
  popd >/dev/null
}

run_production_checks() {
  cargo build --locked -p seseragi-cli || return $?
  cargo test --locked -p seseragi-cli --test production || return $?
  bun test scripts/production-artifacts.test.ts || return $?
  bun scripts/check-production-artifacts.ts
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
  check_step "$BIOME" lint \
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
    scripts/check-project-fixtures.ts \
    scripts/check-project-fixtures.test.ts \
    scripts/check-playground-isolated-build.ts \
    scripts/generate-playground-runtime.ts \
    scripts/generate-playground-samples.ts \
    scripts/generate-playground-tour.ts \
    scripts/tour-curriculum.ts \
    scripts/tour-lessons.ts \
    scripts/release-contract.ts \
    scripts/release-contract.test.ts
}

run_playground_bundle_build() {
  echo "Building the Playground bundle from an isolated dependency install..."
  check_step bun scripts/check-playground-isolated-build.ts
}

run_playground_checks() {
  require_playground_tools
  run_sample_base_checks
  run_playground_lint

  echo "Running Playground tests..."
  pushd apps/playground >/dev/null
    check_step bun run tour:check
    check_step bun run runtime:check
    check_step bun test tests
  popd >/dev/null

  echo "Type-checking Playground TypeScript..."
  pushd apps/playground >/dev/null
    check_step "$PLAYGROUND_TSC" --noEmit
  popd >/dev/null

  run_playground_bundle_build
}

run_cargo_tests() {
  local -a cargo_args=("$@")
  if (($# == 0)); then
    cargo_args=(--workspace)
  fi

  if [[ "$(uname -s)" == "Darwin" && $# == 0 ]]; then
    local artifacts
    artifacts="$(mktemp -t seseragi-cargo-tests.XXXXXX)"
    if ! cargo test --no-run --message-format=json "${cargo_args[@]}" >"$artifacts"; then
      rm -f "$artifacts"
      return 1
    fi
    local test_result=0
    bun "$ROOT/scripts/run-macos-cargo-tests.ts" "$artifacts" || test_result=1
    rm -f "$artifacts"
    cargo test --doc --no-fail-fast "${cargo_args[@]}" || test_result=1
    return "$test_result"
  fi

  cargo test --no-fail-fast "${cargo_args[@]}"
}

run_rust_checks() {
  echo "Checking Rust formatting..."
  check_step cargo fmt --all -- --check

  echo "Testing the Rust workspace..."
  check_step run_cargo_tests "$@"
}

run_conformance_checks() {
  require_root_tools
  require_root_tools
  require_playground_tools
  echo "Checking project fixture roles and availability..."
  check_step bun run fixtures:check
  echo "Checking the bundled PostgreSQL external Provider..."
  check_step bun run postgres:bundle:check
  echo "Checking the pinned timezone database bundle..."
  check_step bun run timezones:bundle:check
  check_step bun test runtime/providers/timezones.test.ts
  echo "Type-checking TypeScript runtime Providers..."
  check_step "$PLAYGROUND_TSC" --noEmit -p "$ROOT/runtime/providers/tsconfig.json"
  echo "Testing all TypeScript runtime boundaries..."
  check_step bun test runtime/ts/tests
  check_step "$BIOME" lint runtime/ts/src/benchmark.ts runtime/ts/src/benchmark-runner.ts runtime/ts/src/benchmark-baseline.ts runtime/ts/src/benchmark-host.ts runtime/ts/fixtures/benchmark-quality runtime/ts/tests/benchmark.test.ts runtime/ts/tests/benchmark-host.test.ts runtime/ts/tests/benchmark-quality.test.ts
  echo "Testing release output shapes..."
  check_step bun test scripts/release-shapes.test.ts
  echo "Checking pinned Unicode data and text conformance..."
  check_step bun run unicode:check
  echo "Running canonical conformance fixtures..."
  if (($# == 0)); then
    check_step cargo run -p seseragi-conformance -- .
  else
    check_step cargo run -p seseragi-conformance -- "$@"
  fi
}

run_wasm_checks() {
  echo "Checking committed WASM freshness..."
  check_step ./scripts/build-playground-wasm.sh apps/playground/src/wasm/pkg
  check_step git diff --exit-code -- apps/playground/src/wasm/pkg
}

run_release_contract_metadata_check() {
  require_root_tools
  echo "Checking the canonical release contract..."
  check_step bun scripts/release-contract.ts check
  check_step bun scripts/release-readiness.ts check
  check_step bun scripts/check-extension-identity.ts

  echo "Testing release contract tooling..."
  check_step "$BIOME" lint \
    scripts/release-contract.ts \
    scripts/release-contract.test.ts \
    scripts/release-gate.ts \
    scripts/release-gate.test.ts \
    scripts/release-readiness.ts \
    scripts/release-readiness.test.ts \
    scripts/native-release.ts \
    scripts/check-generated-types.ts \
    scripts/release-shapes.ts \
    scripts/release-shapes.test.ts \
    scripts/linux-native-abi.ts \
    scripts/linux-native-smoke.ts \
    scripts/native-release.test.ts \
    scripts/linux-native-abi.test.ts \
    scripts/release-promotion.ts \
    scripts/release-promotion.test.ts \
    scripts/local-dogfood.ts \
    scripts/local-dogfood.test.ts \
    scripts/local-web-product-e2e.ts \
    scripts/local-web-product-e2e-extension.cjs \
    scripts/local-web-product-e2e.test.ts
  check_step bun test \
    scripts/release-contract.test.ts \
    scripts/release-gate.test.ts \
    scripts/release-readiness.test.ts \
    scripts/native-release.test.ts \
    scripts/linux-native-abi.test.ts \
    scripts/release-promotion.test.ts \
    scripts/local-dogfood.test.ts \
    scripts/local-web-product-e2e.test.ts
}

run_release_contract_check() {
  run_release_contract_metadata_check

  echo "Packaging and re-extracting the host native release archive..."
  check_step bash -c 'cargo build --locked --release -p seseragi-cli -p seseragi-lsp && bun scripts/native-release.ts smoke'
}

run_extension_lint() {
  require_root_tools
  echo "Linting VS Code extension sources..."
  check_step "$BIOME" lint \
    extensions/seseragi/extension.js \
    extensions/seseragi/extension-core.js \
    extensions/seseragi/scripts \
    extensions/seseragi/tests \
    extensions/seseragi-legacy/extension.js \
    scripts/check-extension-identity.ts
}

run_extension_behavior_checks() {
  echo "Checking official extension identity and legacy references..."
  check_step bun scripts/check-extension-identity.ts

  echo "Testing the VS Code extension..."
  pushd extensions/seseragi >/dev/null
    check_step bun test tests
  popd >/dev/null
}

run_extension_checks() {
  require_extension_tools
  run_extension_lint
  run_extension_behavior_checks

  echo "Packaging and verifying the VS Code extension..."
  pushd extensions/seseragi >/dev/null
    check_step bun scripts/package-extension.ts
    check_step bun scripts/package-legacy.ts
  popd >/dev/null
}

run_full_checks() {
  local artifact_mode="${1:-package}"
  local wasm_mode="${2:-check}"
  if [[ "$artifact_mode" != "package" && "$artifact_mode" != "delegate" ]]; then
    echo "invalid full gate artifact mode: $artifact_mode" >&2
    exit 2
  fi
  if [[ "$wasm_mode" != "check" && "$wasm_mode" != "verified" ]]; then
    echo "invalid full gate WASM mode: $wasm_mode" >&2
    exit 2
  fi

  echo "Installing frozen root dependencies for the full gate..."
  bun install --frozen-lockfile

  echo "Installing frozen Playground dependencies for the full gate..."
  (
    cd apps/playground
    bun install --frozen-lockfile
  )

  echo "Installing frozen extension dependencies for the full gate..."
  (
    cd extensions/seseragi
    bun install --frozen-lockfile
  )

  require_root_tools
  require_playground_tools
  require_extension_tools
  COLLECT_CHECK_FAILURES=1

  check_step bun test scripts/check-collection.test.ts
  check_step bun scripts/check-readme.ts
  echo "Checking Rust formatting..."
  check_step cargo fmt --all -- --check

  echo "Linting active TypeScript and HTML sources..."
  check_step "$BIOME" lint \
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
    scripts/check-project-fixtures.ts \
    scripts/check-project-fixtures.test.ts \
    scripts/postgres-provider-bundle.ts \
    scripts/timezone-bundle.ts \
    scripts/run-macos-cargo-tests.ts \
    scripts/native-release.ts \
    scripts/check-generated-types.ts \
    scripts/release-shapes.ts \
    scripts/release-shapes.test.ts \
    scripts/linux-native-abi.ts \
    scripts/linux-native-smoke.ts \
    scripts/native-release.test.ts \
    scripts/linux-native-abi.test.ts \
    scripts/local-web-product-e2e.ts \
    scripts/local-web-product-e2e-extension.cjs \
    scripts/local-web-product-e2e.test.ts \
    scripts/release-contract.ts \
    scripts/release-contract.test.ts \
    scripts/release-gate.ts \
    scripts/release-gate.test.ts \
    scripts/release-readiness.ts \
    scripts/release-readiness.test.ts \
    runtime/ts/src \
    runtime/ts/tests \
    runtime/providers/browser \
    runtime/providers/bun \
    runtime/providers/node \
    runtime/providers/postgres/adapter.ts \
    runtime/providers/postgres/pg.ts \
    runtime/providers/sqlite \
    runtime/providers/filesystem.ts \
    runtime/providers/http-client.ts \
    runtime/providers/timezones.ts \
    runtime/providers/timezones.test.ts \
    runtime/timezones/rules.ts

  echo "Testing Rust workspace..."
  check_step run_cargo_tests

  run_conformance_checks
  check_step run_production_checks
  check_step bun scripts/check-production-browser.ts
  run_native_sample_checks
  if [[ "$wasm_mode" == "check" ]]; then
    run_wasm_checks
  else
    echo "Skipping committed WASM freshness in this release gate."
  fi
  if [[ "$artifact_mode" == "delegate" ]]; then
    run_release_contract_metadata_check
  else
    run_release_contract_check
  fi

  echo "Checking Playground catalog and Tour manifests..."
  pushd apps/playground >/dev/null
    check_step bun run samples:check
    check_step bun run tour:check
    check_step bun test tests
    check_step "$PLAYGROUND_TSC" --noEmit
  popd >/dev/null
  run_playground_bundle_build

  if [[ "$artifact_mode" == "delegate" ]]; then
    echo "Checking the VS Code extension contract..."
    run_extension_behavior_checks
    echo "Release artifact packaging is delegated to the SHA-pinned matrix jobs."
  else
    echo "Packaging the VS Code extension..."
    check_step bun run build:extension
  fi

  if ((${#CHECK_FAILURES[@]} != 0)); then
    printf '\nFull gate failures (%s):\n' "${#CHECK_FAILURES[@]}" >&2
    printf ' - %s\n' "${CHECK_FAILURES[@]}" >&2
    return 1
  fi
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
  production)
    run_production_checks
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
  release-gate)
    (($# == 0)) || {
      echo "release-gate lane does not accept arguments" >&2
      exit 2
    }
    run_full_checks delegate
    ;;
  release-gate-after-wasm)
    (($# == 0)) || {
      echo "release-gate-after-wasm lane does not accept arguments" >&2
      exit 2
    }
    run_full_checks delegate verified
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
