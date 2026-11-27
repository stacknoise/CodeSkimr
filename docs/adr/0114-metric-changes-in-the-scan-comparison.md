# 0114 – Metric changes in the scan comparison (worse and better objects)

Status: accepted (asked for by the product owner on 2026-11-27; the mock 43 variant A "two lists" chosen by him on 2026-11-27; extends ADR 0097 and uses the coupling of ADR 0037 and the module numbers of ADR 0108)

## Context

The comparison of ADR 0097 says which symbols are new, removed, changed or moved. It does not say whether a changed symbol became harder or easier to maintain. The product owner wants to see which objects got worse and which got better by their metrics. The data is there: lines (`loc`), cyclomatic complexity (`cc`, `cc_max`) and the maintainability index (`mi`) are stored per symbol row (`metric_rows`), coupling (`ce`, `ca`, `instability`) per snapshot (`coupling_rows`). Both are readable per retained snapshot through the view `metrics`. The comparison already pairs the same symbol in two snapshots by its key (kind, qualified name, parameter types). Trends over months (concept A3) need stored sums per scan and are not part of this decision.

## Options

1. A new stored table of metric deltas per scan. Answers any pair without reading two snapshots, but doubles the writes of every scan and breaks with retention (the reason ADR 0097 rejected the change record).
2. Read the metrics of both retained snapshots at question time and compute the delta for the symbols the comparison pairs. No schema change, limited to the retained scans.
3. Judge only the changed symbols of the comparison (reasons `signature`, `visibility`, `content`) and ignore the rest.

## Decision

Option 2, and for the pairs: every symbol that exists in both scans under the same key (a moved symbol counts as the same symbol), so also a symbol whose own text did not change but whose coupling did.

1. **Direction of each metric and its level.** Members (callables, properties) have `loc`, `cc` and `mi`. Types have the same (sums over their members, ADR 0015) and in addition `ce`, `ca` and `instability`. Modules (namespaces, packages) have `ce`, `ca` and `instability` (ADR 0037 stores coupling for namespaces, modules and types only, not for members); so on the level `member` the metrics `ce`, `ca` and `instability` do not exist and are not judged, and a member does not inherit the value of its type. `cc` and `ce`: higher is worse. `cc_max` (the highest `cc` below a type) has no threshold of its own and does not count towards the verdict; it is shown as a detail of the row. `ca` is not judged either (being used more is no fault); it is shown as a detail. `mi`: lower is worse. `loc`: neutral, it only gets the mark "grew a lot" (see 3), never "worse" on its own. For a module, a rising `instability` of a module that many places use counts as worse (ADR 0108 risk); a falling one as better.
2. **Verdict of an object.** *Worse* if at least one directed metric moved to the worse by at least its threshold and none moved to the better by at least its threshold; *better* in the mirrored case; an object with both is *mixed* and listed on both sides with only the metrics that moved that way. An object under every threshold is not listed.
3. **Thresholds** (provisional until the product owner has confirmed them) live in `[compare]` of `config/default.toml`, one value per metric, in the unit of the metric: `metric_min_cc` 2, `metric_min_mi` 5, `metric_min_ce` 2, `metric_min_instability` 100 (thousandths) and `metric_loc_growth` 20 lines. They stop noise from counting as a change. Moving across a limit (the limits the statistics screen of ADR 0081 uses for "over the limit") is marked on the row and ranks first, whatever the size of the step.
4. **Order and cut.** Worst first, by the size of the move in relation to its threshold; ties by qualified name. The lists are cut by `compare.max_symbols` and say so; the totals are always counted.
5. **Not compared.** Added and removed symbols (they have no pair), a renamed symbol (it has another key, ADR 0097 addendum), symbols of level 2 origin (as in the comparison), and test code unless the caller asks for it.
6. **Surfaces** (one use case in `codeskimr_application`, same JSON with `"schema": 1`):
   - registry commands `scans.metrics` and `scans.metrics_markdown` (permission `read`; inputs: project, base, target, `level` = module | type | member, `direction` = worse | better | all, `metric`, `include_tests`);
   - CLI `codeskimr compare <project> --metrics [--worse | --better] [--metric cc,mi,loc,ce,instability] [--level module|type|member] [--base <id>] [--target <id>] [--json] [--markdown]`;
   - MCP: no new tool. `impact_of_diff` gets the argument `metrics` (boolean, default false) and adds the lists `worse` and `better`; the tool list stays as ADR 0097 left it;
   - UI: a third tab "Metrics" on the screen "Change" with mock 43 variant A (two lists, filter by metric, "Copy as Markdown" through `scans.metrics_markdown` as ADR 0092 asks).
   The comment of the pull request action (ADR 0109) may take the three worst rows from the same answer.
7. **Retention.** Only retained scans can be compared; a longer view per object is a later decision (A3 stores sums only).
8. **Texts and codes.** German and English through Fluent. The errors are those of the group `CMP` (ADR 0097); no new code is needed. No new dependency.

## Consequences

- No migration: reads of two snapshots, linear in the number of paired symbols; the metric rows of a symbol shared by both snapshots are the same row, so only changed symbols can differ and the read can start from the symbols whose row or whose coupling differs.
- The verdict is only as good as the thresholds; they are in the configuration and can be tuned from what is seen on real projects.
- A symbol rewritten without a change of its numbers is not listed here (the comparison lists it as changed).
- Work packages, one pull request each: (1) domain and application: pairing, verdict, thresholds; (2) registry commands, CLI and MCP argument; (3) the tab "Metrics" with the mock copied to `docs/mocks/` and screenshots; (4) manual and acceptance on a real project.
