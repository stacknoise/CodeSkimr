# QA walk, repeat before M5 acceptance

Second run of 2026-11-15 on commit `03bc2a6` (main after #460). The first run (`qa-1-0.md`, commit `ac9081d`) was before the scan budget work, level 2 through the real path, the error format of `/mcp`, the language packs that load on first use (ADR 0059) and the test references (#430). Role: QA, no product code changed.

**Machine:** Linux container, 4 vCPU, release build, no load during the timings. Absolute times are indicative (blueprint 16.1 names a mid-range laptop). Not covered: Windows and macOS (#104, #131, #327), level 2 indexers (not installed here, so every scan says the tools are missing, which is the intended message; level 2 is in `m5.md`), the HTTP part of `/mcp` (needs a keystore, not available here).

Legend: ✅ ok · ◐ ok with remarks · ❌ finding (issue linked)

## Verdict

**No blocker found by the walk, but one finding is wrong data on a main screen and needs the product owner's decision.** Two new findings: #464 (layer map counts ambiguous references, so the map of a project with layers shows a full mesh and false violations) and #465 (loading the language packs of a project with many languages takes longer than its scan and is in no budget). Neither stops the product from working; whether #464 is a blocker for 1.0 is the product owner's call, because the dependency map is a main screen. Older findings: #399 and #401 are closed (#399 checked again below, #401 not: no project with exactly one layer in this run); #400 and #402 are open and still reproduce.

## 1. Screen walk (`tools/acceptance/qa-walk.mjs`)

Run unchanged on pallets/click and on CodeSkimr itself: every rail screen and the quick search in Hell, Petrol and Dunkel, in German and English (12 runs). **No console error, no failed request, no horizontal overflow, no untranslated key.** The report lists 76 (click) and 41 (CodeSkimr) strings that are equal in both languages; going through them they are names, paths, code identifiers and words that are the same in German ("Status", "Branch", "Theme", "Link", "Pipeline", "Petrol"), plus the sample name `haac-bridge` of the settings preview that was already noted in the first run. No missing text.

By eye on the Scan screen (click, Petrol, English): the table of targets shows "Scan 100k lines 433 ms" and "Graph layout 14 ms" in the right colours, the pipeline and the language table are as in the mock. The dependency screen of CodeSkimr (Dunkel, English) shows the finding of #464.

The script cannot see what it does not click. New since the first run and **not covered by the script**: the "violations only" switch, the namespace level and type focus of the map, the usage screen. A later version of the script should open them (not done here to keep this run comparable with the first).

## 2. Scans (`codeskimr scan`, two runs each, wall time and printed time)

| Project | Files | Symbols | References | Scan (printed) | Pack load | Wall | Result |
|---|---|---|---|---|---|---|---|
| pallets/click (Python) | 91 | 2,546 | 11,730 | 0.26 to 0.30 s | 0.09 s | 0.4 s | ✅ same counts as before |
| Newtonsoft.Json (C#, 197k lines) | 951 | 15,831 | 439,356 | 2.7 to 2.8 s | 0.77 s | 3.6 s | ◐ was 4.1 s; about 1.4 s per 100k lines without the pack load (0.77 s extra, #465), still over 1 s in this container (#410) |
| google/gson (Java) | 266 | 6,048 | 74,751 | 0.61 to 0.68 s | 0.16 s | 0.8 to 0.9 s | ✅ was 1.05 s |
| CodeSkimr itself (Rust, TS, TSX, JS, Python, Java, Kotlin, C#) | 749 | 13,476 | 166,439 | 1.5 to 1.9 s | **2.12 s** | 3.8 to 4.2 s | ◐ pack load longer than the scan (#465) |

Symbol and reference counts of click, Newtonsoft.Json and gson are identical to the first run, so the scan work since then changed time and no result. All exit codes 0. The CodeSkimr row has 749 files and 166,439 references (first run 724 and 135,874), because the repository grew.

Of the 166,439 references of CodeSkimr 148,936 (89 %) are `probable` (name matches several symbols, `RES-301`). That is level 1 by design (the share of `probable`, not the "unchecked" share of ADR 0033), but it is the cause of #464.

## 3. MCP over stdio (`codeskimr mcp`, project click)

`initialize` (version 0.9.0-preview.1), `tools/list` (11 tools), `get_project`, `search_symbols`, `find_usages`, `get_symbol` and `rescan` answer. Checked since the first run:

- #399 (symbol id as a number): a number is accepted now (`get_symbol` with an id that does not exist gives `MCP-202`, not `MCP-101`). ✅
- `rescan` with `paths` (#460): `["src/click/core.py","nope.py"]` runs incrementally in 212 ms and echoes both paths; the path that names no file is not reported (#461, known).
- #400 item 1 still reproduces: an unknown tool gives `API-201` with the text "The REST request names a command or route that does not exist".
- #402 still reproduces: rust-analyzer as a rustup shim without the component is reported as "version not readable".

## 4. Findings

| Class | Issue |
|---|---|
| New, wrong data | #464 layer map counts `probable` references: 25 violations that are very probably false (the layering is enforced in CI; the 25 were not checked one by one) in CodeSkimr's own map |
| New, measurement | #465 pack load of the first scan (2.1 s for eight packs) is in no budget |
| Open, still reproduce | #400, #402 |
| Closed since the first run | #399, #401 |
