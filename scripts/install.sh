#!/bin/sh
# Install or update the Seseragi CLI and LSP.
#
#   curl -fsSL https://github.com/KentaroMorishita/seseragi/releases/latest/download/install.sh | sh
#
# Environment variables:
#   SESERAGI_VERSION       version to install, such as 0.61.165 (default: latest)
#   SESERAGI_INSTALL       install root; binaries go to $SESERAGI_INSTALL/bin
#                          (default: $HOME/.seseragi)
#   SESERAGI_DOWNLOAD_URL  base URL that holds the release archives
#                          (default: the GitHub release for the version)

set -eu

repository="https://github.com/KentaroMorishita/seseragi"

fail() {
  printf 'seseragi install: %s\n' "$1" >&2
  exit 1
}

need() {
  command -v "$1" >/dev/null 2>&1 || fail "\`$1\` is required"
}

need curl
need tar
need uname
need mktemp

case "$(uname -s)" in
  Darwin) os=darwin ;;
  Linux) os=linux ;;
  *) fail "unsupported OS: $(uname -s). On Windows, use install.ps1." ;;
esac

case "$(uname -m)" in
  arm64 | aarch64) cpu=arm64 ;;
  x86_64 | amd64) cpu=x64 ;;
  *) fail "unsupported CPU: $(uname -m)" ;;
esac

target="$os-$cpu"
case "$target" in
  darwin-arm64 | darwin-x64 | linux-x64) ;;
  *) fail "no prebuilt CLI for $target yet. Build from source: cargo install --git $repository --locked seseragi-cli" ;;
esac

version="${SESERAGI_VERSION:-}"
if [ -z "$version" ]; then
  latest_url="$(curl -fsSLI -o /dev/null -w '%{url_effective}' "$repository/releases/latest")" ||
    fail "could not find the latest release"
  version="${latest_url##*/}"
fi
version="${version#v}"
case "$version" in
  '' | *[!0-9.]*) fail "invalid version: $version" ;;
esac

archive="seseragi-v$version-$target.tar.gz"
base_url="${SESERAGI_DOWNLOAD_URL:-$repository/releases/download/v$version}"
install_root="${SESERAGI_INSTALL:-$HOME/.seseragi}"
bin_dir="$install_root/bin"

work="$(mktemp -d)"
trap 'rm -rf "$work"' EXIT INT TERM

printf 'Downloading Seseragi %s for %s\n' "$version" "$target"
curl -fsSL -o "$work/$archive" "$base_url/$archive" ||
  fail "could not download $base_url/$archive"
curl -fsSL -o "$work/$archive.sha256" "$base_url/$archive.sha256" ||
  fail "could not download the checksum $base_url/$archive.sha256"

expected="$(awk '{ print $1; exit }' "$work/$archive.sha256")"
if command -v sha256sum >/dev/null 2>&1; then
  actual="$(sha256sum "$work/$archive" | awk '{ print $1 }')"
elif command -v shasum >/dev/null 2>&1; then
  actual="$(shasum -a 256 "$work/$archive" | awk '{ print $1 }')"
else
  fail "neither sha256sum nor shasum is available, so the download cannot be checked"
fi
case "$expected" in
  '' | *[!0-9a-f]*) fail "the checksum file is malformed" ;;
esac
[ "$expected" = "$actual" ] ||
  fail "checksum mismatch for $archive (expected $expected, got $actual)"

mkdir -p "$work/extract"
tar -xzf "$work/$archive" -C "$work/extract"
for binary in seseragi seseragi-lsp; do
  [ -f "$work/extract/$binary" ] || fail "$archive does not contain $binary"
done

mkdir -p "$bin_dir"
for binary in seseragi seseragi-lsp; do
  # Copy next to the destination, then rename, so a running binary is
  # replaced without being truncated.
  cp "$work/extract/$binary" "$bin_dir/.$binary.new"
  chmod 755 "$bin_dir/.$binary.new"
  mv -f "$bin_dir/.$binary.new" "$bin_dir/$binary"
done

# The archive's license notice covers Unicode data inside the binaries.
if [ -f "$work/extract/UNICODE-LICENSE" ]; then
  cp "$work/extract/UNICODE-LICENSE" "$install_root/UNICODE-LICENSE"
fi

installed="$("$bin_dir/seseragi" --version)" || fail "the installed CLI does not run"
printf 'Installed %s to %s\n' "$installed" "$bin_dir"

case ":${PATH:-}:" in
  *":$bin_dir:"*) ;;
  *)
    printf '\nAdd the CLI to your PATH, for example in ~/.profile or ~/.zshrc:\n'
    printf '  export PATH="%s:$PATH"\n' "$bin_dir"
    ;;
esac

if ! command -v bun >/dev/null 2>&1; then
  printf '\nRunning programs needs Bun 1.4.4 or later: https://bun.sh/docs/installation\n'
fi
