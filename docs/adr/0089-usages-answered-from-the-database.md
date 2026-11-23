# 0089 – Answer usages from the database

Status: accepted (decision of Anton of 2026-11-23, card "Noch vor 1.0": the usages index is no longer built in memory when the program starts; questions are answered by SQL through an index of the database)

## Context

Since ADR 0033 and WP-31 the references of a snapshot were loaded into a `ReverseIndex` in memory the first time a question about usages, impact, namespaces, types or statistics came in, and the app built it in the background at every program start and every time a project was opened (#333). That is fast per question (1.6 ms) and costs the load: 200 to 250 ms and a few hundred MB for 100,000 references, and for `microsoft/vscode` (600,000 symbols, millions of references) minutes and gigabytes at every start of the program, with the explorer, the diagram and the statistics waiting behind it (rc.3 hand test of 2026-11-23: "not usable"; rc.2 behaved the same on that project). The rows are in a database that already has indexes on both ends of a reference, so most of what the index was built for can be asked of the database.

## Options

1. Keep the index and make it cheaper (smaller rows, build on a second thread). The cost still grows with the project and is paid at every start.
2. Answer the questions from the database, step by step, with no index in memory. The cost grows with what a question touches; a large project costs nothing until someone asks.
3. Option 2 plus read models for the totals (counts of references per symbol with and without tests), so that the last step of a deep walk needs no rows at all.

## Decision

Option 2 now, option 3 where measurement asks for it (see Consequences).

- **No new index.** `references_by_target (to_symbol)` and `references_by_source (from_symbol)` exist since migration 0004 and are used by every statement below. A new index on the same columns would only cost disk space; so there is no migration in this change.
- **Usages and impact** (`UsageIndex::usages`, `impact`): a walk reads the references of its frontier with one statement per step (`json_each` of the symbol ids against the index on the target or the source) and walks the rows it got with the existing `ReverseIndex`. The frontier of the next step is the set of symbols reached for the first time, as in the in-memory walk, so the result (order, depth, `verified`, total) is the same. Statements go through the `symbol_references` table and look up the file of every row in the snapshot; a row of another snapshot is dropped as before. The walk into a symbol first checks that the snapshot holds it, so that references into a symbol of another snapshot do not count; the walk out of a symbol checks the target of each row too.
- **Entries** (names, kinds, files, visibility) are read for the entries that are shown, in one statement each, not for the whole snapshot.
- **Statistics** (ADR 0081): the usage counts, the files that hold a test reference and the symbols that override a member are counted in one pass over the references of the snapshot, without keeping them. The facts per snapshot that #661 shares between the statistics tabs stay; they are kept for the data version they were counted at and dropped after every write of this store.
- **Namespace graph and type focus**: the graph is counted in one pass over the certain references and kept (it is small). The type focus reads the symbols once and only the references that touch a member of the type (`type_members` in the domain).
- **Removed:** the usages index built at program start and when a project is opened (`warm_newest`, `warm`, the `usages-warm-up` and `index-warm-up` threads, the warming outline wrapper), the kept lookups (`KEPT_LOOKUPS`, `SnapshotLookup`, `refresh_lookup`) and the build gate of the index. The gate per snapshot stays for the two builds that remain (facts, namespace graph); a write counter keeps a build that was running during a write from keeping what it read before it.
- The rule of ADR 0033 stays: recall before speed, direct usages only.

## Measurement

Release-profile benchmark `usages_page/depth_2_hub` (the four reads of one `usages.query`: both walks, the direct usages, the impact; 500 entries; names and files included), development machine, 4 cores:

| Case | In memory (before, index already built) | From the database (after) |
| --- | --- | --- |
| 100,000 references, 20,000 symbols, hub with 5,000 direct users | 0.8 ms (plus 256 ms to build, at every start) | 32 ms |
| 3 million references, 600,000 symbols, symbol with a few hundred users | not measurable: the build takes minutes and gigabytes | 19 ms |
| 3 million references, 600,000 symbols, hub with 150,000 direct users | same | 1.2 s |

The first query needs no build, and the program holds no index in memory.

## Consequences

- The program starts and opens a project without reading the references. The first query of a large project costs what the query touches, and the memory of a query is what it reads.
- **The budget `usages_query_ms` (20 ms) is kept for ordinary symbols and exceeded for a hub at depth 2:** the hub of the benchmark (every 20th reference points at one symbol, so its depth-2 walk touches 7,000 rows and the four reads about 25,000) takes 32 ms against 20 ms, and a hub of 150,000 direct users takes 1.2 s. The bench `usages_page/depth_2_hub` therefore reports the budget exceeded. The budget is not loosened here (CLAUDE.md, rule 5); the decision on it is Anton's. The way back below 20 ms is option 3: counts per symbol and per kind as a read model, so that the last step of a walk only needs the rows that are shown. It needs a migration that fills the counts for existing databases and is not part of this change.
- The statistics read all references once per snapshot and write (a few seconds for a project of the size of VS Code, on a connection of its own, kept as facts afterwards). The namespace graph and the type focus read the symbols of the snapshot (once per graph; once per click for the type focus). The measurement of these on VS Code belongs to the hand test of the release candidate.
- A level 2 import or a scan of another process changes the data version; kept facts and graphs are dropped at the next question, as the kept index was.
