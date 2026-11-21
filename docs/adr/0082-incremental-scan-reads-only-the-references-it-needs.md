# 0082 – An incremental scan reads only the references it needs

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

Reading is the expensive part and cannot be made cheaper per row: SQLite alone needs 63 ms for the rows, the rest is the column access. The only way is to read fewer rows.

The scan needs the old references for four things:

1. **Gone targets.** References from an unchanged file to a symbol of a changed or removed file lose their target; their names are resolved again (ADR 0033 decision 13 for the verified ones).
2. **Comparison.** The names that are resolved again produce references; those that equal a kept one are not written twice, kept ones that are no longer produced are dropped (issue #49, issue #408).
3. **Coupling.** Ce, Ca and instability depend on the references of the whole snapshot (ADR 0037, ADR 0074).
4. **Warnings.** The number of ambiguous places counts over the whole snapshot.

The product owner released this work on 2026-11-21. Recall goes before speed: the result of an incremental scan must stay the one a full scan gives.

## Decision

### Touched and untouched files

An unchanged file is **touched** by an incremental scan if at least one of its names is resolved again or one of its references points at a symbol that is gone. Every other unchanged file is **untouched**: the scan neither adds, drops nor changes a reference in it. This is the rule the scan already follows place by place (`Carried::reconcile` keeps what is out of scope as it is); the decision only draws the line by file, so that whole files can be left in the store.

The scan reads from the store:

- the references that point at a symbol of a changed or removed file (by the index on the target),
- the files that hold a reference verified by level 2,
- all references of the touched files, once the names to resolve again are known (by the index on the file),
- the references of the previous state of the changed and removed files, for the warning count below.

References of untouched files are never read as full rows.

### Coupling

Coupling is still computed over the whole snapshot, never updated in place, because the unit of a symbol depends on the declarations of all files. For the untouched files the store returns only the distinct pairs *(from symbol, to symbol)* of their references that are not of kind `test`. The pairs of the touched and changed files come from memory. `ce` and `ca` count each pair of units once, so pairs give the same values as the references.

### Ambiguous places

A place is a file and a span, so the count is a sum over files. A snapshot stores its count in the new column `snapshots.ambiguous_places`. The next incremental scan computes

`new count = stored count − count of the old references of the touched, changed and removed files + count of their new references`.

The column is `NULL` when the count is unknown: for snapshots written before this decision, and after level 2 replaced references of the snapshot (the store sets it to `NULL` there, because verified references change the count). The next scan writes it again.

### Fallback

The scan reads all references of the unchanged files, as before, when

- all names are resolved again (a base list changed, issue #408): every unchanged file is touched then, or
- the stored count of the previous snapshot is `NULL`.

In the fallback nothing comes from the stored count or from pairs; the result is computed exactly as before this decision.

### Ports

`PreviousState` no longer carries the references. `ProjectRepository` gets the reads listed above; `MetricsCalculator::measure` takes the pairs of the untouched files in addition to the references. `latest_index`, which level 2 uses, is unchanged.

## Consequences

- Migration 0018 adds the nullable column `snapshots.ambiguous_places`. No new table, no new index: a full scan writes nothing more than before, so its budget is not touched.
- The first incremental scan after the update and the first one after a level 2 run take the fallback and are as slow as before.
- Expected gain on Newtonsoft.Json: about 0.2 s per re-scan, more when the saved file is used in few places, less for a file that half the project uses. The budget of 300 ms is still not met for a project of this size; the remaining time is in reading the stored names, in the read models, which are built again in full, and in the totals. Those are not part of this decision.
- A test compares the stored result of an incremental scan with a full scan of the same files (references, coupling, warning count), for the narrow path and for the fallback.
- Storing the pairs per file, so that coupling needs no pass over the references at all, stays open. It would cost a full scan an extra table and needs its own decision.
