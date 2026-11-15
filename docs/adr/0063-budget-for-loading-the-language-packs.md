# 0063 – A budget for loading the language packs of a first scan

Status: proposed

## Context

ADR 0059 loads a language pack when a file of its language is first analyzed, and leaves the load time out of the reported scan time ("Scanned in", option (a)). The start-up of the program no longer pays for packs, the first scan does. No budget of blueprint 16.1 covers that time: the full scan budget uses the reported scan time, the cold start budget ends before the first scan, and the benchmark `pack_load/rust_and_csharp` had no absolute limit (issue #465).

Measured on 2026-11-15 (release build, 4 cores, no other load), `pack_load_seconds` of `codeskimr scan --json` for a folder with one small file:

| Language | Load time |
|---|---|
| C# | 0.96 s |
| TSX | 0.39 s |
| Kotlin | 0.36 s |
| TypeScript | 0.35 s |
| Rust | 0.17 s |
| Java | 0.14 s |
| JavaScript, Python | 0.11 s each |
| All eight in one folder | 0.96 to 1.07 s (parallel, C# is the long pole) |
| Rust and C# (the benchmark), five runs | 0.78 to 0.92 s |

In a real scan of CodeSkimr itself (eight languages, 749 files) the load time is 2.1 s next to a scan time of 1.5 s, because loading competes with parsing for the cores. For a project in one language the load is 0.1 to 1 s.

## Decision

1. Blueprint 16.1 gets the row "Loading the language packs of a first scan (Rust and C#, the longest) < 1.5 s", and `config/default.toml` the key `budgets.pack_load_ms = 1500`.
2. The benchmark `pack_load/rust_and_csharp` is checked against it (`tools/bench/budgets.toml`), on the same runner as the other budgets.
3. The value is 1.5 s, not lower, because the benchmark runs on a shared CI runner with fewer cores than the container of the measurement and the budgets there are a coarse upper bound (`tools/bench/machine.toml`). It is above today's 0.8 to 0.9 s by a factor of about 1.7 so that noise does not fail the check, and low enough that a pack that doubles its load time does. A later change that makes the packs faster (for example splitting `references.scm`, ADR 0059 "not chosen") may lower it through a new ADR.
4. The budget covers the two longest packs in isolation. The load time of a project with many languages (2.1 s for CodeSkimr itself) is reported in `docs/benchmarks.md` and not budgeted, because it depends on the number of languages and the cores left over by the parser.
5. No other budget changes. The "Scanned in" time still leaves the load out.

## Options not chosen

- **Count the load into the full scan budget.** This would make the existing budget stricter by an amount that depends on the language mix, and the 100k line benchmark (Rust only) would hardly notice it.
- **Add the load to the cold start budget.** The load happens at the first scan, not at start-up.
- **A budget for all eight packs.** 2.1 s was measured with parsing running at the same time; a benchmark for it would measure the scheduler as much as the packs.
- **A lower value (1.0 s).** Would fail on a slower runner without a change in the code.

## Consequences

- The `bench` job fails when loading Rust and C# takes more than 1.5 s on the runner.
- The key is a new setting (`budgets.pack_load_ms`); user configuration can override it like the other budgets.
- The status screen does not show it yet; the mock has no row for it.
