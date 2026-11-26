#!/usr/bin/env bash
# Scans base and head of a pull request and upserts one comment with the change impact.
# Draft: the exact CLI arguments are checked when `codeskimr change --markdown` has landed.
set -euo pipefail

marker='<!-- codeskimr-change -->'
work="${RUNNER_TEMP:?}/codeskimr"
mkdir -p "$work"
export CODESKIMR_DATA_DIR="$work/data"

asset="codeskimr-${CS_VERSION}-linux-x86_64"
gh release download "v${CS_VERSION}" --repo stacknoise/CodeSkimr --dir "$work" --pattern "${asset}*"
(cd "$work" && sha256sum --check "${asset}.sha256")
chmod +x "$work/$asset"
cs="$work/$asset"

git fetch --no-tags --depth=1 origin "$CS_BASE_SHA" "$CS_HEAD_SHA"
git checkout --quiet "$CS_BASE_SHA"
"$cs" scan .
git checkout --quiet "$CS_HEAD_SHA"
"$cs" scan .

body="$work/comment.md"
"$cs" change . --markdown --lang "$CS_LANG" > "$body"

if [ "${CS_FAIL_ON}" != none ]; then
  "$cs" change . --json > "$work/change.json"
fi

comment_id=$(gh api "repos/${CS_REPO}/issues/${CS_PR}/comments" --paginate \
  --jq "map(select(.body | startswith(\"${marker}\"))) | first | .id // empty")
if gh api "repos/${CS_REPO}/pulls/${CS_PR}" --jq '.head.repo.fork' | grep -q true; then
  cat "$body" >> "${GITHUB_STEP_SUMMARY:-/dev/null}"
elif [ -n "$comment_id" ]; then
  gh api -X PATCH "repos/${CS_REPO}/issues/comments/${comment_id}" -F "body=@${body}" > /dev/null
else
  gh api -X POST "repos/${CS_REPO}/issues/${CS_PR}/comments" -F "body=@${body}" > /dev/null
fi
