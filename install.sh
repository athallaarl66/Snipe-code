#!/usr/bin/env bash
# Installs a published snipe-code release binary.
#
# Downloads the archive to a temporary path, verifies the published checksum and
# the cosign bundle when cosign is present, then moves the binary into place.
# Nothing is piped into a shell. A failed check exits non-zero without leaving a
# partial binary behind.
#
#   curl -fsSL https://raw.githubusercontent.com/athallaarl66/Snipe-code/main/install.sh | less
#   bash install.sh
#
# Overrides: SNIPE_INSTALL_VERSION (default latest), SNIPE_INSTALL_DIR
# (default ~/.local/bin), SNIPE_RELEASE_BASE_URL.
set -euo pipefail

REPO="athallaarl66/Snipe-code"
VERSION="${SNIPE_INSTALL_VERSION:-latest}"
INSTALL_DIR="${SNIPE_INSTALL_DIR:-$HOME/.local/bin}"
BASE_URL="${SNIPE_RELEASE_BASE_URL:-https://github.com/${REPO}/releases/download}"

fail() {
  printf 'install.sh: %s\n' "$1" >&2
  exit 1
}

command -v curl > /dev/null || fail "curl is required"
command -v tar > /dev/null || fail "tar is required"

system=$(uname -s)
machine=$(uname -m)

case "$system" in
  Linux) os=linux ;;
  Darwin) os=darwin ;;
  *) fail "unsupported operating system: $system (use install.ps1 on Windows)" ;;
esac

case "$os/$machine" in
  linux/x86_64 | linux/amd64) target=x86_64-unknown-linux-gnu ;;
  darwin/x86_64) target=x86_64-apple-darwin ;;
  darwin/arm64 | darwin/aarch64) target=aarch64-apple-darwin ;;
  *) fail "unsupported architecture: $system/$machine" ;;
esac

if [ "$VERSION" = "latest" ]; then
  VERSION=$(curl -fsSL -A 'snipe-code-install' \
    "https://api.github.com/repos/${REPO}/releases/latest" |
    grep -m1 '"tag_name"' | cut -d'"' -f4)
  [ -n "$VERSION" ] || fail "could not resolve the latest release"
fi

asset="snipe-code-${VERSION}-${target}.tar.gz"
url="${BASE_URL}/${VERSION}/${asset}"
workspace=$(mktemp -d)
trap 'rm -rf "$workspace"' EXIT

curl -fsSL -A 'snipe-code-install' -o "$workspace/$asset" "$url" ||
  fail "could not download $url"
curl -fsSL -A 'snipe-code-install' -o "$workspace/$asset.sha256" "$url.sha256" ||
  fail "could not download the checksum for $asset"

if command -v sha256sum > /dev/null; then
  actual=$(sha256sum "$workspace/$asset" | cut -d' ' -f1)
else
  actual=$(shasum -a 256 "$workspace/$asset" | cut -d' ' -f1)
fi
expected=$(cut -d' ' -f1 < "$workspace/$asset.sha256")
[ -n "$expected" ] || fail "the published checksum file is empty"
[ "$expected" = "$actual" ] ||
  fail "checksum mismatch for $asset (published $expected, downloaded $actual)"

if command -v cosign > /dev/null; then
  if curl -fsSL -A 'snipe-code-install' -o "$workspace/$asset.bundle" "$url.bundle"; then
    cosign verify-blob --bundle "$workspace/$asset.bundle" \
      --certificate-identity-regexp 'https://github.com/.*/.github/workflows/.*' \
      --certificate-oidc-issuer 'https://token.actions.githubusercontent.com' \
      "$workspace/$asset" > /dev/null ||
      fail "the cosign bundle for $asset did not verify"
  else
    fail "cosign is installed but the signature bundle for $asset could not be downloaded"
  fi
fi

mkdir -p "$workspace/extract"
tar -xzf "$workspace/$asset" -C "$workspace/extract"
source_binary=$(find "$workspace/extract" -type f -name snipe-code | head -1)
[ -n "$source_binary" ] || fail "the archive does not contain the snipe-code binary"

mkdir -p "$INSTALL_DIR"
install -m 0755 "$source_binary" "$INSTALL_DIR/snipe-code" ||
  fail "could not install to $INSTALL_DIR"

printf 'Installed snipe-code %s to %s/snipe-code\n' "$VERSION" "$INSTALL_DIR"
case ":$PATH:" in
  *":$INSTALL_DIR:"*) ;;
  *) printf 'Add it to PATH:  export PATH="%s:$PATH"\n' "$INSTALL_DIR" ;;
esac
