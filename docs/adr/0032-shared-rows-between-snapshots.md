# 0032 – Symbols, metrics, references and fingerprints are stored once and shared by snapshots

Status: proposed

## Context

Blueprint 16.1 asks for an incremental rescan under 300 ms. Every snapshot held its own copy of all rows, so a rescan after one saved file copied the symbols, metrics, references and fingerprints of the whole project into the new snapshot (issue #108). On 100k lines of code the copy took more than half of the rescan, and it grows with the project, not with the change. Pruning the oldest snapshot had to delete the same amount again.

## Options

1. Keep the copies and make them faster (batching, fewer indexes). The cost stays proportional to the project.
2. Delta snapshots: a row is written once and owned by the file (symbol, reference) or symbol (metric, fingerprint) it belongs to. A snapshot sees the rows of the files it contains.
3. A second database file per snapshot. Reads across snapshots would need `ATTACH` and the retention would be a file operation.

## Decision

Option 2, chosen by the product owner on 2026-11-13.

- Schema version 4 stores `symbol_rows`, `metric_rows`, `symbol_references` and `fingerprints` without a snapshot id. Ids are never reused (`IdSequence`), so an id names exactly one row for ever.
- The views `symbols`, `metrics` and `snapshot_references` have the columns of the former tables. A snapshot contains the symbols and metrics of its files, and the references whose file and target symbol are in it. All read queries keep their form. Rebuilds of read models and the prune use the tables directly.
- An incremental scan copies only the `files` rows of the carried files. Metrics for an existing symbol replace the stored value; metrics depend on the file content alone, so older snapshots see the same value.
- The prune deletes the shared rows of the files that no other snapshot contains, then the snapshot's own rows, table by table.
- A reference remembers the snapshot that wrote it (`first_snapshot`); a snapshot older than that never sees it. A reference that a later scan rebuilds for a carried file therefore appears only in the snapshots from that scan on, as with the per-snapshot copies. The migration takes the oldest snapshot that held a reference.
- The foreign keys are replaced by a check when a snapshot is written (`STO-901`, rolls the snapshot back): the file and parent of every new symbol, the file and both ends of every new reference, and the symbol of every new metric and fingerprint must be in the snapshot.
- The migration copies the rows of all snapshots with `INSERT OR IGNORE` (the copies carry identical ids) and drops the old tables.

## Condition: metrics depend on one file

A metric row is shared and a new value replaces the stored one, so older snapshots see it too. This holds as long as a metric depends on the content of its own file, which is true for all metrics `StandardMetrics` computes (loc, complexity, maintainability). The cross-file metrics of `MetricKind` (efferent and afferent coupling, instability) depend on references; whoever computes them has to store them per snapshot or change this decision first (issue #150).

## Consequences

- A rescan writes what changed and the read models; measured on the benchmark corpus (100k lines, one changed file) the rescan takes about 175 ms on a 4-vCPU machine, where the report of issue #108 measured 0.8 to 2 s.
- Foreign keys no longer protect these four tables; the checks above and the tests of the prune (no rows left behind) take over.
- Deleting a project through the foreign key cascade (`projects` to `snapshots` to `files`) does not remove the shared rows. Storage has no such function today; a future one has to run `SHARED_ROWS_DELETE` for every snapshot first.
- A program of an older version does not open a version 4 database (`STO-301`); there is no way back to version 3 other than a new scan.
- The read models (`search_index`, `usage_counts`, `type_summary`, `layer_graph`) are still rebuilt per snapshot; making them incremental is a possible next step.
