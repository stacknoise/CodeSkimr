#!/usr/bin/env bash
# Clean-install probe of a release asset (docs/release.md): check the checksum, then run the program
# as the only file in an empty folder, like a user who downloaded it.
# Usage: tools/release/verify.sh <asset> <version>   (the .sha256 file lies next to the asset)
set -euo pipefail

asset="$1"
version="$2"
work="$(mktemp -d)"
trap 'rm -rf "${work}"' EXIT

(cd "$(dirname "${asset}")" && case "$(uname -s)" in
  Darwin) shasum -a 256 -c "$(basename "${asset}").sha256" ;;
  *) sha256sum -c "$(basename "${asset}").sha256" ;;
esac)

# The program alone in a folder: no languages folder, no configuration, no UI folder next to it.
mkdir "${work}/install"
case "${asset}" in
  *.exe) bin="${work}/install/codeskimr.exe" ;;
  *) bin="${work}/install/codeskimr" ;;
esac
cp "${asset}" "${bin}"
chmod +x "${bin}"

"${bin}" --version | grep -F "${version}"

# A real scan with its own data folder and a tiny project. The language packs are inside the program.
project="${work}/project"
mkdir "${project}"
printf 'fn main() {}\n' > "${project}/main.rs"
(cd "${work}" && "${bin}" --data-dir "${work}/data" scan "${project}" --json) | grep -F '"files":1'
[ "$(ls -A "${work}/install" | wc -l | tr -d ' ')" = "1" ] || { echo "The program must stay alone in its folder" >&2; exit 1; }
echo "Clean install probe passed: ${asset}"
