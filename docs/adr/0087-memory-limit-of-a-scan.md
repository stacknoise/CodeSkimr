# 0087 – Memory limit of a scan

Status: accepted (follow-up of ADR 0084, asked for by the coordinator on 2026-11-21 after the measurement of the Linux kernel)

## Context

A scan keeps the symbols, the pending names and the references of the whole project in memory until it is stored. Measured on 2026-11-21 (release build, 4 cores): `microsoft/vscode` (16,717 files) needs 8.7 GB at the peak after ADR 0084 and 13 GB before; `drivers/net` of the Linux kernel (5.4 million lines of C) needs 0.8 GB, so the whole kernel (about 40 million lines) is estimated at 5 to 6 GB. A machine with less memory than a scan needs does not fail with a message: the system ends the whole program without a word. The user does not learn what happened, and a running app takes its other projects with it.

## Options

1. Do nothing: ADR 0084 reduces the peak, and a project that is still too large kills the program.
2. Estimate the memory from the number of files or lines before the scan and refuse. The estimate is wrong by a factor of ten between C and TypeScript.
3. Measure the memory of the program while the scan runs and stop the scan when it passes a limit.

## Decision

Option 3.

- Setting `scan.max_memory_mb` (default 16384). The limit that counts is the lower of this value and 70 % of the installed memory, so a machine with 8 GB stops a scan at about 5.7 GB.
- A watch thread of the scanner reads the resident memory of the program (`memory-stats`) every 500 ms while a scan runs. When the limit is passed, it raises the cancel token of the scan; every stage already polls the token, and the resolver now does too (every 4096 occurrences), because it is the stage that grew to 13 GB on VS Code.
- A scan stopped this way ends as a failure with the new code `PRJ-503` (`scan_memory_limit`), not as a cancellation, and writes no snapshot. The text tells what to do: exclude folders, lower `scan.max_candidates`, or scan on a machine with more memory; the text says that the limit never exceeds 70 % of the installed memory.
- Like the other limits that protect the machine, the project file `.codeskimr.toml` may only lower `scan.max_memory_mb` (ADR 0040, `LOWER_ONLY_SCAN_KEYS`). The limit is fixed when the program starts: a project value counts for scans started in the terminal or by the MCP server in that project folder; the app server reads no project file for it, so there only the user and global values count.
- The 70 % cap is deliberate and not raisable: the message names it and says that more is not possible on this machine, and the log carries the effective value in megabytes and the measured value. If the system gives no memory reading, or the watch thread cannot be started, the scan runs unwatched and the log says so (warning).
- The watch sleeps interruptibly, so a finished scan never waits for the check interval.
- The port `MemoryGauge` (application) hides the system call; the app crate wires `memory-stats` and `sysinfo`, which are already dependencies, so there is no new dependency.

## Consequences

- A scan that is too large ends within about half a second of passing the limit, with a clear message, instead of an abrupt end of the program. A step that holds one block of memory for longer (the write of a snapshot) is not interrupted; the stage boundaries before it are.
- The program measures its own memory: a second scan or the UI at the same time count toward the limit of a scan.
- The limit is a protection, not a budget: it never makes a scan faster.
