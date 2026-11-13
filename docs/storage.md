# Storage adapter (`codeskimr_storage`)

SQLite database per workspace, WAL mode, schema migrations in `crates/adapters/storage/migrations/`
(applied at start-up, version in `PRAGMA user_version`). Blueprint 6.2.

## What it does

- Schema version 2 adds `projects.credential`: the name of the credential a Git source logs in with (empty for none). Only the name is stored, never a secret (blueprint 12.2).
- Schema version 3 adds `fingerprints` (`snapshot_id`, `symbol_id`, `position`, `hash`): the syntax fingerprints of the callables (blueprint 10.4). A hash is the 64 bits stored as a signed integer; an index on `(snapshot_id, hash)` serves the similarity query. Databases written before version 3 have no fingerprints until their next full scan.
- Schema version 4 (ADR 0032) stores symbols (`symbol_rows`), metrics (`metric_rows`), references (`symbol_references`) and fingerprints once, without a snapshot id, owned by the file or symbol they belong to. The views `symbols`, `metrics` and `snapshot_references` give every snapshot the rows of its files, with the columns of the former tables. An incremental scan copies only the `files` rows; the prune deletes the shared rows of files that no other snapshot contains. A reference whose ends are not in the snapshot is refused with STO-901.
- `SqliteStore` implements the ports `ProjectRepository`, `SnapshotStore`, `CodeIndex`, `UsageIndex` and `SimilarityIndex` (read access for the MCP server: `latest_snapshot`, `search`, `symbol`, `layer_edges`).
- `persist` writes one snapshot and its read models in one `BEGIN IMMEDIATE` transaction, then prunes to the
  retention count. Any failure rolls everything back (STO-302 busy/locked, STO-501 disk full, STO-601 other).
- Incremental scans (`ScanMode::Incremental`): files that were not analysed again are copied from the
  previous snapshot with their ids, except the paths in `SnapshotData::removed`; their symbols, metrics and
  references (whose two ends survive) are seen through the shared rows without a copy. Metrics in the new data
  replace stored ones of the same symbol and kind. The returned stats are the totals of the whole snapshot. `latest_index` returns all symbols
  and references of the newest snapshot, the input of `docs/incremental-scan.md`.
- Read models (rebuilt per snapshot, inside the same transaction): `search_index`, `usage_counts`,
  `type_summary`, `layer_graph`. Queries: `search`, `usage_count`, `type_summary`, `layer_edges`, `symbol_detail`
  (one symbol by id or qualified name with its members and metrics).
- `UsageIndex` (usages and impact of a symbol, `docs/usages.md`): the references of a snapshot are kept as a `ReverseIndex` in memory for the snapshot that was asked about last.
- `SimilarityIndex` (`fingerprints_of`, `similar`): `similar` takes the hashes of the query, counts per stored function how many distinct hashes it shares, and completes the union with the number of distinct hashes of those functions. The percent is the Jaccard value of `Fingerprints::similarity`, rounded down, so it equals what the duplicate check of `docs/conventions.md` computes. The statements take their lists as one JSON text (`json_each`), so the number of hashes is not limited by the number of SQL parameters. A hash that very many functions share makes the count query slow in proportion; capping those is issue #99.
- Outside the ports: `save_project` (registers a project, PRJ-301 for a duplicate source),
  `latest_snapshot`, `next_free_id` (start value for `IdSequence`).

## Known limits

- Search is case-insensitive for ASCII only (SQLite `lower`); the query is lowered the same way.
- The project's layers come from the configuration and are saved with the project at every scan (ADR 0031).
- `layer_graph.violates_layering` is always false until the project model carries the allowed directions.
- Deleted files are reported by the pipeline (`SnapshotData::removed`); the store trusts that list.
