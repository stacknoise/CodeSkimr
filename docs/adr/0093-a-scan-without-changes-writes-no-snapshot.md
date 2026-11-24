# 0093 – A scan that finds nothing changed writes no snapshot

Status: accepted (decision of Anton of 2026-11-24, card "Soll ein Scan ohne Änderung keinen neuen Stand mehr schreiben?": "Ja, bestehenden Stand bestätigen"; after the hand test of 1.0.0-rc.6 of the same day, in which an incremental scan of `microsoft/vscode` without any change took 56.8 s)

## Context

An incremental scan (blueprint 7) analyses only the files whose hash changed. Everything after that ran all the same, also when no file had changed: the discover stage read every symbol of the newest snapshot, the resolve stage resolved the base lists of all files again against the whole symbol table, the measure stage measured every symbol, and the persist stage wrote a new snapshot, carried every row over to it, rebuilt the read models and pruned the oldest snapshot. In the hand test of rc.6 on Windows (`microsoft/vscode`, 16,717 files, no file changed) that was 15.3 s for the discover stage, 23.8 s for the resolve stage and 17.3 s for the persist stage, 56.8 s in all, for a result that is the snapshot the project already had.

## Options

1. Make each of the stages cheaper for a scan with few changes. That is worth doing, but the work still grows with the project when the answer is known after the discover stage.
2. Write a new snapshot that shares every row with the previous one, without resolving and measuring. The read models are stored per snapshot, so they would have to be copied or rebuilt, which is most of the time of the persist stage.
3. Stop after the discover stage: the newest snapshot is the result of the scan.

## Decision

Option 3.

- **When.** An incremental scan found nothing changed if the newest snapshot exists and the scan sees exactly its files, each with the stored hash and language. The pipeline compares them itself and does not rely on what the discovery adapter marked. A new file, a file that is gone, a file that could not be analysed before (its hash is not the stored one, or it is in no snapshot) and a scan that fell back to a full one (other naming rules, no stored names) are changes. A scan restricted to paths looks only at those paths, as before.
- **What runs.** The discover stage reads the files of the newest snapshot without its symbols (`ProjectRepository::latest_head`); the symbols are read only once a change is known (`snapshot_symbols`). The analyze, resolve and measure stages pass through. The persist stage writes no snapshot: it sets the creation time and the commit of the compared snapshot to those of this scan and its mode to incremental (`SnapshotStore::confirm`), in one statement. If another process wrote a snapshot in between, the compared one is no longer the newest; nothing is stamped and the scan fails with `STO-302` (retryable).
- **Layers.** The layers of a project are no scanned file (`.codeskimr.toml`), and the layer graph was built with every snapshot. Now the store builds the layer graph of the newest snapshot again when the layers of the project are saved with another value (`save_project`), which the scan does before it starts. Layers that changed therefore show after the next scan as before, with or without a changed file.
- **Settings.** A setting that goes into resolving or measuring (`scan.max_candidates`, say) applies to the names a scan resolves. An incremental scan resolved only the names its changes reached and the base lists, so such a setting has needed a full scan to apply everywhere before this decision too; a scan without changes now applies it nowhere.
- **What the listeners see.** The scan ends like any other: the stages report their progress, `SnapshotCreated` is published with the confirmed snapshot and says `unchanged`, the result of the scan names the warnings the snapshot stores, the alert step runs, the report of the scan is stored, and level 2 runs where the caller asked for it. `rescan` of MCP returns the id of the newest snapshot, which is a new id only if something changed.
- **The screen.** The progress card says "No file changed · N checked" where it said "0 of 0 files", when the end event says `unchanged`.

## Consequences

- A scan without changes costs the walk over the folder with the hashes and one read of the file list. Resolving, measuring and writing are gone (41 s of the 56.8 s above), and of the discover stage the read of all symbols.
- The history of snapshots holds one entry per change, not one per scan. The retention (`storage.snapshot_retention`) therefore reaches further back, and a scan without changes no longer prunes the oldest snapshot.
- The creation time of a snapshot is the time of the last scan that confirmed it, and its mode is the mode of that scan. Nothing but the screens reads the mode.
- What is left of the time is the hashing: every file is read to compare its hash. Comparing size and modification time first would need both stored per file (a migration) and trusts the file system; it is not part of this decision.
- A project with a file that is left out for good (an encoding that is not read) never takes this path: the file is in no snapshot, so every scan sees it as new and analyses it again. The result is right, the time is the old one. Storing the hash of such a file with the snapshot would close that; it needs a migration and is not part of this decision (issue #684).
