# Incremental scan

Blueprint section 7: only files whose hash changed are analysed, dependent references are resolved
again. This page describes how the pipeline in `crates/application` does it (`pipeline/carry.rs`).

## Rule

An incremental scan gives the same references as a full scan of the same files (issue #49). The
full scan is the oracle: a test in `crates/app/src/incremental_tests.rs` runs both ways over a real
project (rename, delete, new target, second candidate, new overload) and compares them.

## Steps

1. **Discover.** The discovery adapter marks changed files. A file the previous snapshot does not know is
   marked as changed here, whatever the adapter says. `ProjectRepository::latest_head` reads the
   files of the previous snapshot and `snapshot_symbols` its symbols once a change is known; both
   name the snapshot, so a snapshot written in between cannot make them disagree. The references are read later, and only those the scan needs
   (see "Which references are read"). `Carried::from_previous` splits the state into
   - the unchanged files and their symbols,
   - the references from unchanged files whose target is gone,
   - the **names** of the unchanged files, that is every occurrence of a name as the analysis found it
     (`PendingReference`), resolved or not,
   - the **removed** paths (in the previous snapshot, no longer on disk).
   A snapshot that does not store the names (written before migration 11) is followed by a full scan.
   If the scan sees exactly the files of the previous snapshot, each with the stored hash, nothing
   changed: the symbols are not read (`latest_head` reads the files only), the next three stages pass
   through, and the persist stage writes no snapshot but confirms the previous one with the time of
   the scan (`SnapshotStore::confirm_latest`, ADR 0093).
2. **Analyze.** Only changed files are analysed.
3. **Resolve.** The global symbol table is the symbols of the unchanged files plus the new symbols. The
   resolver gets the new names and the names of the unchanged files, so it does exactly what a full scan
   does: a reference to a symbol that was renamed or moved is found again or gone, a name that matched
   nothing before finds a target that a changed file now declares, and the confidence is the one of the
   new situation (a second candidate makes a `certain` reference `probable`, and the other way round).
   `Carried::reconcile` then compares the result with the kept references of unchanged files. One that
   is produced again with the same target and confidence stays as it is, a new one is written, a kept
   one that is no longer produced is dropped (`SnapshotData::dropped_references`).
4. **Measure.** The calculator sees all symbols of the new snapshot and all its references, those
   left in the store as pairs of start and target, so coupling
   (Ce, Ca, instability) is computed on complete data and stored for the new snapshot alone (ADR 0037). The metric counts (`FileFacts::counts`) exist only
   for the analysed files, so loc, cc, cc_max and mi are measured for those; the carried values of the
   other files stay. Metrics the calculator delivers replace the carried ones.
5. **Persist.** `SnapshotData` holds what is new, the names of the analysed files, `removed` and
   `dropped_references`; the store carries the rest, including the names of the unchanged files.

A full scan does none of this: `Carried` is empty and `removed` is empty. It stores the names of all files.

## Which names are resolved again

The resolver reads of a symbol its name, kind, qualified name and parent, and of a type its base
list (`pipeline/shape.rs`). `Carried::due_names` compares that shape of the changed and removed
files, before and after:

- **Always** (the part that a typical save needs, where bodies and comments changed and
  declarations did not): the names that pointed into the changed files, because the ids of their
  symbols are new (including an import that pointed into them, also when it is the only name of its
  file), the imports of the files that hold such a name, and the base lists, because the resolver
  learns from them what a file sees and which type inherits from which.
- **A symbol differs** (added, removed, renamed, moved or changed in kind, or a file added or
  removed): `Shape::delta` lists the simple names and the qualified names of these symbols. The
  resolver looks a name up by the simple name of its target and, for a typed or named receiver, by the
  simple name of the receiver, so every name whose target or receiver is one of the changed simple
  names is resolved again (`StoredNames::Targeting`). An import is looked up by its qualified name;
  every name of a file with an import of a changed qualified name is resolved again, because the
  import decides what the file sees (`ImportsTargeting`, `OfFiles`). A moved symbol carries its
  members with it, since their qualified names change too.
- **A base list differs** (a type gets, loses or changes a base): all names of the unchanged files,
  because what a type inherits decides how members resolve anywhere.

The rule is proven against the full scan as the oracle: the cases of
`crates/app/src/incremental_tests.rs` (C#, Python, TypeScript, Java, Kotlin, Rust) and a test that
draws random changes of a C# project (files that are added, deleted, renamed, moved to another
namespace, given another base class, another import, other calls) and compares every step with a
full scan. Taking away any part of the rule makes that test fail.

`Carried::reconcile` drops a kept reference only if its name was resolved again.

## Measured

CodeSkimr itself as the corpus (`crates/` and `tools/`: 493 Rust files, 76 000 lines, 10 154 symbols,
84 880 references), release build, one file edited between scans, the same machine and files for
base (`origin/main`) and this change, in milliseconds:

| | base | all names every time | same shape (now) |
|---|---|---|---|
| incremental scan, total | 360 to 435 | 460 to 510 | 355 to 400 |
| of it resolve | 14 to 29 | 120 to 150 | 22 to 27 |
| of it discover (reads the previous snapshot) | 133 to 181 | 180 to 210 | 180 to 196 |
| full scan, total | 1360 | 1840 | 1435 |

The budget of blueprint 16.1 is 300 ms for one saved file on the reference laptop. This machine is
not that laptop and the base already needs 360 ms on it (discover and persist are 80 percent of
that, they are not part of this change), so the absolute number is no verdict on the budget; what
counts is the change against the base (CI measures it with `tools/bench`, ratio against the base
commit, ADR 0014, limit 10 percent). Resolving all names every time would have cost 25 to 35 percent
more, which is why a change of the declarations is the only case that does it.

The names are not read with the previous state. `latest_state` reads files and symbols
(and says whether names are stored); the resolver asks for the names it needs through
`ProjectRepository::stored_names`: the base lists (inherit and implement, partial index
`pending_references_bases`), the names at the sites that lost a target together with the names of the
callers of removed symbols (index `(file_id, from_symbol)`), the names whose target or receiver is
a changed simple name, the imports of the involved files, or all names when a base list changed. An
edit therefore reads a few hundred rows instead of 64 000. `tools/bench` (release, this machine, base and head back to back): `rescan/one_file_100k_loc`
123 ms against 133 ms, `rescan/one_file_100k_loc_csharp` 176 ms against 180 ms.

The names take space: 64 333 rows in `pending_references` for the 84 880 references, 3.3 MB of a
database of 37.1 MB with five snapshots (32.8 MB without; the rows are shared between snapshots that
carry a file, ADR 0032). One row is about 50 bytes.

A save that adds a declaration (issue #408; CodeSkimr's own `crates/` as the corpus, release build,
one function added to a file, resolve stage and the whole rescan that watch mode reports, this
machine):

| | all names (before) | by changed symbols (now) |
|---|---|---|
| resolve | 226 ms | 43 to 59 ms |
| rescan, total | 410 to 483 ms | 231 to 273 ms |

A save that changes only a body takes 190 to 220 ms and 26 to 34 ms in the resolve stage, so a
declaration costs about 40 ms more. A change of a base list still resolves all names.

## Level 2

References that level 2 verified (ADR 0033) are not touched by level 1:

- A file with verified references keeps them. Level 1 does not add references to it and does not drop
  any, because level 2 replaced its level 1 references at the places where it answered.
- A verified reference whose target is in a changed or removed file and is gone is discarded and
  counted (log event `verified_references_discarded`, with the count). It is not valid any more and
  is not kept silently. The occurrence falls back to level 1: if the name still resolves, the new
  reference is unchecked (`verified = false`) until level 2 runs again.
- The names of such a file are still given to the resolver, so it knows the type hierarchy of the
  whole project.

## Content hashes and alerts (ADR 0068)

The parse stage takes a content hash of every type and every member (`SymbolKind::has_content_hash`):
BLAKE3 over the leaf tokens of the symbol's span, each token followed by a unit separator, comments
left out, so whitespace, layout and comments do not change it. It is the same function as the file
hashes. A type hashes its own whole span, including its members; a namespace has no hash of its own
and gets a derived one from the sorted hashes of its children when it is read. Modules and parameters
have none. All 13 language packs are covered, because the hash only needs the tree and the comment
kinds of the pack. In Rust and Go the methods of a type are written outside its span (`impl` blocks,
receivers), so the hash of such a type does not change when a method changes; the method has its own
hash. Python has no token for the start and end of a block, so its pack lists the node kind `block`
in `block_nodes` and the walk puts a marker for the start and end of every block into the stream: moving
a statement out of an `if` changes the hash, a comment or a blank line does not. Another pack with an
indentation grammar would need the same entry.

The hashes are stored with the symbol rows (`symbol_rows.content_hash`, schema version 16), so a file
that is carried over keeps its hashes and an incremental scan stores the same hashes as a full scan of
the same files (`crates/app/src/alert_tests.rs`). The naming revision rose to 4 with this change, so the
first scan after the update is a full one.

After every snapshot (full, incremental, watch mode, Git fetch) the composition root runs the alert
step (`codeskimr_application::alerts`) as a read-model step: it runs after the snapshot transaction, a
failure is logged (`alert_step_failed`) and never undoes the snapshot. For each critical annotation it
looks up the target in the new snapshot:

- no baseline yet: the current state is stored as the baseline, no alert;
- content hash or signature hash differ from the baseline: alert `changed`; lines of code and
  complexity are recorded as well, but they alone never raise an alert, so a cosmetic edit stays silent;
- the symbol is gone and no symbol with the same simple name exists: alert `removed`, with the number
  of names in the snapshot that still point at it;
- the symbol is gone and one with the same simple name exists elsewhere: alert `orphaned`, naming the
  closest one;
- there is one open alert per annotation; another sighting counts it once more, but only if the symbol
  looks different from the last sighting (the alert keeps what it saw), so unrelated saves add nothing.
  A dismissed alert is closed and a new change opens a new one.

A note or a task whose target is gone is marked `without_target` and raises no alert. The trigger of
an alert is `scan`, `watcher` (watch mode) or `fetch` (every scan of a Git project). The step publishes
`AlertsChanged` (project and number of open alerts) on the event bus; the REST and WebSocket API do not
carry it yet. Without annotations the step costs two cheap statements (does the project have any, then
the list of critical ones). With 100 critical annotations on 20,000 symbols it takes about 1 ms
(`alerts/step_100_critical`, checked against `incremental_rescan_ms`; the targets are found through the
index `symbols_by_name`), and with 50 more whose symbols are gone about 2 ms (`alerts/step_50_vanished`; the
lookups of namesakes and of the names that still point at the symbol use the indexes `symbols_by_simple_name`
and `pending_references_by_target`; the step makes them when the first critical annotation needs them, so a project without one does not pay for them on writing, see `docs/benchmarks.md`).
While an alert is open the step looks at a vanished symbol again at every scan: if it is now `orphaned`
instead of `removed` or the other way round, or the closest symbol changed, the alert says so; if the
symbol is back with the state the alert saw last, the alert turns to `changed` again without counting
another time.

## Limits

- The names are stored once per file and shared by the snapshots that carry the file (migration 11,
  ADR 0032). A reference that is dropped is a shared row, so older snapshots that carry the same file
  lose it too, like level 2 does when it replaces the references of a file (ADR 0033).
- A change of the declarations (see above) resolves the names of all files again. That costs the
  resolve stage of a full scan (in memory, without parsing); recall comes before speed here. A
  narrower rule for that case, by the names of the symbols that differ, is possible but must be
  proved to give the full scan's result in every case, so it is not part of this change.
- The oracle tests (`crates/app/src/incremental_tests.rs`) cover C# and Python. The names of other
  languages are stored and resolved by the same code (the receiver kinds are covered by the storage
  tests), but no oracle test of TypeScript, Java, Kotlin or Rust runs yet.
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

## Which references are read

ADR 0086. A symbol of an unchanged file is **touched** if a name written in it is resolved again or
one of its references lost its target. The scan reads

- the references that point at a symbol of a changed or removed file (`StoredReferences::To`),
- the files with references verified by level 2 (`verified_files`),
- the references that start in a touched symbol (`StoredReferences::From`), in `Carried::reconcile`,
- the old references of the changed and removed files (`StoredReferences::In`), only to count.

Of all other references it takes two things without reading them: the pairs of start and target for
coupling (`reference_pairs`) and the number of ambiguous places. That number is stored with every
snapshot (`snapshots.ambiguous_places`); the new one is the stored one minus the count over the old
references that were read plus the count over the references in memory after the scan.

The scan reads every reference in one pass instead (`StoredReferences::All`), and counts and
measures from them alone, when all names are resolved again or when the stored number is unknown
(a snapshot from before the column existed, or level 2 replaced references since).
