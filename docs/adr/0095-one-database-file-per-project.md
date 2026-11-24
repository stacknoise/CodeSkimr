# 0095 – One database file per project and a small catalog database

Status: proposed (wish of Anton of 2026-11-24: "jedes Projekt hat seine eigene DB und bei einem Löschen wird einfach die DB gelöscht"; decided on the card "DB pro Projekt schon vor 1.0", which moves 1.0)

## Context

Blueprint 6.2 says "one SQLite database per workspace". All projects share its tables. Because rows are shared between snapshots (ADR 0032) they carry no project id, so removing a project means deleting millions of rows table by table through three or four indexes each (issues #714, #720, #722). On the machine of the product owner a large project takes more than 25 minutes, and every other write (a rename, a scan of another project) has to wait for the single writer in between. A scan of one project also grows the shared file and its indexes, which makes writes of all projects slower.

## Options

1. Keep one file and make the removal faster still (hide the project at once, delete in the background). The work stays proportional to the project and still blocks other writers.
2. One file per project plus a small catalog file. Removing a project removes its file, which does not depend on its size. A scan writes only its own file.
3. One file per snapshot. Rejected in ADR 0032: reads across snapshots, and retention as a file operation.

## Decision

Option 2.

**Files.** In the user data directory: `catalog.db` and `projects/<project id>.db` (plus the `-wal` and `-shm` files SQLite creates next to it).

**The project file** holds everything that is derived from the source of one project: `snapshots`, `files`, `symbol_rows`, `metric_rows`, `symbol_references`, `fingerprints`, `pending_references`, `coupling_rows`, the read models (`search_index`, `usage_counts`, `type_summary`, `layer_graph`), the statistics tables, `parse_times`, `snapshot_warnings`, `snapshot_left_out`, `snapshot_skipped_files`. Its schema is the current schema without the `projects` table; it has its own `user_version` and migrations.

**The catalog file** holds what belongs to the user and to the list of projects: `projects` (including `removing`), `layers` with their uses, `level2_consent`, `tool_reports` and `tool_findings`, `scan_reports` and the level 2 reports, `annotation`, `annotation_baseline` and `alert` (ADR 0068), and the registered credentials' names. It stays small (kilobytes to megabytes) and is the only file every start opens.

**Ids.** Snapshot, file and symbol ids stay unique across projects, so a bare `SnapshotId` or `SymbolId` of a port or a URL still names one place and the ports do not change. An id is `project id` in the high 24 bits and a per-file `IdSequence` in the low 39 bits (`crates/adapters/storage`: one helper builds and splits ids). A project id never repeats (catalog `AUTOINCREMENT`), so an id never names two rows, as ADR 0032 requires.

**Routing.** `SqliteStore` becomes the router `StoreSet` that implements the same ports. It splits the project id from the id of a call, keeps one open `ProjectFile` (today's `SqliteStore` with its connections, caches and read models) per project that was used lately, and opens a file on first use. The catalog is a second small store. Cross-project functions (project list, overview, `codeskimr` MCP over several projects, the statistics of the home screen) read the catalog first and then the head of each project file they need; nothing queries two files in one statement and no `ATTACH` is used.

**Removing a project.** One call: mark the project in the catalog (`removing`, as today), close its connections and drop its caches, delete the file with its `-wal` and `-shm` files, delete its rows in the catalog in one short write. On Windows an open file cannot be deleted, so the router waits for running reads and the writer of that project, refuses new ones with `PRJ-201`, and retries the deletion for a short time; a file that still cannot be deleted stays marked `removing` and is removed at the next start. The slice-wise removal of #714 and #720 is deleted with its constants. A marked project without a file is a finished removal.

**Showing where the data is.** The project list and the project screen show, for every project, the path of its file and its size (file plus WAL), in the UI, in `codeskimr projects` and in `projects.list`. The size is read from the file system and is not stored. A project in the middle of removal shows as such.

**No migration.** There are no installations yet (decision of Anton, 2026-11-24). The first build with this ADR does not read an existing shared database: the file is ignored, and a program that finds the old `codeskimr.db` says once, in the log, that it is no longer used and can be deleted. Projects are registered and scanned again. The old migrations `0001`..`0022` are replaced by one schema file per database (`catalog/0001_initial.sql`, `project/0001_initial.sql`), which also drops the shared-database workarounds (`projects.removing` slices, the project id columns of the project tables).

**Concurrency.** Every file has its own single writer. A scan of project A no longer blocks a rename or a scan of project B; the busy timeout and the lock-order rules apply per file, and the catalog is written only in short, small transactions (never during a scan's long write).

## Consequences

- Removing a project takes the time of a file deletion, independent of its size.
- Scans and removals of different projects run in parallel without a shared writer.
- Blueprint 6.2 changes ("one database per workspace" becomes this decision) and `docs/storage.md` is rewritten for the two schemas; label `blueprint-change`.
- Backing up or moving one project is copying one file. The catalog holds nothing derived from source and could in principle be rebuilt from the project list.
- Foreign keys across the two files do not exist; the router keeps them by order of operations (create: catalog row first, then file; remove: see above). A project file without a catalog row is an orphan; the start lists and deletes them. A catalog row without a file is a project without a snapshot.
- Annotations and alerts live in the catalog, so they survive a re-scan and are removed with the project by `project_id`.
- Total disk use can be a little higher (each file has its own free pages); the retention count applies per project as before.
- The work is large: store split, router, all callers that open `SqliteStore`, tests, UI display. The packages are described in the pull requests that follow this ADR.
