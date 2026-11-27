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
- **A log file in the data folder.** `logs/codeskimr.log` receives every line that standard error gets, through the same redaction (blueprint S-01); no line reaches the file unredacted. The file rotates at 10 MB and keeps 5 files in all (`codeskimr.log` and `.1` to `.4`). On Unix it is readable by the owner only. A link or a folder of that name is refused, so the log never writes through something that points elsewhere. A file that cannot be rotated (for example because another program holds it open) keeps growing past the limit; this is accepted, and nothing is logged about it, because the log must not write into itself. Emptying the file uses a handle of its own, because the append handle may not cut a file on Windows. A file that cannot be opened leaves the log on standard error only. The log file of the tray copy (`codeskimr.log` in the data folder itself, ADR 0067) stays as it is; it holds standard error only and starts empty with every start.
- **Five commands, none takes a path.** `log.status` and `log.tail` (right `read_source`, so the token is needed like for the source view, ADR 0035: the log holds paths and names of projects and the path of the data folder, and no other program on the machine may read it through the open port), `log.set_level`, `log.clear` and `log.open_folder` (manage). The file and its folder are built from the data folder. `log.set_level` accepts `off`, `warn`, `info`, `debug` and `trace` and refuses anything else, and a run whose log another subscriber owns, with `API-101`. `log.tail` gives at most 2000 lines and reads only the last 512 KiB. `log.open_folder` opens the folder with the program of the operating system like `project_file.open` (ADR 0111) and answers `APP-606` where nobody sits at the machine. New: `APP-606`, `APP-607`.
- **REST only.** The commands are not MCP tools: an agent has no use for changing the log of its host.
- **The level applies to the program, not to libraries.** The filter sets the chosen level for the targets that start with `codeskimr`; every other target stays at most at the default level (`warn`), so Debug and Trace chosen in the UI do not log what the requests of a library carry. The level that `CODESKIMR_LOG` sets at the start, and the default after the 24 hours, count for every target as they always did: whoever sets the variable asked for it. The redaction (S-01) is pattern based and remains the second line.
- **Hardened file.** On Unix the file is opened with `O_NOFOLLOW` (a link is refused by the open itself), set to mode 0600 even if it existed with wider rights, and its folder is created with 0700. On Windows the rights of the data folder in the user profile apply. One timer thread serves every choice of a level; a new choice only moves its end.
- **Copying.** The button "Copy log" is followed by a note that the text can hold paths and names of projects and should be read before it goes into a public issue (CLAUDE.md rule 11).
- **Not in the log, as before:** no logins, no tokens, no source text. A line that a secret reached anyway is masked by the redaction layer before it is written.

## Consequences

- No behaviour changes for a person who does not open the tab, except that a log file appears in the data folder.
- The data folder holds up to 50 MB of logs at most.
- The level of a run started from a terminal can be changed from the UI of that run.
- A program that installs its own subscriber first (a test, an embedder) keeps it; the tab then shows that the level cannot be changed.
