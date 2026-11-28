# Acceptance: metric changes in the scan comparison (ADR 0114)

State of 2026-11-27, `main` with packages 1 to 3 of ADR 0114 merged (#825, #833, #837); the Markdown export (#838) was in review. Verdict: **met** for the criteria below.

## What was run

A real project, a copy of the folder `crates/domain` of this repository (65 Rust files; level 2 not enabled), built binary of `main`, own data folder. Two scans in a row, with two changes by hand in between:

1. In `SymbolFact::identity` eight branches were added (complexity up).
2. In `judge_metric` a block of three lines was taken out (complexity down).

## Results

| Criterion (ADR 0114) | Observed |
|---|---|
| `compare --metrics` lists what got worse and what got better, the worst first | Level member: 1 worse (`SymbolFact::identity`, complexity 1 -> 13, marked "now rated poor", maintainability 78.56 -> 55.09), 1 better (`judge_metric`, complexity 7 -> 4), 288 objects looked at. |
| Thresholds: a small move is not listed | The 286 other members, which did not move, are in neither list. |
| The level chooses the objects | `--level type`: 1 worse (`SymbolFact`, complexity 3 -> 15, maintainability 80.24 -> 63.75, coupling out 2 -> 6), 0 better, 98 objects looked at. Coupling out shows on the type level and does not exist on the member level. |
| Crossing the bound of the rating is marked and ranks first | "(now rated poor)" next to the complexity of the worse object. |
| Screen | The tab "Metrics" shows the same two lists as tables with sortable heads (e2e `ui-change`, three themes; screenshots in `docs/screenshots/change/`). |

## Backed by tests only

`--markdown` and the button "Copy as Markdown" (#838), the argument `metrics` of the MCP tool `impact_of_diff` (#833) and the module level (the folder had no module with a move) are backed by the tests of the packages, not by this run. The screen itself was not clicked through by hand in this run either; it is backed by the e2e test `ui-change` (three themes) and its screenshots.

## Open points

- None for this feature. The comparison sees the same objects as the comparison of ADR 0097: a renamed object is not paired and is not in these lists.
