#!/usr/bin/env bash
set -euo pipefail

# CI runner preparation only. An alternate APT directory permits fixture review
# without touching the local host. Invoke with sudo on the Ubuntu runner.
BROWSER_APT_ROOT="${1:-/etc/apt}"
if (($# > 1)); then
  echo "usage: local-web-product-e2e-apt.sh [apt-directory]" >&2
  exit 2
fi

# Hosted Ubuntu uses this optional priority mirror list. Keep its metadata and
# other repositories intact, replacing only the Azure HTTP archive that stalled.
BROWSER_APT_MIRRORS="$BROWSER_APT_ROOT/apt-mirrors.txt"
if [[ -f "$BROWSER_APT_MIRRORS" ]]; then
  BROWSER_APT_TEMP="$(mktemp)"
  trap 'rm -f "$BROWSER_APT_TEMP"' EXIT
  sed -E \
    's|^([[:blank:]]*)http://azure[.]archive[.]ubuntu[.]com/ubuntu(/?)([[:blank:]].*)?$|\1https://archive.ubuntu.com/ubuntu\2\3|' \
    "$BROWSER_APT_MIRRORS" > "$BROWSER_APT_TEMP"
  cat "$BROWSER_APT_TEMP" > "$BROWSER_APT_MIRRORS"
  echo "Browser dependency APT mirrors:"
  cat "$BROWSER_APT_MIRRORS"
fi

# APT reads these fragments in filename order. Use a late filename so the
# runner's default retry settings do not replace our finite network limits.
mkdir -p "$BROWSER_APT_ROOT/apt.conf.d"
cat > "$BROWSER_APT_ROOT/apt.conf.d/zz-seseragi-browser-network" <<'APT'
Acquire::Retries "1";
Acquire::http::Timeout "30";
Acquire::https::Timeout "30";
APT
