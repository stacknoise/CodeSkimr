# Projects and scan

A **project** is a source that CodeSkimr knows: a folder on your machine or a Git repository. A **scan** reads the source and stores the result as a **snapshot**. Browsing, search, usages and MCP always answer from the newest snapshot. CodeSkimr keeps the last five snapshots per project.

## In the browser

The first screen is the project list.

- **Add a project:** type or paste an absolute folder path or a Git address into the source field and confirm. Key `N` jumps to the field. A folder or a text dropped on the window goes into the field; some browsers do not reveal the path of a dropped folder, then the screen asks you to paste it.
- **Open a project:** select it with the arrow keys or the mouse and press `Enter`. It becomes the active project that the explorer, the dependency map and the usages screen show.
- **Scan again:** key `R` or the small "scan again" button next to the status. While a scan runs the status reads "scan running".

The same source is registered only once. Two scans of the same project cannot run at once (`PRJ-302`).

A relative folder path is refused (`PRJ-101`), so the folder the server happens to run in never decides what is scanned.

## Scan and performance screen

Shows the scan that is running or ran last: the stage it is in (fetch source, find files, parse, resolve dependencies, metrics, save), files and lines per second, cores in use, memory, and a table per language with tier, files, symbols and parse time. "Recent scans" lists earlier ones. "Performance targets" compares measured values of the program with its budgets (program start, search per keystroke, switching a type, usages query, map layout).

The first scan of a project is full. Later scans are **incremental**: files are compared by a content hash and only changed files are read again; a scan after a small change finishes in a fraction of a second. The interface stays usable during a scan.

## On the command line

```
codeskimr scan <path|git-url> [--branch <name>] [--json] [--watch] [--credential <name>]
```

- The argument is an existing folder or a Git address (`https://`, `http://`, `ssh://`, `git://`, `git@host:org/repo`, or ending in `.git`). A folder is made absolute. `--branch` only applies to Git sources and defaults to `main`.
- The command registers the source as a project (once) and runs one full scan. The result is a two-line summary, or one JSON line with `--json`: `{project, snapshot, files, symbols, references, seconds}`.
- **`--watch`** keeps the command running after the first scan. When files in the folder change, they are collected for a short moment and scanned again incrementally; each re-scan prints one line. Only folders can be watched (`CLI-102` for a Git address). Stop with Ctrl+C. A failed re-scan prints its error and watching goes on.
- **Secrets in a Git address** (user name and password, a token, a query or a fragment) are refused with `PRJ-101`. See [Sources and credentials](sources-and-credentials.md) for the right way (`--credential`).

Other commands: `codeskimr serve` starts the local server, `codeskimr open` opens the browser, `codeskimr mcp` starts the MCP server on standard input and output ([MCP setup](mcp.md)), and `codeskimr check` runs the convention checks over a folder without a scan.

Global options for all commands: `--data-dir`, `--languages`, `--config` and `--set key=value` (see [Installation](installation.md)).

Exit codes: `0` success, `1` a command failed (`error[CODE]: text` on standard error) or `check` found violations, `2` invalid arguments (`CLI-101`).

## What a scan skips

Files that `.gitignore` excludes (also outside Git repositories), the folders in `scan.excluded_folders`, the `.git` folder, and files of unknown languages (they do not count against the size limit). Limits: a project over `scan.max_source_mb` (default 2048 MB) fails with `SRC-501`; a single file that takes longer than `scan.parse_timeout_ms` (default 5000 ms) to parse fails with `PRS-501`. A file that is not valid text (`PRS-101`) is skipped.

## Layers

CodeSkimr shows your architecture as **layers**. You declare them in the file `.codeskimr.toml` in the root of the scanned folder:

```toml
[[layers]]
name = "Domain"
namespaces = ["Shop.Domain"]

[[layers]]
name = "Application"
namespaces = ["Shop.Application"]

[[layers]]
name = "Infrastructure"
namespaces = ["Shop.Infrastructure", "Shop.Data"]
```

`namespaces` are qualified-name prefixes; the separator is `.` or `::`. A name belongs to the first layer whose prefix matches. List outer layers first: the dependency map draws them on top, one row per layer. Layers apply at the next scan. Without layers the explorer groups everything under an empty layer and the map is empty.
