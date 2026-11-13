# Reverse index and usages (WP-31)

Blueprint 9. The reverse index answers three questions for one symbol: who uses it (`incoming`), what it uses (`outgoing`) and what a change of it can reach (impact).

## Where it lives

- `codeskimr_domain::ReverseIndex` holds the references of one snapshot, indexed by both ends. It knows nothing about databases and has no dependency. `usages(query)` walks the references breadth first; `reached(query)` lists the symbols the same walk reaches.
- `codeskimr_application::ports::UsageIndex` is the port with `usages` and `impact`. The answers carry qualified names, kinds, file paths, confidence and the step of the walk, and are cut to a `limit` with the full `total` next to them.
- `codeskimr_storage::SqliteStore` implements the port. It loads the references of a snapshot into a `ReverseIndex` on first use and keeps the index of the snapshot that was asked about last. A snapshot never changes (a scan writes a new one), so the cache needs no invalidation. The names of all symbols and the paths of all files of that snapshot are loaded with it, so the entries of an answer are put together in memory and not looked up one by one in the database (#165: that lookup took 17 to 76 ms for symbols with over 100 usages).

The `usage_counts` read model (the "Usages" column) stays as it is; `ReverseIndex::usage_count` gives the same number from memory.

## Rules of the walk

- Step 1 lists the references of the start symbol, step 2 those of the symbols reached in step 1, and so on up to `depth`. A depth of zero finds nothing.
- A reference is listed once. A symbol that was reached before is not walked again, so cycles end; the reference that closes a cycle is still listed.
- Within one symbol the references are ordered by file, position and id; the answer is the same for the same snapshot.
- References of the kind `test` count only if `include_tests` is set. Impact always includes them.
- A reference carries `verified` (true only if level 2 produced it; every level-1 reference is unchecked, ADR 0033). A reached symbol of `impact` is verified only if a path of verified references leads to it in its nearest step. UI and MCP count verified and unchecked results apart and list verified ones first.
- The walk follows references of any confidence. Every entry carries its `confidence`; UI and MCP must show `probable` entries as such (blueprint 9.4).
- References end at the symbol that contains the occurrence. The usages of a type do not include the usages of its members; ask for the members separately.
- A symbol that does not exist has no usages. Ask `CodeIndex::symbol` first to tell "unknown symbol" from "no usages".

## Budget

`tools/bench/benches/usages.rs` measures depth 1 and 2 on 100 000 references and 20 000 symbols with such a hub; a hub symbol that every 20th reference points at (about 5 000 direct users) takes about 2.4 ms at depth 2 on the development machine (depth 1: about 0.2 ms), against the budget `usages_query_ms` (20 ms, blueprint 16.1). `crates/adapters/storage/benches/usages_page.rs` measures what the command asks of the store (both walks, the direct usages and the impact, 500 entries each, names and files included) for the same hub: about 0.8 ms, against 51 ms when every entry was looked up in the database. It is checked against the same budget.

## Not part of this package

- The MCP tools `find_usages` and `impact_of_change` use this port (WP-36, `docs/mcp.md`) and clamp `depth` and `limit` to the configured maximum (`mcp.max_depth`, `mcp.max_limit`). The REST route and the UI screen follow in their own packages.
- Grouping the impact by layer and marking public interfaces: the answer carries the qualified name and the visibility of each symbol, the project's layers decide the rest.
- Blueprint 9.4 ("updated incrementally together with the snapshot"): the index is derived from the references of each new snapshot, which the store already writes incrementally, so it is always in step with the snapshot. A snapshot that is not stored (never existed, or pruned) answers empty. Re-resolving dependent references of an incremental scan through this index (blueprint 7, issue #49) is unchanged.
