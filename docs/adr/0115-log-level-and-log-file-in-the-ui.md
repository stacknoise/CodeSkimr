# 0115 – Log level and log file in the UI

Status: accepted (variant A of mock 42 was chosen by the product owner on 2026-11-27: tab "Diagnosis" in the settings). Follow-up of ADR 0092.

## Context

The log level could only be set with the environment variable `CODESKIMR_LOG`, and the log went to standard error only. A person who starts the app from the start menu or the tray has no console, so to get a log with `info` they had to start the program from a terminal with the variable set (found in the hand test of 2026-11-27). Principle of the product (ADR 0092): what the console can do, the UI can do.

## Options

1. A tab "Diagnosis" in the settings: choose the level, see the file, copy, open the folder, empty it, read the last lines (mock 42, variant A).
2. A switch in the status bar with a small popup, on or off only (mock 42, variant B; not chosen).
3. Keep the variable and document it.

## Decision

Option 1.

- **The level changes while the program runs.** The subscriber is installed once with a reloadable level filter. `CODESKIMR_LOG` still sets the level at the start and stays the default; the UI changes the level at once, for standard error and the file alike. The level chosen in the UI is not saved: a restart goes back to the default.
- **Back to the default after 24 hours.** A level other than the default ends after `LOG_LEVEL_HOLD` (24 hours), so that a forgotten `trace` cannot fill the disk. A new choice restarts the time.
- **A log file in the data folder.** `logs/codeskimr.log` receives every line that standard error gets, through the same redaction (blueprint S-01); no line reaches the file unredacted. The file rotates at 10 MB and keeps 5 files in all (`codeskimr.log` and `.1` to `.4`). On Unix it is readable by the owner only. A link or a folder of that name is refused, so the log never writes through something that points elsewhere. A file that cannot be opened leaves the log on standard error only. The log file of the tray copy (`codeskimr.log` in the data folder itself, ADR 0067) stays as it is; it holds standard error only and starts empty with every start.
- **Five commands, none takes a path.** `log.status` and `log.tail` (read), `log.set_level`, `log.clear` and `log.open_folder` (manage). The file and its folder are built from the data folder. `log.set_level` accepts `off`, `warn`, `info`, `debug` and `trace` and refuses anything else with `API-101`. `log.tail` gives at most 2000 lines and reads only the last 512 KiB. `log.open_folder` opens the folder with the program of the operating system like `project_file.open` (ADR 0111) and answers `APP-606` where nobody sits at the machine. New: `APP-606`, `APP-607`.
- **REST only.** The commands are not MCP tools: an agent has no use for changing the log of its host.
- **Not in the log, as before:** no logins, no tokens, no source text. A line that a secret reached anyway is masked by the redaction layer before it is written.

## Consequences

- No behaviour changes for a person who does not open the tab, except that a log file appears in the data folder.
- The data folder holds up to 50 MB of logs at most.
- The level of a run started from a terminal can be changed from the UI of that run.
- A program that installs its own subscriber first (a test, an embedder) keeps it; the tab then shows that the level cannot be changed.
