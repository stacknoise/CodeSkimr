# 0059 – Load language packs when their language is first needed

Status: proposed (draft for Anton; the prototype is on its own branch and changes blueprint 8.2 only after the decision)

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
5. **Blueprint 8.2** gets one sentence in place of "their packs are loaded on first use": "Packs of every tier are loaded on first use; the tests load all of them". Section 16 (budgets) is not changed.

## Options not chosen

- **Compile the three queries of a pack on three threads.** Measured: no gain (C# 0.75 s before and after), `references.scm` alone is 0.52 s.
- **Keep loading all packs eagerly and start the load in the background** while the server starts. The cost stays and competes with the first scan for the cores.
- **Cache compiled queries on disk.** Tree-sitter has no serialisation of a compiled query; not possible without a fork.
- **Split `references.scm` into smaller queries.** The analysis cost grows faster than linearly with the number of patterns, so smaller queries compile faster, but each file needs one more tree walk during every scan. Not measured yet; a possible follow-up.

## Consequences

- Start-up of `serve`, `mcp` and every CLI command that does not scan drops by about 1 s; the first scan of a language pays that language's pack once.
- A broken pack is no longer an error at start-up but at the first file of its language (and in CI). The error text and code do not change.
- The benchmarks of `tools/bench` do not build language packs, so no budget changes; the measurements above are by hand. A benchmark that scans real code (issue #410, plan step 3) loads the packs before the timed part.
- The stage timing of the scan (`parse_micros`) is not affected.
