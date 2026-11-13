# Incremental scan

Blueprint section 7: only files whose hash changed are analysed, dependent references are resolved
again. This page describes how the pipeline in `crates/application` does it (`pipeline/carry.rs`).

## Steps

1. **Discover.** The discovery adapter marks changed files. A file the previous snapshot does not know is
   marked as changed here, whatever the adapter says. `Carried::from_previous` then splits the previous
   snapshot (`ProjectRepository::latest_files` and `latest_index`) into
   - the unchanged files and their symbols,
   - the references between unchanged files (they stay valid and are carried over by the store),
   - the **removed** paths (in the previous snapshot, no longer on disk),
   - the references of unchanged files that point into a changed or removed file. Their target is turned
     back into a name, the qualified name of the old target symbol, and listed as `rebuilt`.
2. **Analyze.** Only changed files are analysed.
3. **Resolve.** The global symbol table is the symbols of the unchanged files plus the new symbols. The
   resolver gets the new pending names and the `rebuilt` ones. A name whose target is gone finds nothing
   and is dropped.
4. **Measure.** The calculator sees all symbols and all references of the new snapshot, so coupling
   (Ce, Ca, instability) is computed on complete data. The metric counts (`FileFacts::counts`) exist only
   for the analysed files, so loc, cc, cc_max and mi are measured for those; the carried values of the
   other files stay. Metrics the calculator delivers replace the carried ones.
5. **Persist.** `SnapshotData` holds what is new plus `removed`; the store carries the rest.

A full scan does none of this: `Carried` is empty and `removed` is empty.

## Limits

- A reference that is resolved again by the qualified name of its old target keeps its earlier
  confidence (`PendingReference::previous`); the resolver never raises it. The kept base-list references
  are resolved again as well, so that the resolver knows the type hierarchy between unchanged files
  (`Carried::inheritance_seed`).
- A name that matched nothing in the previous snapshot is not stored. If a changed file now declares it,
  the reference from an unchanged file appears only after that file changes or after a full scan.
- A reference that was resolved again is dropped if an identical one was kept (a name with candidates in
  an unchanged and a changed file).
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
scan on that event belongs to the watch mode of the composition root.
