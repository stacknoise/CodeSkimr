# 0059 – Load language packs when their language is first needed

Status: accepted (Anton, 15 Nov, with option (a) for the benchmarks, see Consequences)

## Context

Blueprint 8.2 lets the basic-tier packs load on first use and says nothing else about when the full-tier packs load; the implementation loads every full-tier pack when the set of packs is read, on at most one thread per core, so that a broken pack is reported at the start (issue #66). Compiling the three queries of a pack (`symbols.scm`, `references.scm`, `metrics.scm`) is the cost: the static analysis of a Tree-sitter query (`ts_query__perform_analysis`) is slow for packs with many patterns.

Measured for issue #410 (release build, 4 cores, Newtonsoft.Json `Src`, 951 files; one-file C# folder; empty folder):

| Case | Before | With lazy loading |
|---|---|---|
| `codeskimr scan` of an empty folder (start-up only) | about 1.1 s | 0.05 s |
| `codeskimr scan` of one C# file | about 1.1 s | about 0.75 s |
| `codeskimr scan` of Newtonsoft.Json (C# only), wall time | about 4.0 s | about 3.9 s |

- All full-tier packs together cost about 1.1 s of start-up on 4 cores; C# alone costs about 0.75 s (`references.scm` 0.52 s, `symbols.scm` 0.19 s, `metrics.scm` 0.11 s). In a callgrind profile of a scan of this corpus, the query analysis at load is 37 to 57 % of all instructions.
- The start-up is paid by every use of the program: each CLI call, `serve` and `mcp` pay it before the first request, also when nothing is scanned. The lines "Scanned in x s" do not include it today; with lazy loading the load of the first pack moves into the scan (C#: about 0.75 s more in the printed time, about the same in wall time).
- A scan of a project in one language no longer builds the other five full-tier packs. A scan of a project with C# only gains nothing, because C# is the longest pack and was already the long pole of the parallel load.

## Decision

1. **Every pack is loaded when a file of its language is first analyzed**, whatever its tier. `LanguagePacks::load` reads only the manifests (`language.toml`), so a pack whose manifest is wrong is still reported at the start.
2. **The first thread that needs a pack builds it; the others wait** for the result (`OnceLock::get_or_init`). Today, workers that reach the first file of a language at the same time each compile the pack and all but one result is discarded.
3. **A pack that cannot be built is remembered as failed** for the run, as the basic tier already does (issue #66). Every file of its language fails with `PRS-301` (language pack invalid); the other languages are scanned.
4. **`LanguagePacks::verify()` loads every pack** (one thread per core) and returns the first error. The tests of the packs and CI use it (`every_installed_pack_loads`), so a broken pack in the repository is found by CI and not by a user. A scan does not call it.
5. **Blueprint 8.2** gets one sentence in place of "their packs are loaded on first use": "Packs of every tier are loaded on first use; the tests load all of them", and says that the load time is left out of the reported scan time (option (a) below). The numbers of section 16 are not changed.
6. **The analyzer port reports the load time** (`FileAnalyzer::pack_load_time`, the time during which at least one pack was being loaded, so packs that load at the same time count once); `Session::run_job` subtracts what was loaded during the scan from the reported time.

## Options not chosen

- **Compile the three queries of a pack on three threads.** Measured: no gain (C# 0.75 s before and after), `references.scm` alone is 0.52 s.
- **Keep loading all packs eagerly and start the load in the background** while the server starts. The cost stays and competes with the first scan for the cores.
- **Cache compiled queries on disk.** Tree-sitter has no serialisation of a compiled query; not possible without a fork.
- **Split `references.scm` into smaller queries.** The analysis cost grows faster than linearly with the number of patterns, so smaller queries compile faster, but each file needs one more tree walk during every scan. Not measured yet; a possible follow-up.

## Consequences

- Start-up of `serve`, `mcp` and every CLI command that does not scan drops by about 1 s; the first scan of a language pays that language's pack once.
- A broken pack is no longer an error at start-up but at the first file of its language (and in CI). The error text and code do not change.
- **The scan benchmarks get slower by the pack load, and the ratio check against the base fails.** The benchmarks of `tools/bench` run the real binary and read the "Scanned in" time, which starts after the program has started and now contains the load of the first pack. The CI run of the prototype shows `scan/full_100k_loc` 0.60 s → 0.72 s (+20 %) and `scan/full_100k_loc_csharp` 2.62 s → 3.18 s (+21 %). Nothing got slower for the user: the same time was paid at start-up before. The decision for the implementation: **(a)** keep the meaning "time of the scan without loading packs" and leave the load out of the reported time (no benchmark measures the pack load yet, as none did before: the cold start budget ends when the UI is usable); this needs the load time of the packs to be reported by the analyzer port and subtracted in `Session::run_job`; or **(b)** accept that "Scanned in" includes the first load, and merge once the base of the ratio check includes it (the check passes for every later PR; the PR that brings the change needs an exception from Anton for the `bench` check). Chosen: (a), because the budget in blueprint 16.1 then keeps its meaning. A benchmark of the pack load is a follow-up (issue #426). The clock is one per analyzer, so a scan can subtract loads of other workers or scans that ran at the same time; its reported time is then a little too short, and the first scan of a single C# file reports less than the wait of the user.
- The stage timing of the scan (`parse_micros`) is not affected.
