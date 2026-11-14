#!/usr/bin/env bash
# Packs the release build of CodeSkimr into one archive (preview releases, docs/release.md).
# Usage: tools/release/package.sh <platform-name> <version>   (run from the repository root, after
# `npm run build` in web/ and `cargo build --release --locked -p codeskimr`).
set -euo pipefail

platform="$1"
version="$2"
name="codeskimr-${version}-${platform}"
stage="dist/${name}"

rm -rf dist
mkdir -p "${stage}/docs"

case "${platform}" in
  windows*) binary="codeskimr.exe" ;;
  *) binary="codeskimr" ;;
esac

cp "target/release/${binary}" "${stage}/"
cp -R languages "${stage}/languages"
cp config/default.toml "${stage}/default.toml"
cp -R docs/manual "${stage}/docs/manual"
cp LICENSE README.md "${stage}/"
cp docs/release/preview-de.md "${stage}/LIES-MICH.md"
cp docs/release/preview-en.md "${stage}/README-PREVIEW.md"

case "${platform}" in
  windows*)
    (cd dist && tar -a -cf "${name}.zip" "${name}")
    archive="${name}.zip"
    ;;
  *)
    tar -C dist -czf "dist/${name}.tar.gz" "${name}"
    archive="${name}.tar.gz"
    ;;
esac

(cd dist && sha256sum "${archive}" > "${archive}.sha256")
echo "dist/${archive}"
