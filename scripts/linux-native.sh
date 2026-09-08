#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
mode="${1:?expected build or verify}"
shift
image="$(bun -p 'require("./scripts/linux-native-contract.json").image')"
bun_binary="$(command -v bun)"
args=(run --rm --platform linux/amd64 -v "$PWD:/workspace" -v "$bun_binary:/usr/local/bin/bun:ro" -w /workspace)
if [[ "$mode" == build ]]; then
  args+=(-v "$(rustc --print sysroot):/opt/rust:ro")
elif [[ "$mode" != verify ]]; then
  echo 'expected build or verify' >&2
  exit 2
fi
"${CONTAINER_ENGINE:-docker}" "${args[@]}" "$image" bash scripts/linux-native-container.sh "$mode" "$@"
