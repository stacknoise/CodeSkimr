# 0091 – Count the statistics once after the scan and store the numbers

Status: accepted (decision of Anton of 2026-11-23, card "Statistik beim Scan vorberechnen": "Ja, noch vor 1.0"; extends ADR 0081 and ADR 0089, changes none of their decisions)

## Context

The statistics screen (ADR 0081) ranked the symbols of a project. Every time it opened, and after every write, it loaded all symbols of the snapshot with the usage counts and measured values into memory (the "facts"), and then sorted them in the program. Only the facts of two snapshots were kept, and every scan or level 2 import dropped them. In the hand test of rc.4 (2026-11-23, Windows) a project of about 47,000 lines needed 18.5 s for the first `stats.overview` (175 ms for the second); for `microsoft/vscode` (600,000 symbols, about 3 million references) it was 1.8 minutes and about a gigabyte of memory, and switching the project started it again. The reason for the 18.5 s is the scan of the whole reference table of all projects in the facts load (issue #655, fixed separately in #668); but even with that fixed, the work grows with the size of the project and is paid again after every restart and every project switch, although the answer only changes when a scan or a level 2 import changes the data.

## Options

1. Keep the facts in memory and make loading cheaper (only the references of the project). The cost still grows with the project and is paid at every start and project switch.
2. Count the numbers inside the transaction that writes the snapshot and store them in a table that the rankings read through SQL.
3. Count them right after the scan, in a thread of their own, store them in the same table, and count them at the first question if the thread did not run.

Option 2 was built and measured first: the numbers cost 100 ms on the scan of the benchmark corpus of 100k lines (894 ms before, 998 ms after, base commit and change measured one after the other on the same machine), which is more than the tolerance of 10 % against the base (blueprint 16.2) and leaves almost nothing of the budget of 1000 ms. So the numbers are counted after the scan has been written and reported.

## Decision

Option 3.

- **Migration 0020** adds `symbol_statistics`: one row per type and callable of a snapshot with the usage counts (certain and all), the five measured values, the kind, the language, the visibility, whether it is test code or reached indirectly (constructor, override), and `place`, the position of the symbol in the order of path, qualified name and id, so that equal values come in the same order as before. Six indexes hold the order of the rankings the overview shows (most used by certain usages, most complex, most coupled, and the unused ones in the order of their place); the two rankings that are only asked for on their own tab (all usages, size) read the rows of the snapshot once. `statistics_facets` and `statistics_complexity` hold how many rows there are per group (kind, language, visibility, test code, used or not, has a measure) and per complexity, so that no count of the overview needs a pass over the rows. `snapshot_statistics` says for which test folders the rows were counted, and holds the sum of the certain and the probable references.
- **One calculation.** The rows are made by the calculation the screen used before (`load_facts`, then `mark_tests`); it is kept as `CodeStatistics::symbol_facts` and is what the tests compare the stored numbers with. The program counts them in a thread of its own after a scan from the browser has written the snapshot, and again after level 2 has replaced references (`CodeStatistics::prepare`); the scan does not wait for it and its time is unchanged. The count runs on a connection of its own and takes the write lock only to store the rows; if a write of this store or of another process came in meanwhile, it is repeated, and the second time under the write lock. A level 2 import of any process deletes the rows of the snapshots whose references it replaced, so a snapshot never keeps numbers that do not match its references. A question that comes before the thread has finished waits for the same count (one count per snapshot at a time).
- **Test code is marked for the test folders of the settings** (`conventions.test_folders`). The numbers remember the folders they were counted for; a question that comes with other names (a changed setting) counts them again once. The same holds for a snapshot that has no numbers: one written before this version, or by the command line, which does not count them. They are counted when the project is first opened, no new scan is needed.
- **The rankings are SQL.** `CodeStatistics::ranked` reads the ids of the first rows from the index of the ranking and then only those rows with their names; `unused_counts` and `summary` read the counts per group. The assessment of "no usage found" (very likely unused, public, approximate) is still decided by `Assessment::of`; the ranking of unused symbols reads the rows of each assessment in the order of their place, the very likely unused ones first.
- **Removed:** the facts kept in memory (`shared_facts`, `KeptFacts`).
- The statistics page of the browser stops the requests of a view that was replaced by a newer one, so they do not hold one of the few connections of the browser.
- The rule of ADR 0033 stays: recall before speed, only direct usages, "no usage found" never says "dead".

## Measurement

Release build, development machine (4 cores, noisy), `crates/adapters/storage/benches/statistics.rs` (the questions of one `stats.overview`: the four cards, the counts of the unused symbols, the sums; synthetic graph with 20,000 symbols and 100,000 references, and 30 times that for the size of `microsoft/vscode`):

| Case | Before (the load of the facts) | After (stored numbers) |
| --- | --- | --- |
| 100,000 references, 20,000 symbols | 22 ms per question | 1.7 ms; counting them once: 226 ms |
| 3 million references, 600,000 symbols | 1.1 s per question, 7 s the first time | 35 ms; counting them once: 10 s (in the background, after the scan) |
| Hand test rc.4, 47,000 lines, Windows | 18.5 s the first time, 175 ms after | to be measured on rc.5 |

The scan itself is not slower: `scan/full_100k_loc` measures the same as the base commit, because the count runs after it.

## Consequences

- Opening the statistics, switching the project and restarting the program cost milliseconds; no gigabyte of memory is kept.
- **Counting is work of its own.** It reads all symbols and references of the snapshot once (the cost of the former first question, now once per scan and in the background, not once per opening), then writes one row per type and callable. A very large project (VS Code) is counted for about 10 s after its scan; a question that comes earlier waits for it, which is as long as the screen waited before at the most. The command line `codeskimr scan` does not count: the numbers are counted when the project is first opened in the app.
- The database grows by about one row per type and callable (about 100 bytes) and the indexes of the six rankings.
- A snapshot that was written by another program with other test folders (the command line without the settings of the app, an older version) is counted again by the first question of the app, which takes as long as the load of the facts did, once.
- Blueprint 6.2 (read models) gets one more table of the same kind. Unlike the other read models it is not written in the transaction of the snapshot but after it, because it must not slow the scan down; a snapshot without the numbers is a valid state and is completed by the first question.
