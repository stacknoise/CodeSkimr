# 0095 – One database file per project and a small catalog database

Status: accepted (wish of Anton of 2026-11-24: "jedes Projekt hat seine eigene DB und bei einem Löschen wird einfach die DB gelöscht"; decided on the card "DB pro Projekt schon vor 1.0", which moves 1.0; reviewed in PR 723)

## Context

Blueprint 6.2 says "one SQLite database per workspace". All projects share its tables. Because rows are shared between snapshots (ADR 0032) they carry no project id, so removing a project means deleting millions of rows table by table through three or four indexes each (issues #714, #720, #722). On the machine of the product owner a large project takes more than 25 minutes, and every other write (a rename, a scan of another project) has to wait for the single writer in between. A scan of one project also grows the shared file and its indexes, which makes writes of all projects slower.

## Options

1. Keep one file and make the removal faster still (hide the project at once, delete in the background). The work stays proportional to the project and still blocks other writers.
2. One file per project plus a small catalog file. Removing a project removes its file, which does not depend on its size. A scan writes only its own file.
3. One file per snapshot. Rejected in ADR 0032: reads across snapshots, and retention as a file operation.

## Decision

Option 2.

**Files.** In the user data directory: `catalog.db` and `projects/<project id>.db` (plus the `-wal` and `-shm` files SQLite creates next to it).

**The project file** holds everything that belongs to one project, derived from its source or entered about it: `snapshots`, `files`, `symbol_rows`, `metric_rows`, `symbol_references`, `fingerprints`, `pending_references`, `coupling_rows`, the read models (`search_index`, `usage_counts`, `type_summary`, `layer_graph`), the statistics tables, `parse_times`, `snapshot_warnings`, `snapshot_left_out`, `snapshot_skipped_files`, and the annotations of the project: `annotation`, `annotation_baseline` and `alert` (ADR 0068; decision of Anton, 2026-11-24: notes and alerts live in the database of their project, so that removing the file removes everything of the project). A note finds its symbol by kind, qualified name and parameter types, never by an id, as before. Its schema is the current schema without the `projects` table; it has its own `user_version` and migrations.

**The catalog file** holds what belongs to the user and to the list of projects: `projects` (including `removing`), `layers` with their uses, `level2_consent`, `tool_reports` and `tool_findings`, `scan_reports` and the level 2 reports, and the registered credentials' names. It stays small (kilobytes to megabytes) and is the only file every start opens.

**Ids.** Snapshot, file and symbol ids stay unique across projects, so a bare `SnapshotId` or `SymbolId` of a port or a URL still names one place and the ports do not change. An id is the project number in the high 20 bits and a per-file `IdSequence` in the low 33 bits, so every id is below 2^53 and survives JavaScript numbers in the browser unchanged (one helper in `crates/adapters/storage` builds and splits ids; a test builds the highest id and checks that it is below 2^53). A project number never repeats (catalog `AUTOINCREMENT`). The numbers run to 1,048,575; the next registration after that fails with a coded error instead of wrapping. A file counts at most 8.5 billion ids (`ID_STEP` included); a scan that would exceed this stops with `STO-601` before it writes. A project number never names two rows, as ADR 0032 requires.

**Routing.** `SqliteStore` becomes the router `StoreSet` that implements the same ports. It splits the project id from the id of a call, keeps one open `ProjectFile` (today's `SqliteStore` with its connections, caches and read models) per project that was used lately, and opens a file on first use. The catalog is a second small store. Cross-project functions (project list, overview, `codeskimr` MCP over several projects, the statistics of the home screen) read the catalog first and then the head of each project file they need; nothing queries two files in one statement and no `ATTACH` is used.

**Removing a project.** One call: mark the project in the catalog (`removing`, as today), close its connections and drop its caches, delete the file with its `-wal` and `-shm` files, delete its rows in the catalog in one short write. On Windows an open file cannot be deleted, so the router waits for running reads and the writer of that project, refuses new ones with `PRJ-201`, and retries the deletion for a short time; a file that still cannot be deleted stays marked `removing` and is removed at the next start. The slice-wise removal of #714 and #720 is deleted with its constants. A marked project without a file is a finished removal.

**Showing where the data is.** The project list and the project screen show, for every project, the path of its file and its size (file plus WAL), in the UI, in `codeskimr projects` and in `projects.list`, which is the local REST API of the UI and not an export; the MCP tools and exports do not carry the path. The API field is `database: { path, size_bytes, state }`, with `state` one of `ok`, `missing` (catalog row without a file) and `removing`. The size is read from the file system and is not stored. A project in the middle of removal shows as such.

**No migration.** There are no installations yet (decision of Anton, 2026-11-24). The first build with this ADR does not read an existing shared database: the file is ignored, and a program that finds the old `codeskimr.db` says once, in the log, that it is no longer used and can be deleted. Projects are registered and scanned again. The old migrations `0001`..`0022` are replaced by one schema file per database (`catalog/0001_initial.sql`, `project/0001_initial.sql`), which also drops the shared-database workarounds (`projects.removing` slices, the project id columns of the project tables).

**Concurrency.** Every file has its own single writer. A scan of project A no longer blocks a rename or a scan of project B; the busy timeout and the lock-order rules apply per file, and the catalog is written only in short, small transactions (never during a scan's long write).

**Order of operations.** create = catalog row first, then the file; scan = file first (snapshot, alerts), then the report entries of the catalog in a short write after it; remove = mark in catalog, delete file, delete catalog rows. A note is created in one transaction of the project file (symbol check, note), so notes and alerts never span two files. A failure between two steps leaves a state that the start repairs: a catalog row without a file is a project without a snapshot, a marked project is removed again.

**Orphans.** A project file without a catalog row is only listed in the log at the start. It is deleted only if all of these hold: the catalog opened and is intact (`PRAGMA integrity_check` quick check and its identity row), the file carries the identity of this installation (an `application_id` and an installation id in the file header table that match the catalog), and its project number is not above the highest number the catalog has ever handed out (the catalog keeps that counter in its own table, not from the rows). Anything else is left alone, so a missing, older or replaced catalog never causes project files to be deleted.

## Consequences

- Removing a project takes the time of a file deletion, independent of its size.
- Scans and removals of different projects run in parallel without a shared writer.
- Blueprint 6.2 changes ("one database per workspace" becomes this decision) and `docs/storage.md` is rewritten for the two schemas; label `blueprint-change`.
- A project file holds the user's notes, so it is no longer disposable: throwing it away loses them, and backing up a project means copying its file (with the WAL file, or after the program closed it). The catalog holds the project list, layers, consents and reports; it too is user input and cannot be rebuilt from the project files.
- Foreign keys across the two files do not exist; the router keeps them by the order of operations in the decision above.
- Annotations and alerts live in the project file; a re-scan does not touch them (ADR 0068) and removing the project removes them with the file.
- Total disk use can be a little higher (each file has its own free pages); the retention count applies per project as before.
- The number of open files is bounded: the router keeps at most `store.open_project_files` project files open (a setting in `config/default.toml`, least recently used closed first, never one with a running write or scan). The path of a database file is shown in the UI and the CLI but is not part of exports or MCP answers.
- The work is large: store split, router, all callers that open `SqliteStore`, tests, UI display. The packages are described in the pull requests that follow this ADR.
