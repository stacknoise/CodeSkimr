# Reverse index and usages (WP-31)

Blueprint 9. The reverse index answers three questions for one symbol: who uses it (`incoming`), what it uses (`outgoing`) and what a change of it can reach (impact).

## Where it lives

- `codeskimr_domain::ReverseIndex` holds the references of one snapshot, indexed by both ends. It knows nothing about databases and has no dependency. `usages(query)` walks the references breadth first; `reached(query)` lists the symbols the same walk reaches.
- `codeskimr_application::ports::UsageIndex` is the port with `usages` and `impact`. The answers carry qualified names, kinds, file paths, confidence and the step of the walk, and are cut to a `limit` with the full `total` next to them.
- `codeskimr_storage::SqliteStore` implements the port. It loads the references of a snapshot into a `ReverseIndex` on first use and keeps the index of the snapshot that was asked about last. A snapshot never changes (a scan writes a new one), so the cache needs no invalidation. Only the entries of the answer are looked up in the database.

The `usage_counts` read model (the "Usages" column) stays as it is; `ReverseIndex::usage_count` gives the same number from memory.

## Rules of the walk

- Step 1 lists the references of the start symbol, step 2 those of the symbols reached in step 1, and so on up to `depth`. A depth of zero finds nothing.
- A reference is listed once. A symbol that was reached before is not walked again, so cycles end; the reference that closes a cycle is still listed.
- Within one symbol the references are ordered by file, position and id; the answer is the same for the same snapshot.
- References of the kind `test` count only if `include_tests` is set. Impact always includes them.
- The walk follows references of any confidence. Every entry carries its `confidence`; UI and MCP must show `probable` entries as such (blueprint 9.4).
- References end at the symbol that contains the occurrence. The usages of a type do not include the usages of its members; ask for the members separately.
- A symbol that does not exist has no usages. Ask `CodeIndex::symbol` first to tell "unknown symbol" from "no usages".

## Budget

`tools/bench/benches/usages.rs` measures depth 1 and 2 on 100 000 references and 20 000 symbols with a few heavily used symbols; depth 2 takes some microseconds against the budget `usages_query_ms` (20 ms, blueprint 16.1). The lookup of the entries (at most `limit` rows) comes on top.

## Not part of this package

- The MCP tools `find_usages` and `impact_of_change`, the REST route and the UI screen use this port; they follow in their own packages (clamping of `depth` and `limit` to the configured maximum belongs there).
- Grouping the impact by layer and marking public interfaces: the answer carries the qualified name and the visibility of each symbol, the project's layers decide the rest.
- Re-resolving dependent references of an incremental scan through this index (issue #49) is unchanged.
