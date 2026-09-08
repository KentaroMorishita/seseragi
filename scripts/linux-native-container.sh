#!/usr/bin/env bash
set -euo pipefail
cd /workspace
# Install tools in the disposable container; no compiler or host library is
# copied into the downloaded-artifact verification environment.
if [[ "$1" == build ]]; then
  dnf -y install gcc git binutils tar gzip
  git config --global --add safe.directory /workspace
  export PATH="/opt/rust/bin:$PATH"
  export CARGO_INCREMENTAL=0
  export CARGO_TARGET_DIR=/workspace/target/linux-native
  cargo build --locked --release --target x86_64-unknown-linux-gnu -p seseragi-cli -p seseragi-lsp
elif [[ "$1" == verify ]]; then
  dnf -y install binutils tar gzip
else
  echo 'expected build or verify' >&2
  exit 2
fi
expected="$(bun -p 'require("./scripts/linux-native-contract.json").glibcMaximum')"
test "$(getconf GNU_LIBC_VERSION)" = "glibc $expected"
if [[ "$1" == build ]]; then
  bun scripts/linux-native-abi.ts target/linux-native/x86_64-unknown-linux-gnu/release/seseragi target/linux-native/x86_64-unknown-linux-gnu/release/seseragi-lsp
else
  shift
  bun scripts/linux-native-smoke.ts "$@"
fi
