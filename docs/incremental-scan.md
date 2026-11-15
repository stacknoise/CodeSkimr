# Incremental scan

Blueprint section 7: only files whose hash changed are analysed, dependent references are resolved
again. This page describes how the pipeline in `crates/application` does it (`pipeline/carry.rs`).

## Rule

An incremental scan gives the same references as a full scan of the same files (issue #49). The
full scan is the oracle: a test in `crates/app/src/incremental_tests.rs` runs both ways over a real
project (rename, delete, new target, second candidate, new overload) and compares them.

## Steps

1. **Discover.** The discovery adapter marks changed files. A file the previous snapshot does not know is
   marked as changed here, whatever the adapter says. `ProjectRepository::latest_state` reads the
   files, the symbols, the references and the names as written of the previous snapshot in one
   transaction, so a snapshot written in between cannot make them disagree. `Carried::from_previous`
   splits it into
   - the unchanged files and their symbols,
   - the references from unchanged files to symbols that still exist,
   - the **names** of the unchanged files, that is every occurrence of a name as the analysis found it
     (`PendingReference`), resolved or not,
   - the **removed** paths (in the previous snapshot, no longer on disk).
   A snapshot that does not store the names (written before migration 11) is followed by a full scan.
2. **Analyze.** Only changed files are analysed.
3. **Resolve.** The global symbol table is the symbols of the unchanged files plus the new symbols. The
   resolver gets the new names and the names of the unchanged files, so it does exactly what a full scan
   does: a reference to a symbol that was renamed or moved is found again or gone, a name that matched
   nothing before finds a target that a changed file now declares, and the confidence is the one of the
   new situation (a second candidate makes a `certain` reference `probable`, and the other way round).
   `Carried::reconcile` then compares the result with the kept references of unchanged files. One that
   is produced again with the same target and confidence stays as it is, a new one is written, a kept
   one that is no longer produced is dropped (`SnapshotData::dropped_references`).
4. **Measure.** The calculator sees all symbols and all references of the new snapshot, so coupling
   (Ce, Ca, instability) is computed on complete data and stored for the new snapshot alone (ADR 0037). The metric counts (`FileFacts::counts`) exist only
   for the analysed files, so loc, cc, cc_max and mi are measured for those; the carried values of the
   other files stay. Metrics the calculator delivers replace the carried ones.
5. **Persist.** `SnapshotData` holds what is new, the names of the analysed files, `removed` and
   `dropped_references`; the store carries the rest, including the names of the unchanged files.

A full scan does none of this: `Carried` is empty and `removed` is empty. It stores the names of all files.

## Level 2

References that level 2 verified (ADR 0033) are not touched by level 1:

- A file with verified references keeps them. Level 1 does not add references to it and does not drop
  any, because level 2 replaced its level 1 references.
- A verified reference whose target is in a changed or removed file and is gone is discarded and
  counted (log event `verified_references_discarded`, with the count). It is not valid any more and
  is not kept silently. The occurrence falls back to level 1: if the name still resolves, the new
  reference is unchecked (`verified = false`) until level 2 runs again.
- The names of such a file are still given to the resolver, so it knows the type hierarchy of the
  whole project.

## Limits

- The names are stored once per file and shared by the snapshots that carry the file (migration 11,
  ADR 0032). A reference that is dropped is a shared row, so older snapshots that carry the same file
  lose it too, like level 2 does when it replaces the references of a file (ADR 0033).
- Every incremental scan resolves the names of all files again. That costs the resolve stage of a full
  scan (in memory, without parsing); recall comes before speed here. If the 300 ms budget of watch mode
  (blueprint 16) shows it, the set can be narrowed by name, but not before it is measured.
- `LayerEdge.violates_layering` still depends on the allowed directions of a layer (issue #45).

## Progress and events

Every stage reports its amount of work through `ScanProgressed` (`done` of `total`, both set at the end).
A stage with nothing to do ends with 0 of 0. The scan view of mock 5 shows six rows (get source, find
files, parse, symbols, resolve, metrics and save), the application layer has six stages with Parse and
Extract in one (ADR 0013). The rows do not map one to one: parse and symbols both follow the Analyze
stage, metrics and save follow Measure and Persist. The frontend package decides how the view derives
its rows from the events; no stage is added for it.

`codeskimr_source::publish_changes` watches a project folder and publishes `FilesChanged` (count of changed
files, `rescan` if the operating system lost events) after the debounce window. Starting the incremental
scan on that event is the watch mode of the composition root (`crates/app/src/watch.rs`, `codeskimr scan --watch`, ADR 0029): the changes that piled up while a scan ran become one incremental scan, or a full scan if `rescan` is set.
