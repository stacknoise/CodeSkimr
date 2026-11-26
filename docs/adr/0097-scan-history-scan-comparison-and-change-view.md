# 0097 – Scan history, scan comparison and the view "Change"

Status: accepted (version 1.1 "Change and agent package" decided by the product owner on 2026-11-23, history variant A of mock 28 on 2026-11-26, mocks of 1.1 approved on 2026-11-26; supersedes the plan of comparing snapshots in ADR 0068 "WP-103" and the entry "snapshot comparison" of ADR 0034)

## Context

A snapshot is the scan state of a project; the last N are kept (`storage.snapshot_retention`, default 5, blueprint 6.2). Three things build on two of them: a list of the last scans with their duration (issues #711 and #725), the comparison of two scans (concept G1) and the view "Change" that shows what a difference hits (concept E1). ADR 0093 made a scan without changes write no snapshot, so the retained snapshots are exactly the scans that changed something. ADR 0068 already stores a content hash per type and member (token stream without whitespace and comments) and finds a symbol again by kind, qualified name and parameter types; the comparison reuses both and needs no new field per symbol. The concept is in the project folder (`entscheidungen/erweiterungen-nach-1-0.md`), the mocks are 28 a (history), 29 (comparison) and `erw-auswirkung` (impact).

## Decision

1. **History.** Two nullable columns on `snapshots` (migration 0023): `duration_ms` (wall time of the scan that wrote the snapshot, from the start of the discover stage to the commit) and `files_changed` (files of the scan that were new, changed or gone against the previous snapshot; none for the first scan). A snapshot written before the migration has neither; the screen shows a dash. The list is the retained snapshots, newest first, with time, mode, duration, files and changed files, and no other limit than the retention (the tab says how many are kept). `ScanSummary` of the REST interface carries both fields.
2. **Identity.** Two symbols are the same symbol when kind, qualified name and parameter types are equal (the key of ADR 0068). Only types, callables, properties, fields, constants, events and enum members take part; namespaces, modules and parameters do not.
3. **Result of a comparison** (base = older, target = newer, both snapshots of one project):
   - *added*: key only in the target;
   - *removed*: key only in the base;
   - *changed*, with one or more reasons: `signature` (the signature text differs), `visibility` (differs; narrowing is marked), `content` (the content hash differs, nothing else);
   - *moved*: same key in another file, or a removed and an added symbol of the same kind in the same file with the same non-empty content hash and no other candidate on either side (a rename). A moved symbol is not also listed as removed and added. An ambiguous pair stays removed and added; the comparison never guesses.
   A symbol without a content hash (level 2 symbols, for instance) can be *changed* by `signature` or `visibility` only. Both snapshots must come from the same scan naming rules; otherwise the comparison fails with a retryable error that asks for a full scan (the naming rules are stored per snapshot already).
4. **Impact.** For every removed symbol and every symbol changed in `signature` or `visibility`, the direct usages in the **base** snapshot (the references that point at it) are listed with their confidence (sure, probable). A usage is *adapted* when the symbol it sits in is itself changed, moved or removed in the comparison, and *not touched* otherwise; the second group is the checklist. Changes in `content` only and added symbols are shown without a checklist (their usages cannot break by this change). Only direct usages are known: calls by reflection, in strings or from other repositories are not seen, and the view and every output say so. New layer violations and new cycles are those present in the target and not in the base (the layer graph is stored per snapshot); symbols marked critical (ADR 0068) are named when a change touches them.
5. **Choice of the two states.** The user picks two snapshots of the list or "previous" and "newest". A Git branch or commit as base (`--base main`) would need a scan of that state into a temporary snapshot; it is **not** part of this decision and becomes its own issue with its own ADR (it touches the retention, the clone and credentials).
6. **Surfaces** (one use case in `codeskimr_application`, four ways in, same JSON with `"schema": 1`):
   - registry commands `scan_list`, `scan_compare` and `scan_impact` (permission `read`);
   - CLI `codeskimr scans <project>`, `codeskimr compare <project> [--base <id>] [--target <id>]` and `codeskimr impact <project> [--base <id>] [--target <id>]`, each with `--json`; default base is the snapshot before the newest, default target the newest;
   - MCP `impact_of_diff` (the only new tool in this decision; the list grows to 14);
   - UI: the tab "Scan" gets the table of mock 28 a with the button "Compare with current"; a new screen "Change" (rail entry, mock 29 and `erw-auswirkung`) with the tabs "Impact" and "Comparison". Binding mocks: 28 a, 29 and `erw-auswirkung`; they are copied to `docs/mocks/` with the first UI package.
   Limits for the output (number of symbols, usages per symbol) come from `config/default.toml` (`compare.*`); the answers say when they are cut.
7. **Retention.** A comparison reads two retained snapshots; a snapshot that retention removed is not offered. A running comparison holds a read transaction like any other read and does not block a scan.
8. **Texts and codes.** Texts in German and English (Fluent). A new error group `CMP` (snapshot not found, different naming rules, comparison cut) is registered with the package that uses it.
9. **No new dependency.**

## Consequences

- One change of the schema (migration 0023) and one use case serve the screen, the CLI and the agent. Trends and the live view (later) can reuse the comparison.
- "Changed" is exactly as good as the content hash: a change of a comment or of whitespace is none, a change of an unrelated line inside a method is one.
- The checklist "not touched" is exact to the symbol that holds the usage, not to the line. A usage in a method that changed for another reason counts as adapted; the view shows the line either way.
- Renames inside one file with a changed body, and renames together with a move, show as removed and added.
- Work packages, in this order, each its own PR: (1) this ADR with the blueprint changes; (2) migration 0023, duration and changed files stored and shown in the tab "Scan" (#711, #725); (3) domain and application: comparison and impact, with storage reads; (4) registry commands, REST, CLI and MCP; (5) the screen "Change" with the mocks; (6) documentation, manual and the acceptance on a real project.
