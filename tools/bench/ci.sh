#!/usr/bin/env bash
# Runs the benchmarks for the base commit and for the working tree on the same machine and
# compares both (blueprint 16.1). Usage: tools/bench/ci.sh [base-commit]
# Without a base commit, or when the base has no harness yet, only the budgets are checked.
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
BASE_REF="${1:-}"
BASE_BASELINE="ci-base"
HEAD_BASELINE="ci-head"
BENCH_ARGS=(-p codeskimr_bench --bench domain --locked)
export CRITERION_HOME="$ROOT/target/criterion"

if [ -n "$BASE_REF" ] && git cat-file -e "$BASE_REF:tools/bench/Cargo.toml" 2>/dev/null; then
  BASE_TREE="$(mktemp -d)"
  git worktree add --detach "$BASE_TREE" "$BASE_REF"
  (cd "$BASE_TREE" && CARGO_TARGET_DIR="$BASE_TREE/target" \
    cargo bench "${BENCH_ARGS[@]}" -- --save-baseline "$BASE_BASELINE")
  git worktree remove --force "$BASE_TREE"
else
  echo "NOTE: no base measurement available for '${BASE_REF}'."
fi

cargo bench "${BENCH_ARGS[@]}" -- --save-baseline "$HEAD_BASELINE"
cargo run --locked -p codeskimr_bench --bin bench-check -- \
  --criterion-dir "$CRITERION_HOME" --base "$BASE_BASELINE" --new "$HEAD_BASELINE" --root "$ROOT"
