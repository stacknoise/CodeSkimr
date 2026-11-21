# 0086 – An incremental scan reads only the references it needs

Status: accepted

## Context

An incremental scan starts from the newest snapshot (blueprint section 7). Until now it read **every** reference of that snapshot into memory, although it analyses only the changed files. On Newtonsoft.Json (138k LOC, 418k references) a re-scan after saving one file takes about 0.84 s against a budget of 300 ms (blueprint 16.1, issue #410). About 0.3 s of it is spent on the references of files that the scan does not touch:

| Step | Time |
|---|---|
| read all references from the store | 181 ms |
| split them into kept and gone | 29 ms |
| compare them with the newly resolved ones | 83 ms |
| count the ambiguous places (`RES-301`) | 12 ms |
| coupling over all references | about 15 ms |

Reading is the expensive part and cannot be made much cheaper per row: SQLite alone needs 63 ms for the rows, the rest is the column access. The way out is to read fewer rows.

The scan needs the old references for four things:

1. **Gone targets.** References from an unchanged file to a symbol of a changed or removed file lose their target; their names are resolved again (ADR 0033 decision 13 for the verified ones).
2. **Comparison.** The names that are resolved again produce references; those that equal a kept one are not written twice, kept ones that are no longer produced are dropped (issue #49, issue #408).
3. **Coupling.** Ce, Ca and instability depend on the references of the whole snapshot (ADR 0037, ADR 0074).
4. **Warnings.** The number of ambiguous places counts over the whole snapshot.

The product owner released this work on 2026-11-21. Recall goes before speed: the result of an incremental scan must stay the one a full scan gives.

## Decision

### Touched symbols

A symbol of an unchanged file is **touched** by an incremental scan if a name written in it is resolved again or one of its references points at a symbol that is gone. In every other symbol of an unchanged file the scan neither adds, drops nor changes a reference. This is the rule the scan already follows place by place (`Carried::reconcile` keeps what is out of scope as it is); the decision draws the line by the symbol a reference starts in, because the store has an index on it.

The line cannot be drawn by file: the base lists of all types are resolved again in every scan, so nearly every file holds a name that is resolved again.

The scan reads from the store:

- the references that point at a symbol of a changed or removed file (index on the target),
- the files that hold a reference verified by level 2 (new partial index),
- the references that start in a touched symbol, once the names to resolve again are known (index on the start),
- the references of the previous state of the changed and removed files, for the warning count below.

All other references are never read as full rows.

### Coupling

Coupling is still computed over the whole snapshot, never updated in place, because the unit of a symbol depends on the declarations of all files. For the references that are not read, the store returns only the distinct pairs *(from symbol, to symbol)* of those that are not of kind `test`. The pairs of the references in memory come from memory. `ce` and `ca` count each pair of units once, so pairs give the same values as the references.

### Ambiguous places

A place is a file and a span, and it lies in exactly one symbol, so the count is a sum over symbols. A snapshot stores its count in the new column `snapshots.ambiguous_places`. The next incremental scan computes

`new count = stored count − count over the old references it read + count over the references in memory after the scan`.

The column is `NULL` when the count is unknown: for snapshots written before this decision, and after level 2 replaced references of the snapshot (the store sets it to `NULL` there, because verified references change the count). The next scan writes it again.

### Fallback

The scan reads all references of the unchanged files in one pass, as before, when

- all names are resolved again (a base list changed, issue #408), or
- the stored count of the previous snapshot is `NULL`.

A limit on the share of touched symbols was tried and dropped: even when every sixth symbol is touched, the reads through the indexes were not slower than the pass over all references.

In the fallback nothing comes from the stored count or from pairs; the result is computed exactly as before this decision.

### Several reads instead of one

The reads are separate calls and not one transaction. That is safe because every read names the previous snapshot, and the rows of a snapshot change only when a later scan of the same project is stored or level 2 refreshes it. A project runs one scan at a time (`PRJ-302`), and level 2 belongs to the scan before. Before this decision the state was also read in a call of its own, long before the new snapshot was stored, so nothing is given up.

The stored count is not trusted blindly: if it is lower than the ambiguous places among the references read, the scan logs `ambiguous_count_too_low` and counts the rest as none. A reference that starts in a touched symbol and stands in a file that left is read twice and counted once.

### Ports

`PreviousState` no longer carries the references. `ProjectRepository` gets the reads listed above; `MetricsCalculator::measure` takes the pairs in addition to the references. `latest_index`, which level 2 uses, is unchanged.

## Consequences

- Migration 0019 adds the nullable column `snapshots.ambiguous_places` and the partial index `references_verified` (only rows with `verified = 1`). No new table: a full scan at level 1 writes nothing more than before, so its budget is not touched.
- The first incremental scan after the update and the first one after a level 2 run take the fallback and are as slow as before.
- Measured on Newtonsoft.Json (median of seven re-scans of one file, before and after, in ms): a test file used nowhere else 608 to 404, `BsonObjectId.cs` 636 to 506, `StringUtils.cs` 669 to 503. Files that much of the project uses touch every sixth symbol and gain nothing: `JsonSerializer.cs` 678 to 687, `JsonConvert.cs` 926 to 929.
- The budget of 300 ms is still not met for a project of this size. The main reason is not addressed here: an edited file gets new symbol ids, so every reference into it loses its target and its name is resolved again, even if only a comment changed. Keeping the ids of symbols that did not change would shrink the touched set at its root and needs its own decision. The rest of the time is in the read models, which are built again in full, and in the totals.
- The tests that compare an incremental scan with a full scan of the same files (issue #49) now compare coupling and the stored warnings as well as the references.
