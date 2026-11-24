# Benchmarks

Benchmarks implement blueprint 16.1 and 17.1. The harness is `tools/bench` (crate `codeskimr_bench`), the CI check is the required job `bench`.

## How the check works

1. The `bench` job measures the **base commit** (the target branch of the pull request) and the **change** with `criterion` on the same runner (`tools/bench/ci.sh`). The two sides alternate benchmark target by benchmark target, so a slow phase of the runner hits both sides alike.
2. `bench-check` compares the medians per benchmark. A benchmark that is slower than `regression_tolerance_percent` (`config/default.toml`, 10 %) fails the job.
3. Benchmarks listed in `tools/bench/budgets.toml` are also checked against their absolute budget from `[budgets]` in `config/default.toml`.
4. A benchmark without a base measurement (new benchmark, or the base has no harness yet) is reported and does not fail.

No absolute numbers are stored in the repository: shared CI runners differ too much from machine to machine, so a stored baseline would raise false alarms. Comparing two runs on the same runner removes that difference.

## Reference machine (O-06)

Decided on 2026-11-13 (ADR 0014): the reference is the CI runner itself, compared against the base commit (A/B). `tools/bench/machine.toml` records this and the class of the developer laptop the budgets are meant for.
Absolute budgets on the shared runner are a coarse upper bound. Precise absolute numbers (for example for the acceptance of a milestone) are measured by hand on a mid-range laptop.
If runs turn out too noisy, the options are a longer measurement time or instruction counting (`iai-callgrind`); both need a new decision.

## Running locally

```sh
# run and compare against the current commit as base (no base harness: budgets only)
tools/bench/ci.sh HEAD

# only run the benchmarks
cargo bench -p codeskimr_bench
```

A failed comparison is measured again before the job fails, because one run on a shared runner can be an outlier. If the second comparison fails too, a third run follows (base first again), because noise on a shared runner is not always independent between two runs (issue #456); the job fails only if the best medians of all three runs still break a limit. The second run is a full run of both sides with the order reversed (change first, then base), so an effect of the order does not stay in both runs; `bench-check` then takes the best (lowest) median of each benchmark and side over all runs so far (`--criterion-dir` may be given more than once). Noise only ever makes a run slower, so the best median is the closest to the true value; the tolerance and the budgets stay as they are, and a real regression is slower in every run and still fails. Results of earlier runs (for example from the build cache) are deleted first, and an error while reading results always fails the check; only `--no-base` (the base commit has no harness yet) skips the comparison.

`ci.sh` needs Linux or macOS with bash. On Windows use Git Bash or WSL, or run `cargo bench -p codeskimr_bench` directly.

## Adding a benchmark

1. Add a function to `tools/bench/benches/domain.rs` or a new file in `benches/` (register it in `tools/bench/Cargo.toml` with `harness = false`; `ci.sh` runs every benchmark of the crate).
2. Name it `group/name` through `benchmark_group`; the id is the directory below `target/criterion`.
3. When it measures a budgeted interaction of blueprint 16.1, add `"group/name" = "<budget key>"` to `tools/bench/budgets.toml`.

## Scan benchmark

`scan/full_100k_loc` (`benches/scan.rs`) builds the release binary, generates a corpus of 400 Rust files (about 100k lines, no duplicate names) and takes the scan time that `codeskimr scan --json` reports for it with a fresh database (starting the process has the cold start budget; the reported time leaves out the loading of the language packs, ADR 0059, which `pack_load/rust_and_csharp` measures), against `full_scan_100k_loc_ms`. The corpus is generated so that it is the same everywhere and does not multiply references through equal names. Find the slow stage of a scan with `CODESKIMR_LOG=info`: every pipeline stage logs its duration (`stage` and `elapsed_ms`).

`rescan/one_file_100k_loc` (`benches/rescan.rs`) starts `codeskimr scan --watch` on the same corpus, appends a line to one file and takes the time that watch mode reports for the rescan ("scanned again in N ms"), against `incremental_rescan_ms`. The first rescans fill the retention of snapshots, so the measured ones also prune the oldest snapshot. A rescan writes only what changed (ADR 0032); what is left is discovery, the rebuild of the read models and the prune, each visible with `CODESKIMR_LOG=info`.

`pack_load/rust_and_csharp` (`benches/pack_load.rs`, issue #426) scans a folder with one Rust and one C# file with a fresh database and takes `pack_load_seconds` from `codeskimr scan --json`: the time the process spent loading language packs, which `seconds` leaves out (ADR 0059; with several packs loading at the same time it counts once). C# has the longest pack. It is checked against `pack_load_ms` (1,500 ms, ADR 0063; measured 0.8 to 0.9 s on a 4 core container on 2026-11-15, `main` at 2312740), because the cold start budget (blueprint 16.1) ends before the first scan and the pack load is no part of "Scanned in"; a regression against the base commit fails like for every benchmark. The budget covers these two packs in isolation (C# is the longest pack). A project with many languages loads more packs while it parses, and that takes longer (CodeSkimr itself: eight packs, 2.1 s of load time in a scan of 1.5 s, issue #465); that case is reported, not budgeted.

`git_scan/first_100k_loc` and `git_scan/update_100k_loc` (`benches/git_scan.rs`) scan the same corpus as a Git source (issue #36). The benchmark commits the corpus to a local repository, serves it with `git daemon` on the loopback interface (`file://` addresses are accepted as Git sources as well, issue #388; the benchmark uses the daemon so that clone and update go through the network transport, and it needs the `git` command line tool) and runs `codeskimr scan git://127.0.0.1:<port>/... --branch main`. `first` scans with an empty cache and database (clone and scan). `update` first commits a line to one file and then scans again with the cache and database of the earlier scan (update of the clone and scan). Both have no absolute budget; the A/B comparison against the base commit tells whether a change to the Git source made them slower or faster. The fetch stage shows in `CODESKIMR_LOG=info` (`stage="fetch"`). The benchmark is meant for Linux (the CI runner): stopping the daemon on Windows would stop only the `git` wrapper process. The scan of the CLI is always a full scan, so the update does not benefit from unchanged files, only from what the fetch itself saves. Since the update (#36) takes over the Git data of the old copy and fetches only the new commits, the saving shows most with a slow connection; against the local daemon it is small (about 1.0 s to 0.9 s on the development machine), because the data never crosses a network.

## Output of the tools

Format strings and error messages of `tools/` are exempt from the no-literals rule (ADR 0014); values still live in `consts.rs`.

## Quick search and duplicate check

`search/typed_1` (`crates/adapters/storage/benches/search.rs`) runs the query behind `search.query` on a snapshot with 20,000 symbols, for the first character of a query: it matches the most symbols and is the slowest keystroke. It is checked against `search_keystroke_ms`. The store benchmarks share their snapshot builder (`benches/common`).

`type_switch/explorer_symbol` (`crates/adapters/storage/benches/type_switch.rs`) reads one type as `explorer.symbol` does, on a snapshot with 20,000 symbols in types of 20: the symbol card (`CodeIndex::symbol_card`: symbol, file, language and metrics, without the member list and the same-name count) and the rows of the member table. It is checked against `explorer_switch_ms`. Before issue #492 the command used the full lookup, whose same-name count scans every symbol of the snapshot (no index on the qualified name), once for the type and once for each enclosing symbol of the breadcrumb: 4.5 ms against 0.18 ms for the store part on this snapshot, and the cost grows with the size of the project.

`alerts/step_100_critical` (`crates/adapters/storage/benches/alerts.rs`, ADR 0068) runs the alert step on a snapshot with 20,000 symbols and their content hashes and 100 critical annotations that did not change: what every save of a watched project pays after the snapshot is written. It is checked against `incremental_rescan_ms` (300 ms per saved file, and the 10 % of a scan the ADR asks for stay far below that); it takes about 1 ms. The targets are found through the index `symbols_by_name`; before it, the same step took about 80 ms because every lookup passed over all symbols. `alerts/step_50_vanished` adds 50 more critical annotations whose symbols the newest snapshot lacks, with 50,000 names written in its files: for each of them the step looks for symbols of the same simple name and counts the names that still point at it, on every scan while the alert is open. Without indexes on `symbol_rows.name` and `pending_references.target` that took about 450 ms (over the budget); with them about 2 ms. The indexes are made by the step itself the first time a project has a critical annotation (`ENSURE_STATE_INDEXES`), not by the migration: made with the schema they made `scan/full_100k_loc` and `git_scan/update_100k_loc` about 10 to 15 % slower on the CI runner (the persist stage of the 100k lines corpus took about 330 ms against 280 ms without them, and the token hashes added about 4 % to the analyze stage, within the noise; the two sides were measured alternately on one machine). Without the indexes the persist stage is as fast as before, and the hashes are collected in one buffer per symbol (one call to BLAKE3 per symbol, not per token) and the kinds of nodes are looked up once per kind. Making the three indexes on a snapshot with 20,000 symbols and 50,000 names is a one-off of some 30 ms, paid by the first scan after the first critical annotation. `rescan/one_file_100k_loc` runs the step too (without annotations it is two cheap statements).

`similar/pairs_boilerplate_5k` (`benches/similar.rs`) runs `similar_pairs` over 5,000 bodies that all start with the same boilerplate, so some hashes have 5,000 owners. Without `conventions.duplicate_max_owners` this takes minutes (it grows with the square of the owners); with the default of 50 it takes about 20 ms. It has no absolute budget, a regression against the base fails like for every benchmark.

## Energy proxy

Blueprint 16.2 asks for CPU time per scan next to wall time. criterion measures wall time only, so `scan --json` reports `cpu_seconds` (the CPU time of all threads of the scan, from the operating system) next to `seconds`. `scan/full_100k_loc` writes the median of its CPU times to `cpu.json` beside its criterion results, and `bench-check` prints it under "CPU time per benchmark". It is a report; there is no budget for it. `scan --json` also carries `stages`, how long fetch, discover, analyze, resolve, measure and persist took in seconds (the console prints them as one line, "Steps: …"), so that a slow scan can be traced to its stage. The parse time of a language in the scan screen is the sum over all analysis threads, not the time of the clock; `cpu_seconds` next to `seconds` tells how parallel a scan was. The rescan benchmark has no CPU figure, because watch mode does not report one.

The number of analysis threads is `scan.max_workers` (0 means one per core). Language packs are loaded on at most one thread per core. Reducing the threads on battery power or with a hidden window needs the power state of the device, which std does not offer; it stays open in issue #141.
