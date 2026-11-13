# Benchmarks

Benchmarks implement blueprint 16.1 and 17.1. The harness is `tools/bench` (crate `codeskimr_bench`), the CI check is the required job `bench`.

## How the check works

1. The `bench` job measures the **base commit** (the target branch of the pull request) and then the **change** with `criterion`, one after the other on the same runner (`tools/bench/ci.sh`).
2. `bench-check` compares the medians per benchmark. A benchmark that is slower than `regression_tolerance_percent` (`config/default.toml`, 10 %) fails the job.
3. Benchmarks listed in `tools/bench/budgets.toml` are also checked against their absolute budget from `[budgets]` in `config/default.toml`.
4. A benchmark without a base measurement (new benchmark, or the base has no harness yet) is reported and does not fail.

No absolute numbers are stored in the repository: shared CI runners differ too much from machine to machine, so a stored baseline would raise false alarms. Comparing two runs on the same runner removes that difference.

## Reference machine (O-06)

Decided on 2026-11-13 (ADR 0013): the reference is the CI runner itself, compared against the base commit (A/B). `tools/bench/machine.toml` records this and the class of the developer laptop the budgets are meant for.
Absolute budgets on the shared runner are a coarse upper bound. Precise absolute numbers (for example for the acceptance of a milestone) are measured by hand on a mid-range laptop.
If runs turn out too noisy, the options are a longer measurement time or instruction counting (`iai-callgrind`); both need a new decision.

## Running locally

```sh
# run and compare against the current commit as base (no base harness: budgets only)
tools/bench/ci.sh HEAD

# only run the benchmarks
cargo bench -p codeskimr_bench
```

A failed comparison is measured once more before the job fails, because one run on a shared runner can be an outlier. Results of earlier runs (for example from the build cache) are deleted first, and an error while reading results always fails the check; only `--no-base` (the base commit has no harness yet) skips the comparison.

`ci.sh` needs Linux or macOS with bash. On Windows use Git Bash or WSL, or run `cargo bench -p codeskimr_bench` directly.

## Adding a benchmark

1. Add a function to `tools/bench/benches/domain.rs` or a new file in `benches/` (register it in `tools/bench/Cargo.toml` with `harness = false`; `ci.sh` runs every benchmark of the crate).
2. Name it `group/name` through `benchmark_group`; the id is the directory below `target/criterion`.
3. When it measures a budgeted interaction of blueprint 16.1, add `"group/name" = "<budget key>"` to `tools/bench/budgets.toml`.

## Output of the tools

Format strings and error messages of `tools/` are exempt from the no-literals rule (ADR 0013); values still live in `consts.rs`.

## Energy proxy

Blueprint 16.2 asks for CPU time per scan next to wall time. criterion measures wall time only; the CPU-time figure comes with the scan benchmarks (WP-23 onwards) and extends the report then.
