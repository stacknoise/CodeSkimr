#!/usr/bin/env bash
# Runs the benchmarks for the base commit and for the working tree on the same machine and
# compares both (blueprint 16.1, ADR 0014). Usage: tools/bench/ci.sh [base-commit]
# Without a base commit, or when the base has no harness yet, only the budgets are checked.
#
# A shared runner has noisy neighbours, so the measurement is made robust without loosening a
# limit (tolerance and budgets stay as they are):
# - base and change are measured one benchmark target after the other (base, change, base, change,
#   ...), not the whole base and then the whole change, so a slow phase of the machine hits both
#   sides of a comparison and not only one;
# - a failed comparison is measured once more, and the check then takes the best median of each
#   benchmark and side over both runs. A real slowdown shows in every run; an outlier does not.
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
BASE_REF="${1:-}"
BASE_BASELINE="ci-base"
HEAD_BASELINE="ci-head"
# The packages with benchmarks. The store benchmarks are their own package, because the unit tests
# of its crate cannot take the options of criterion.
PACKAGES=("codeskimr_bench:tools/bench/Cargo.toml" "codeskimr_storage:crates/adapters/storage/Cargo.toml")
RUN_HOME="$ROOT/target/criterion"
RETRY_HOME="$ROOT/target/criterion-retry"

# The names of the `[[bench]]` targets that a manifest lists, one per line (read from stdin).
bench_targets() {
  awk '/^\[\[bench\]\]/ {inside = 1; next} /^\[/ {inside = 0}
       inside && /^name *=/ {gsub(/[" ]/, ""); split($0, part, "="); print part[2]}'
}

# Runs one benchmark target of a package in a tree and saves it under a baseline name.
# Arguments: tree package target baseline. The base tree builds in its own target directory.
bench_target() {
  local tree="$1" package="$2" target="$3" baseline="$4"
  if [ "$tree" = "$ROOT" ]; then
    (cd "$tree" && cargo bench -p "$package" --bench "$target" --locked -- --save-baseline "$baseline")
  else
    (cd "$tree" && CARGO_TARGET_DIR="$tree/target" \
      cargo bench -p "$package" --bench "$target" --locked -- --save-baseline "$baseline")
  fi
}

# Measures every benchmark target: the base first (if there is one), then the change.
# Argument: the base tree, or an empty string. Returns non-zero if any run fails.
measure_targets() {
  local base_tree="$1" entry package manifest target
  for entry in "${PACKAGES[@]}"; do
    package="${entry%%:*}"
    manifest="${entry#*:}"
    for target in $(bench_targets < "$ROOT/$manifest"); do
      # A base without this target (a new benchmark) is not measured and reports it as new.
      if [ -n "$base_tree" ] && [ -f "$base_tree/$manifest" ] &&
        bench_targets < "$base_tree/$manifest" | grep -qx "$target"; then
        bench_target "$base_tree" "$package" "$target" "$BASE_BASELINE" || return 1
      fi
      bench_target "$ROOT" "$package" "$target" "$HEAD_BASELINE" || return 1
    done
  done
}

# One complete measurement into the criterion directory $CRITERION_HOME.
measure() {
  # Old results (for example from the build cache) must never be compared.
  rm -rf "$CRITERION_HOME"
  local base_tree=""
  if [ -n "$BASE_REF" ] && git cat-file -e "$BASE_REF:tools/bench/Cargo.toml" 2>/dev/null; then
    base_tree="$(mktemp -d)"
    git worktree add --detach "$base_tree" "$BASE_REF" || return 1
  else
    echo "NOTE: no base measurement available for '${BASE_REF}'."
  fi
  # set -e does not work inside a function that runs in an `if` condition, so every step checks itself.
  local status=0
  measure_targets "$base_tree" || status=1
  [ -z "$base_tree" ] || git worktree remove --force "$base_tree"
  [ "$status" -eq 0 ]
}

# Compares the results of the given criterion directories (more than one: best of the runs).
check() {
  local flags=() dir
  for dir in "$@"; do flags+=(--criterion-dir "$dir"); done
  local no_base=()
  [ -n "$BASE_REF" ] && git cat-file -e "$BASE_REF:tools/bench/Cargo.toml" 2>/dev/null || no_base=(--no-base)
  cargo run --locked -p codeskimr_bench --bin bench-check -- \
    "${flags[@]}" --base "$BASE_BASELINE" --new "$HEAD_BASELINE" --root "$ROOT" "${no_base[@]}"
}

export CRITERION_HOME="$RUN_HOME"
measure || exit 1
if ! check "$RUN_HOME"; then
  echo "Comparison failed, measuring once more and taking the best of both runs."
  rm -rf "$RETRY_HOME"
  CRITERION_HOME="$RETRY_HOME" measure || exit 1
  check "$RUN_HOME" "$RETRY_HOME"
fi
