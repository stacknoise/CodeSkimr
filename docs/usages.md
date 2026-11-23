# Reverse index and usages (WP-31)

Blueprint 9. The reverse index answers three questions for one symbol: who uses it (`incoming`), what it uses (`outgoing`) and what a change of it can reach (impact).

## Where it lives

- `codeskimr_domain::ReverseIndex` holds the references of one snapshot, indexed by both ends. It knows nothing about databases and has no dependency. `usages(query)` walks the references breadth first; `reached(query)` lists the symbols the same walk reaches.
- `codeskimr_application::ports::UsageIndex` is the port with `usages` and `impact`. The answers carry qualified names, kinds, file paths, confidence and the step of the walk, and are cut to a `limit` with the full `total` next to them.
- `codeskimr_storage::SqliteStore` implements the port and answers from the database (ADR 0089). Nothing is built when the program starts or a project is opened: a walk reads the references of the symbols it reaches with one statement per step, through the index on the target (`references_by_target`) or on the source of a reference (`references_by_source`), and walks the few rows that came back with the `ReverseIndex`; the result is the one a walk over the whole snapshot gives. Only the labels and files of the entries that are shown are read, in one statement each. The memory of a query is what it reads. The earlier design loaded all references of a snapshot into memory on first use (about 250 ms and a few hundred MB for 100,000 references, minutes and gigabytes for `microsoft/vscode`) and kept it.
- The usage counts of the statistics (ADR 0081) are counted from the references in one pass over the table and kept as facts for the snapshot, not as an index; the namespace graph is built in one pass too and only the small graph is kept. The type focus reads the references that touch the members of the type. A kept result is dropped when another connection has committed since (`PRAGMA data_version`) and after every write of this store.

The `usage_counts` read model (the "Usages" column) leaves out references of the kind `test`, as the usages list does by default and as coupling and the layer graph do (issue #430); `ReverseIndex::usage_count` gives the same number for an index in memory. The list shows the `test` references again with `include_tests`.

## Rules of the walk

- Step 1 lists the references of the start symbol, step 2 those of the symbols reached in step 1, and so on up to `depth`. A depth of zero finds nothing.
- A reference is listed once. A symbol that was reached before is not walked again, so cycles end; the reference that closes a cycle is still listed.
- Within one symbol the references are ordered by file, position and id; the answer is the same for the same snapshot.
- References of the kind `test` count only if `include_tests` is set. Impact always includes them.
- A reference carries `verified` (true only if level 2 produced it; every level-1 reference is unchecked, ADR 0033). A reached symbol of `impact` is verified only if a path of verified references leads to it in its nearest step. UI and MCP count verified and unchecked results apart and list verified ones first, then nearest step first, then in the order of the walk (file and position). `ReverseIndex::usages` and `reached` return this order, so the cut by `limit` keeps the verified ones (#202); the UI and MCP take the order as it comes.
- The walk follows references of any confidence. Every entry carries its `confidence`; UI and MCP must show `probable` entries as such (blueprint 9.4).
- References end at the symbol that contains the occurrence. The usages of a type do not include the usages of its members; ask for the members separately.
- A symbol that does not exist has no usages. Ask `CodeIndex::symbol` first to tell "unknown symbol" from "no usages".

## Budget

`tools/bench/benches/usages.rs` measures the walk of the `ReverseIndex` alone (depth 2 on a hub symbol with about 5 000 direct users: about 2.4 ms). `crates/adapters/storage/benches/usages_page.rs` measures what the command asks of the store from the database (both walks, the direct usages and the impact, 500 entries each, names and files included) for the same hub, with 100 000 references: about 32 ms (ADR 0089 gives the measurement and what that means against the budget `usages_query_ms`, 20 ms, blueprint 16.1). With the size of `microsoft/vscode` (600 000 symbols, 3 million references) the same four reads cost about 19 ms for a symbol with a few hundred users, and about 1.2 s for a hub that 5 % of all references point at.

## Not part of this package

- The MCP tools `find_usages` and `impact_of_change` use this port (WP-36, `docs/mcp.md`) and clamp `depth` and `limit` to the configured maximum (`mcp.max_depth`, `mcp.max_limit`). The REST route and the UI screen follow in their own packages.
- Grouping the impact by layer and marking public interfaces: the answer carries the qualified name and the visibility of each symbol, the project's layers decide the rest.
- Blueprint 9.4 ("updated incrementally together with the snapshot"): the index is derived from the references of each new snapshot, which the store already writes incrementally, so it is always in step with the snapshot. A snapshot that is not stored (never existed, or pruned) answers empty. Re-resolving dependent references of an incremental scan (blueprint 7, issue #49) does not use this index: the store keeps the names as written per file and the scan resolves them again, see `docs/incremental-scan.md`.
