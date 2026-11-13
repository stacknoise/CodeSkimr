# 0027 – Watch mode is `codeskimr scan --watch`

Status: proposed

## Context

Blueprint 7 and milestone M2 require a watch mode: file-system events are debounced, the incremental re-scan stays under 300 ms. The file watcher (`codeskimr_source::publish_changes`, WP-30) and the incremental scan (WP-32) exist. What was missing is the piece that reacts to `FilesChanged` and scans again, and the place where a user switches it on. Blueprint 10.2 lists no watch command. The long-running process that holds the index in memory (`codeskimr`, `codeskimr serve`) does not exist yet.

## Options

1. A new command `codeskimr watch <path>`.
2. A flag `--watch` of `codeskimr scan`: scan once, then keep running and scan again on changes.
3. Wait for the server and start the watcher there.

## Decision

Option 2. It needs no second command that duplicates the arguments of `scan`, it works today, and the server can later start the same loop for its projects: the reacting code (`crates/app/src/watch.rs`) only needs a `Session` (project, scanner, event bus) and knows nothing about the command line.

- The first scan of a run is full. The watcher is started before it, so a file that changes during the first scan is scanned again right afterwards.
- Every following scan is incremental. If the operating system lost events (`FilesChanged.rescan`), it is a full scan.
- Scans never overlap. Events that arrive during a scan are merged into the next one; many changes in a row become one scan.
- Only folders can be watched. A Git source with `--watch` is refused with the new code `CLI-102` before anything is registered.
- A failed scan is reported and watching goes on; only a failure of the first scan or of the watcher itself ends the command. The user ends it with Ctrl+C.
- `watcher.max_wait_ms` (default 2000) is new next to `watcher.debounce_ms`: a build that writes all the time cannot postpone the scan beyond it.
- Output: one line per re-scan (`--json`: the usual summary, one line per scan).

## Consequences

- Blueprint 10.2 gets the `--watch` flag on the `scan` line.
- The incremental scan has known limits (`docs/incremental-scan.md`): a reference to a name that matched nothing before appears only after the declaring file changes again or after a full scan. Watch mode inherits them.
- The data folder inside the watched folder would make every scan trigger the next one; the default data folder lies outside. Not guarded here.
- No new dependency: Ctrl+C ends the process through the default signal behaviour.
