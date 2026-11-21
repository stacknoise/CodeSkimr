# 0084 – Bound the candidates of a bare name

Status: accepted (decision of Anton of 2026-11-21: "Versuchen wir es mal mit 100. Runter gehen können wir immer noch")

## Context

ADR 0033 (decision 6) says level 1 never drops a candidate: a name that matches several symbols is stored for all of them as `probable`. That keeps recall high, but the number of edges is the product of uses and candidates. Measured on `microsoft/vscode` (2026-11-21, release build, 4 cores): the folder `src/vs/platform` (3,759 files, 88,732 symbols) gives 13.7 million references and a 1.75 GB database; scanning the whole `src` folder is killed after 13 GB of memory. 84 % of those edges come from places with more than 25 candidates, 58 % from places with 101 to 500 candidates: `x.id`, `x.type`, `.dispose()`, `.get()` on receivers whose type level 1 does not know. Such an edge says nothing about one of its targets (one in several hundred is right), but it fills the memory, the database, the usage counts and every list the UI shows.

## Options

1. Keep everything (status quo). The scan of a large TypeScript project does not finish or takes hours, and the UI cannot be used.
2. Bound the candidates of a place that has nothing but the bare name (step 5 of blueprint 9.2, or a receiver of unknown type). Above the bound the place is given up, counted per scan and reported as `RES-304`.
3. Store such places once and expand them when queried. Keeps recall, but changes the schema (ADR 0032 views), every usage query and the metrics; too large for 1.0.

## Decision

Option 2 with the limit `scan.max_candidates`, default **100**, configurable (user configuration and `--set scan.max_candidates=N`). Anton chose 100 over 25 for more recall and can lower it later.

- Only places that reach their candidates through the bare name are bounded. A place that a scope, an enclosing type, a receiver type or an import narrows down keeps all its candidates, however many.
- A place with at most `max_candidates` candidates behaves exactly as before.
- Level 2 (SCIP) is unaffected: its references are exact and are not bounded.
- The scan reports one warning `RES-304` with the number of places given up. An incremental scan counts only the places it resolved again.
- `scan.max_candidates` is a protection of the machine like `scan.max_source_mb`: the project file `.codeskimr.toml` may lower it but not raise it (ADR 0040, `LOWER_ONLY_SCAN_KEYS`).

## Incremental scans

The number of candidates of a name is a property of the whole project. An incremental scan resolves again every place whose name looks up a name that the changed files add or remove (`Carried::due_names`), so it also gives a place up, or takes it back, when the count of a name crosses the limit because of a changed file. Two tests in `crates/app/src/incremental_tests.rs` compare the incremental result with a full scan for both directions (a third candidate appears; it disappears again). A place whose name is looked up by unchanged files only keeps its result, as before.

## Consequences

- `BLUEPRINT.md` 9.2 and the code table get the bound and `RES-304`; ADR 0033 decision 6 now has this one exception.
- Recall of level 1 drops for names with more than 100 same-named symbols and no other hint; the hand-made samples of the acceptance report are to be repeated after the change. The limit can be lowered without a schema change.
- The recall check is tracked in issue #642 (sample of the acceptance report to repeat).
- Follow-up: show "places given up" in the project summary of the UI (the warnings are stored per snapshot already).
