#!/usr/bin/env bash
# Runs the benchmarks for the base commit and for the working tree on the same machine and
# compares both (blueprint 16.1, ADR 0013). Usage: tools/bench/ci.sh [base-commit]
# Without a base commit, or when the base has no harness yet, only the budgets are checked.
# A failed comparison is measured once more, because one run on a shared runner can be an outlier.
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
BASE_REF="${1:-}"
BASE_BASELINE="ci-base"
HEAD_BASELINE="ci-head"
BENCH_ARGS=(-p codeskimr_bench --benches --locked)
export CRITERION_HOME="$ROOT/target/criterion"

measure() {
  # Old results (for example from the build cache) must never be compared.
  rm -rf "$CRITERION_HOME"
  local no_base="--no-base"
  if [ -n "$BASE_REF" ] && git cat-file -e "$BASE_REF:tools/bench/Cargo.toml" 2>/dev/null; then
    local base_tree
    base_tree="$(mktemp -d)"
    git worktree add --detach "$base_tree" "$BASE_REF"
    (cd "$base_tree" && CARGO_TARGET_DIR="$base_tree/target" \
      cargo bench "${BENCH_ARGS[@]}" -- --save-baseline "$BASE_BASELINE")
    git worktree remove --force "$base_tree"
    no_base=""
  else
    echo "NOTE: no base measurement available for '${BASE_REF}'."
  fi
  cargo bench "${BENCH_ARGS[@]}" -- --save-baseline "$HEAD_BASELINE"
  # shellcheck disable=SC2086
  cargo run --locked -p codeskimr_bench --bin bench-check -- \
    --criterion-dir "$CRITERION_HOME" --base "$BASE_BASELINE" --new "$HEAD_BASELINE" \
    --root "$ROOT" $no_base
}

if ! measure; then
  echo "Comparison failed, measuring once more."
  measure
fi
