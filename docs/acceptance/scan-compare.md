# Acceptance: scan history, comparison and view "Change" (ADR 0097)

State of 2026-11-27, `main` at `d97ba5b` (packages 1 to 5 of ADR 0097 merged: #748, #764, #773, #796, #798). Verdict: **met** for the criteria below; open points are listed at the end.

## What was run

A real project, a copy of the folders `crates/application` and `crates/domain` of this repository (201 Rust files, 5,195 symbols, 29,838 references; level 2 not enabled). Built binary of `main`, own data folder, three scans in a row, changed by hand between them:

1. Scan 1: unchanged state.
2. Scan 2: the parameter list of a public method of the domain (`Comparison::touched_in_base`) was changed; one of its two callers was adapted, the other (a test) was left alone.
3. Scan 3: a public method (`SymbolFact::identity`) was renamed.

## Results

| Criterion (ADR 0097) | Observed |
|---|---|
| `scans` lists the kept scans with time, kind, duration and changed files | Three rows, newest first; `201 files, 2 changed, 1842 ms` for scan 2; the first scan shows `-` for the changed files. |
| `compare` finds added, removed, changed and moved symbols | Scan 1 to 2: 1 added, 1 removed (the method, by its new parameter types), 1 changed (the adapted caller, reason `content`), 0 moved. |
| Identity key is kind, qualified name and parameter types; no rename detection | A changed parameter list is removed plus added; the rename in scan 3 is removed plus added, 0 changed. As decided in the ADR addendum. |
| Impact lists direct usages in the older scan as adapted or not touched | `change`: 1 sure and 1 probable usage; the adapted caller is marked `(adapted)`, the test that was not touched `(not touched)`. |
| `change --markdown` for the pull request comment | First line `<!-- codeskimr-change -->`, key figures, checklist `- [ ] file:line …` with the one place not touched, the note on direct usages only. |
| `--json` carries `schema` 1, `summary`, `is_cut` | Checked: `schema` 1, `changed_symbols` 2, `public_symbols` 1, `usages_not_touched` 1, `is_cut` false. |
| Wrong order is refused with a code | `compare --base <newer> --target <older>` ends with `CMP-101` and exit code 1. |
| Registry, REST, MCP and screen | Covered by the tests of #796 (registry commands, MCP tool `impact_of_diff`) and #798 (screen with the e2e `ui-change` in three themes). Not clicked through by hand in a browser in this run. |

## Backed by tests only

The real run saw no moved symbol, no change of visibility, no critical symbol, no new layer violation or cycle, and neither `CMP-201` nor `CMP-301`. These criteria are backed by the tests of packages 3 to 5 (`crates/domain/tests/compare.rs`, `crates/application/tests/`, the registry, CLI and MCP tests of #796 and the screen tests of #798), not by this run.

## Open points

- Resolved 2026-11-27 (decision of Anton): every scan is a row of the history, also a scan without changes, which shows "0 changed" and compares as "nothing changed" (ADR 0093 addendum, schema version 26). Earlier runs of this acceptance saw one row for three scans in a row.

- The button "Copy as Markdown" of the mock is missing in the screen until #799 is done (ADR 0092 lists `--markdown` as an exception meanwhile).
- The comparison does not see a renamed symbol as the same symbol. A pure rename therefore shows as one removed and one added symbol, and the usages of the old name appear in the impact of the removed one. This is a stated limit (ADR 0097), not a defect.
- Not measured here: comparison time on a very large project (the budget `[compare] max_symbols = 200`, `max_usages = 50` cuts the answer; the answer says so with `is_cut` and `total`).
