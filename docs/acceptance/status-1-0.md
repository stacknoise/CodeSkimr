# Acceptance status M1 to M6 before the 1.0 freeze

State of 2026-11-20, `main` at `a43b233`. This page does not replace the reports; it says per milestone what changed since the report, what is still open and who decides. Issue states are those of the open-issue list of that day. Nothing here is measured again: a figure that needs a new run is marked as such. Acceptance itself is the product owner's decision (one card for M1 to M6).

| Milestone | Report | Verdict of the report | Since the report | Still open | Needed from the product owner |
|---|---|---|---|---|---|
| M1/M2 | `m1-m2.md` | partly met | Budgets of #107 and #108 closed; ADR 0059/0063 accepted (pack load 1.5 s for Rust and C#); the 100k-LOC budget stays at 1 s; Newtonsoft.Json (197k lines, about 1.4 s per 100k LOC) stays over it as a known limit (#410; the product owner decided this on the card of 2026-11-20, the issue does not yet carry `after-1.0`) | Scan and re-scan time on a project of 100k LOC or more on the laptop; the private C# sample (`private-reference.ps1`); usage of the MCP tools by the agents in daily work (`check_conventions` noise #110 is closed; #93 stays open, not a 1.0 blocker) | Run `private-reference.ps1`; accept with the named limit of #410 (then label it `after-1.0`) |
| M3 | `m3.md` | acceptable with two leftovers | Syntax colours exist (ADR 0064 accepted, #484); the visual test runs in all three themes (`web/e2e/ui-visual.spec.ts`, #332 closed) | Long Rust namespace names in the tree (#336, enhancement, after 1.0 proposed) | Accept; confirm #336 as after 1.0 |
| M4 | `m4.md` | acceptable with leftovers | #333 and #335 closed | Canvas for very many nodes and C# layers (#198), resolver recall for the map on C# (#145): both after 1.0 proposed | Accept with the two named leftovers |
| M5 | `m5.md` (draft) | not accepted as it stands | ADR 0033 stays; ADR 0037, 0063 and 0064 accepted (card of 2026-11-20); decision of the product owner on the card of 2026-11-20 (not yet recorded in `m5.md`, whose open point 1 still reads undecided): languages below 99 % (Python, C#, Kotlin) are shown as approximation in the release notes and in the UI; the release notes side is in the checklist, the UI sign is still to be built (work package of the budget thread); import lines no longer count as uses (#250 closed) | Re-measure TypeScript and Python after #250 (#571); C#: #258, #259, #395; TypeScript #394; Rust module paths #436; Kotlin has no reference tool; level 1 recall before #133 has no verdict | Accept with the stated approximations, or name what blocks 1.0 |
| M6 | `m6.md` | preparation, not run on the product owner's machine | Locked memory for credentials (#298 closed, ADR 0077); Git update by fetch and the open points #36, #363, #365 closed; main switch of level 2 for every run (#567, in review) | Run on Windows with a private repository (`m6-private.ps1`); start file and browser start on Windows and macOS (#104, #372); folder dialog on both systems (#559) | Run the script and the manual checks; accept |

## Proposal: issues for the label `after-1.0`

None of these blocks a point of `docs/release/checklist.md`; each is a follow-up, a nice-to-have or a test addition.

- UI details and follow-ups: #584, #576, #336, #488, #487, #486, #494, #198, #515, #516, #517, #518, #522.
- Tests and docs: #565, #561, #534, #533, #571, #476, #456, #570.
- Resolver and level 2 accuracy beyond the stated approximations: #145, #134, #74, #258, #259, #257, #232, #366, #394, #395, #436, #439, #325.
- Tooling, dependencies and refactorings: #548 (the exception of ADR 0073 runs until 2027-02-28), #545, #581, #562, #361, #135, #93, #209.
- Already `after-1.0`: #379, #289, #290.

Open and needed before the freeze (not `after-1.0`): #556 (closed by #567 once merged), #528 (check: #564 fixed the macOS limits test), #536, #532 (level 2 switch details: decide fix or after 1.0), #104, #372, #559 (product owner's machines), #410 (decided: after 1.0, known limit in the release notes), #328 (check whether #379 and the notes close it).

## What the QA walk 3 covers

`tools/acceptance/qa-walk.mjs` now also opens a type with its coupling tiles, the usages of it, the dependency map on every level (layers, "violations only", namespaces with depth 2, type focus) and the projects screen after the scan, each in three themes and both languages. A control it cannot reach is reported as a finding.
