#!/usr/bin/env bash
# Clean-install probe of a release package (docs/release.md): unpack the archive into an empty
# folder, check the checksum, then run the program only from there.
# Usage: tools/release/verify.sh <archive> <version>   (the .sha256 file lies next to the archive)
set -euo pipefail

archive="$1"
version="$2"
work="$(mktemp -d)"
trap 'rm -rf "${work}"' EXIT

(cd "$(dirname "${archive}")" && case "$(uname -s)" in
  Darwin) shasum -a 256 -c "$(basename "${archive}").sha256" ;;
  *) sha256sum -c "$(basename "${archive}").sha256" ;;
esac)

case "${archive}" in
  *.zip) 7z x -bso0 -o"${work}/unpacked" "${archive}" ;;
  *) mkdir "${work}/unpacked" && tar -xzf "${archive}" -C "${work}/unpacked" ;;
esac

dir="$(find "${work}/unpacked" -mindepth 1 -maxdepth 1 -type d | head -n 1)"
bin="${dir}/codeskimr"
[ -x "${bin}" ] || bin="${dir}/codeskimr.exe"

for file in default.toml LICENSE README.md LIES-MICH.md README-PREVIEW.md docs/manual; do
  [ -e "${dir}/${file}" ] || { echo "Missing in package: ${file}" >&2; exit 1; }
done

"${bin}" --version | grep -F "${version}"

# A real scan from the unpacked folder only: own data folder, a tiny project. The language packs are
# inside the program, so the package must not contain a languages folder.
[ ! -e "${dir}/languages" ] || { echo "The package must not ship languages/" >&2; exit 1; }
project="${work}/project"
mkdir "${project}"
printf 'fn main() {}\n' > "${project}/main.rs"
(cd "${work}" && "${bin}" --data-dir "${work}/data" scan "${project}" --json) | grep -F '"files":1'
echo "Clean install probe passed: ${archive}"
