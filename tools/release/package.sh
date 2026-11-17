#!/usr/bin/env bash
# Prepares the release asset of CodeSkimr: the program as one file per system (the UI, the language packs
# and the default configuration are inside it) plus its SHA-256 file (docs/release.md).
# Usage: tools/release/package.sh <platform-name> <version>   (run from the repository root, after
# `npm run build` in web/ and `cargo build --release --locked -p codeskimr`).
set -euo pipefail

platform="$1"
version="$2"

case "${platform}" in
  windows*) binary="codeskimr.exe"; suffix=".exe" ;;
  *) binary="codeskimr"; suffix="" ;;
esac
asset="codeskimr-${version}-${platform}${suffix}"

rm -rf dist
mkdir -p dist
cp "target/release/${binary}" "dist/${asset}"
chmod +x "dist/${asset}"

# macOS has no sha256sum.
if command -v sha256sum >/dev/null 2>&1; then
  (cd dist && sha256sum "${asset}" > "${asset}.sha256")
else
  (cd dist && shasum -a 256 "${asset}" > "${asset}.sha256")
fi
echo "dist/${asset}"
