# 0085 – The files a scan left out are stored per snapshot and shown on the scan screen

Status: accepted

## Context

A file that alone cannot be analysed (not valid UTF-8 or UTF-16, `PRS-101`; unreadable, `PRS-102`; parsing over the time limit, `PRS-501`) is left out and the scan goes on (issue #621). The scan names the files in its summary and keeps one warning per code with the count in `snapshot_warnings`. The command line shows the names; the browser did not: they were gone as soon as the scan ended, because no table kept them. ADR 0072, decision 9, said that the stored scan report holds no paths, so that a database copy or an export never leaks the layout of a private project through a report. Issue #635 asks the browser to show the files, with path and code.

## Decision

1. **A table `snapshot_left_out`** (migration `0018`): `snapshot_id`, `path`, `code`, `kept`, primary key `(snapshot_id, path)`. `kept` says whether the file is shown as the previous scan saw it (a failure that may pass, such as a locked file) or is missing in the result. It is written in the same transaction as the snapshot and deleted with it.
2. **Bounded.** The table holds at most `scan.max_left_out_listed` rows per snapshot, the same limit the command line uses. The count of all of them stays in `snapshot_warnings`; the screen says "and N more files" from the difference. A scan with a million unreadable files writes at most the limit.
3. **Not carried over.** An incremental scan writes the list of its own run, like its warnings: the file list always describes the newest scan, not a mixture.
4. **Decision 9 of ADR 0072 is narrowed, not dropped.** The tables `scan_reports` and `level2_*` still hold no paths. The paths of the left-out files are the same paths the `files` table holds for every analysed file, so they add no new kind of information to the database; they stay in the user's own data directory and are not part of the HTML export (the export reads the project list without these fields).
5. **Output.** `projects.list` gets `left_out` (`path`, `code`, `kept`) and `left_out_more` per project. The scan screen shows a list "Files left out" with path, code (message as tooltip) and "missing in the result" or "shown as the previous scan saw it", and the number of the rest.

## Consequences

- Blueprint changes (label `blueprint-change`): `docs/storage.md` (one more table), `docs/server.md` (two output fields), the manual page on projects and scans (de and en), screen 5 of the mock (the list below the languages).
- Databases of an older version get the empty table with the migration; a snapshot written before has no rows, so the screen shows the count of the warnings only (the names are there after the next scan).
- No new dependency, no new error code.
