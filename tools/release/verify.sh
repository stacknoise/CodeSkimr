#!/usr/bin/env bash
# Clean-install probe of a release asset (docs/release.md): check the checksum, then run the program
# as the only file in an empty folder, like a user who downloaded it.
# Linux needs a Secret Service for the start (SEC-601 without): the workflow runs this script inside one.
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
# The start as a user does it: no arguments, so the server starts and the browser is asked to open. A
# runner without a browser is fine (the server keeps running then); a program that ends is not. The
# probe waits until the start page answers on its own port, then ends the program again.
port=7499
"${bin}" --data-dir "${work}/start-data" --set "server.port=${port}" > "${work}/start.log" 2>&1 &
program=$!
trap 'kill "${program}" 2>/dev/null || true; rm -rf "${work}"' EXIT
answered=""
for _ in $(seq 1 60); do
  if ! kill -0 "${program}" 2>/dev/null; then
    echo "The program ended on its own when started without arguments:" >&2
    cat "${work}/start.log" >&2
    exit 1
  fi
  if page="$(curl -fsS "http://127.0.0.1:${port}/" 2>/dev/null)"; then
    answered="yes"
    break
  fi
  sleep 1
done
if [ -z "${answered}" ]; then
  echo "The server did not answer within 60 seconds:" >&2
  cat "${work}/start.log" >&2
  exit 1
fi
printf '%s' "${page}" | grep -qi '<html' || { echo "The start page is not the UI" >&2; exit 1; }
grep -F "Keep this window open" "${work}/start.log" >/dev/null || grep -F "Dieses Fenster offen lassen" "${work}/start.log" >/dev/null || { echo "The start message lacks the hint to keep the window open" >&2; cat "${work}/start.log" >&2; exit 1; }
kill "${program}" 2>/dev/null || true
wait "${program}" 2>/dev/null || true

[ "$(ls -A "${work}/install" | wc -l | tr -d ' ')" = "1" ] || { echo "The program must stay alone in its folder" >&2; exit 1; }
echo "Clean install probe passed: ${asset}"
