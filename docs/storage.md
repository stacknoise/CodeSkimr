# Storage adapter (`codeskimr_storage`)

SQLite database per workspace, WAL mode, schema migrations in `crates/adapters/storage/migrations/`
(applied at start-up, version in `PRAGMA user_version`). Blueprint 6.2.

## What it does

- Schema version 2 adds `projects.credential`: the name of the credential a Git source logs in with (empty for none). Only the name is stored, never a secret (blueprint 12.2).
- `SqliteStore` implements the ports `ProjectRepository`, `SnapshotStore` and `CodeIndex` (read access for the MCP server: `latest_snapshot`, `search`, `symbol`, `layer_edges`).
- `persist` writes one snapshot and its read models in one `BEGIN IMMEDIATE` transaction, then prunes to the
  retention count. Any failure rolls everything back (STO-302 busy/locked, STO-501 disk full, STO-601 other).
- Incremental scans (`ScanMode::Incremental`): files that were not analysed again are copied from the
  previous snapshot with their ids, except the paths in `SnapshotData::removed`; their symbols, metrics and
  references (whose two ends survive) come along. Metrics in the new data replace carried ones of the same
  symbol and kind. The returned stats are the totals of the whole snapshot. `latest_index` returns all symbols
  and references of the newest snapshot, the input of `docs/incremental-scan.md`.
- Read models (rebuilt per snapshot, inside the same transaction): `search_index`, `usage_counts`,
  `type_summary`, `layer_graph`. Queries: `search`, `usage_count`, `type_summary`, `layer_edges`, `symbol_detail`
  (one symbol by id or qualified name with its members and metrics).
- Outside the ports: `save_project` (registers a project, PRJ-301 for a duplicate source),
  `latest_snapshot`, `next_free_id` (start value for `IdSequence`).

## Known limits

- Search is case-insensitive for ASCII only (SQLite `lower`); the query is lowered the same way.
- `layer_graph.violates_layering` is always false until the project model carries the allowed directions.
- Deleted files are reported by the pipeline (`SnapshotData::removed`); the store trusts that list.
