#!/usr/bin/env bash
set -euo pipefail

repo="${TC_REPOSITORY:-mohamed-dev-labs/Thinking-Computer}"
install_dir="${TC_INSTALL_DIR:-$HOME/.local/bin}"
version="${TC_VERSION:-latest}"

os="$(uname -s)"
arch="$(uname -m)"
case "$os/$arch" in
  Linux/x86_64) target="x86_64-unknown-linux-gnu" ;;
  Darwin/x86_64) target="x86_64-apple-darwin" ;;
  Darwin/arm64) target="aarch64-apple-darwin" ;;
  *) echo "Unsupported platform: $os/$arch" >&2; exit 1 ;;
esac

asset="thinking-computer-${target}.tar.gz"
base_url="https://github.com/${repo}/releases"
if [[ "$version" == "latest" ]]; then url="${base_url}/latest/download/${asset}"; else url="${base_url}/download/${version}/${asset}"; fi

tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT
curl --fail --location --silent --show-error "$url" -o "$tmp/$asset"
tar -xzf "$tmp/$asset" -C "$tmp"
mkdir -p "$install_dir"
install -m 0755 "$tmp/thinking-computer" "$install_dir/thinking-computer"
echo "Installed thinking-computer to $install_dir/thinking-computer"
echo "Add $install_dir to PATH if it is not already available."

