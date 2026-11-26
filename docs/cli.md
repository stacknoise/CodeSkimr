# CLI and composition root (WP-25)

Implements the first part of blueprint section 10.2: `codeskimr scan` and `codeskimr mcp`. `codeskimr serve` came with WP-41 (see `docs/server.md`) and `codeskimr check` with WP-35 (see `docs/conventions.md`). `codeskimr export` came with WP-75 (see `docs/export.md`). `codeskimr codes` lists the error codes or explains one (see below). The bare `codeskimr` starts the server and opens the browser (see `docs/server.md`); on Windows and macOS it does so in the system tray, `--console` keeps it in the window (ADR 0067); its start message ends with the hint to keep the window open, because closing it ends the server.

## Structure

- `codeskimr_cli` knows no adapter. It defines the commands with clap (help texts come from `locales/{de,en}/cli.ftl`), turns `argv` into an `Invocation` and renders the summary and the error line.
- `codeskimr` (`crates/app`) is the composition root. `main.rs` only starts `run::execute`. `wiring.rs` is the only place that creates adapters: `SqliteStore`, `SourceFetcher`, `WalkDiscovery`, `TreeSitterAnalyzer` (with fingerprints from `config.similarity`, closes #41) and the `Scanner`. `scan.rs`, `mcp.rs` and `check.rs` are the commands; `build_analysis` creates the discovery and the analyzer that scan and check share.

## `codeskimr scan <path|git-url>`

```
codeskimr scan <path|git-url> [--branch <name>] [--json] [--watch] [--credential <id>] [--name <name>]
```

1. The argument is an existing folder (made absolute) or a Git URL (`https://`, `http://`, `ssh://`, `git://`, `git@…`, or ending in `.git`). A Git URL with secrets in it (user info for HTTP(S), a password anywhere, a query or a fragment, in any letter case) is refused with `PRJ-101` and a second line that names the reason: they would be stored in the database and printed (blueprint 12). A bare SSH user name (`git@host:org/repo`) is fine. `--branch` only applies to Git sources and is ignored for folders. On Windows the `\\?\` prefix of the absolute path is removed (UNC paths keep it). The branch defaults to `main`.
2. The source is registered as a project (the same source is registered once; scanning again reuses the project).
3. One full scan runs: fetch, discover, analyze, resolve, measure, persist. A new snapshot is written; `--json` prints `{project, snapshot, files, symbols, references, seconds, cpu_seconds, pack_load_seconds, stages, level2, warnings, left_out, tools}` on one line (`tools` is the result of the tool check, `docs/tools-check.md`; `stages` has the seconds of each stage that reported, as `{stage, seconds}` in the order fetch, discover, analyze, resolve, measure, persist, the same wire labels the events use and in both languages unchanged; unlike `seconds`, the times of the stages include the time loading language packs), otherwise a summary in the language of the user (the totals, one line "Steps: …" with the stage times, a line for each language level 2 looked at and for each warning), followed by one line for each tool of level 2 that is missing, too old or whose version could not be read, with its purpose and install hint. A failed tool check never fails the scan.
4. Without `--watch` the scan is full and the command ends.
5. With `--watch` (ADR 0029) the command keeps running after the first full scan. The watcher (`watcher.debounce_ms`, `watcher.max_wait_ms`, `scan.excluded_folders`) publishes `FilesChanged`; `watch.rs` turns the changes that piled up into one incremental scan, or into a full scan if the operating system lost events. Scans never overlap. Re-scans do not check the tools again (`docs/tools-check.md`). Each re-scan prints one line (`N changed files scanned again in M ms.`; with `--json` the usual summary line). A failed re-scan prints its error and watching goes on. Only folders can be watched: a Git URL fails with `CLI-102`. Stop with Ctrl+C.

`--name <name>` gives the project a display name (at most 80 characters, no control or invisible characters such as a direction override; `PRJ-102` otherwise). It is only a label: the id and the source stay. If the source is registered already, the name replaces the old one; without `--name` the name stays. A new project is named after its folder, or after the folder and its parent (`shop/src`) if the folder is called `src`, `source`, `sources`, `lib` or `www` (`GENERIC_FOLDER_NAMES`).

`--credential <id>` names a stored credential (see below) that logs in to a Git source; for a folder it fails with `PRJ-101`. Without it the system's Git credential helper decides, and there is never a prompt on the terminal (the clone runs with `gitoxide.credentials.terminalPrompt=false`), so a server or an MCP session cannot hang on a login.

## `codeskimr mcp`

```
codeskimr mcp [--project <folder>]
```

Serves the three tools of WP-26 over stdin and stdout for the project in `--project` (default: the current folder). The folder must have been scanned before; otherwise the command stops with `PRJ-201` (not registered) or `MCP-203` (registered, but no snapshot). Nothing but the protocol is written to standard output; errors go to standard error.


## `codeskimr credential`

```text
codeskimr credential set <id> --user <name>
codeskimr credential delete <id>
codeskimr credential login <id> --provider github|gitlab --client-id <id> [--host <https-url>]
```

Manages the credentials in the keystore of the operating system (blueprint 12.2, `docs/secrets.md`). `set` stores or replaces one: the secret is read without echo from the terminal, or from standard input if there is none (`printf %s "$TOKEN" | codeskimr credential set work --user anton`). There is no flag for the secret, so it never lands in a shell history, a process list or an environment dump. An empty secret fails with `CLI-103`. `delete` removes the entry; a missing one is not an error. `login` runs the OAuth device flow: it prints a page and a short code, waits until the user has approved in the browser and stores the token (`NET-601` if the sign-in does not finish). The name `installation-token` belongs to the local server and is refused (`CLI-101`). Only the name of the credential is ever printed.

`--client-id` is the id of an OAuth application that is registered at the platform (not a secret). `--host` selects a self-managed installation and must be an HTTPS address.

## `codeskimr level2 confirm|revoke <project>`

Allows or forbids level 2 (the indexers of ADR 0033, which run the build tools of the project) for one project. `<project>` is the project id or the folder or Git URL it was scanned from, as for `export`. `confirm` needs a terminal on standard input and output: it prints what level 2 runs and with which rights (file and network access, no sandbox, the binding to source, location and branch and not to a commit) then shows the name, the location (folder or Git address) and the branch of the project the argument resolved to (a numeric id is resolved too). A Git URL resolves only the default branch of that repository; a project at another branch is addressed by its id. It starts only if the user types the name of that project (no fixed word such as `yes`). The project is read again after the answer and nothing is stored if its name, location or branch changed in the meantime. Without a terminal it fails with `SEC-403`; nothing can pass the word as an argument or through a pipe. `revoke` forbids it again at once. Nothing else confirms level 2: not MCP, not the server, not watch mode (`tests/consent_origin.rs` checks the source). Only `codeskimr scan` starts the indexers; `scan` prints one line per language with an indexer: "Level 2: ... checked" or "skipped (reason)".

## `codeskimr open`

```
codeskimr open [--no-browser [--print-url]]
```

Opens the UI of the running local server in the browser. It asks the server for a one-time launch code, writes it to a private launch file (mode 0600), prints `Launch file: …` and starts the system browser with it, then waits until the code is used or has expired and deletes the file. `--no-browser` does not start a browser. `--no-browser --print-url` (opt-in, ADR 0088) writes no launch file but prints one line, `http://127.0.0.1:<port>/#code=…`, for tools that cannot open `file://` pages (browser tests of an agent); it returns at once, and the code works once within `ui.launch_code_seconds`. The code is then in your terminal or log: use it only where nobody else reads the output. `--print-url` alone is refused with `CLI-101`. Without a running server it stops with `API-601`. Details: `docs/server.md`.

## `codeskimr serve`

```
codeskimr serve
```

Starts the local server (`docs/server.md`) on `127.0.0.1` and the port `server.port` (default 7421), prints `Listening on http://127.0.0.1:7421` and runs until Ctrl+C. There is no option for the address: the server never listens anywhere else (blueprint S-04, `API-102`). On the first start it creates the installation token in the keystore of the operating system; without a usable keystore it stops with `SEC-601`. A taken port is `API-301`.
## `codeskimr check`

```
codeskimr check [PATH ...] [--json]
```

Runs the convention checks (`docs/conventions.md`) over the current folder, or over the files and folders named, relative to it. Nothing is stored and no scan is needed. The text output has two lines per finding (`path:line: CHK-104 message`, then the numbers), the files that were skipped, a summary and the rules that are not checked yet; `--json` prints one line with `files_checked`, `findings`, `unchecked` and `not_checked` (lines are one-based). The exit code is `1` if there is at least one finding or a named path matches no file, so CI can fail on it.

## `codeskimr codes`

```
codeskimr codes [<code>]
```

Without an argument it prints every active error code (retired ones, ADR 0078, are left out) with its message in the language of the user, one line each (`SRC-201  <message>`). With a code (any case, `src-201` works) it explains it: the message, the English developer description, the group, the severity, whether a retry can help and the version since which the code exists; a retired code says so. An id that is not registered fails with `CLI-101` and says so. The data come from the generated registry (`errors/error-codes.toml`), so the command knows no list of its own; nothing is read from disk and no scan is needed.

## `codeskimr export`

```
codeskimr export <project> --html <file>
```

Writes the explorer, the dependency map, the usages and the search of a scanned project into one self-contained HTML file, without source text (`docs/export.md`). `<project>` is a project id or the folder or Git URL it was scanned from. The export page is the built UI: it is part of the program, and a UI folder (`ui.dir`) that exists is read instead. If the program was built without the UI and there is no folder with the built export page (`npm run build` in `web/`), the command stops with `EXP-201`; a project with more references than `export.max_references` (default 600000) is refused with `EXP-502`, a second export of the same project at the same time with `EXP-501`; an unwritable file is `EXP-601`.

## `codeskimr symbols`, `usages`, `impact`, `context` and `unused`

```
codeskimr symbols <query> [--kind KIND] [--limit N] [--project ID|FOLDER] [--json]
codeskimr usages <symbol> [--direction incoming|outgoing] [--depth N] [--include-tests] [--limit N] [--project ID|FOLDER] [--json]
codeskimr impact <symbol> [--limit N] [--project ID|FOLDER] [--json]
codeskimr context <symbol> [--budget short|medium|long] [--project ID|FOLDER] [--json]
codeskimr unused [--kind all|types|functions] [--language ID] [--assessment likely|public|approximate] [--include-tests] [--limit N] [--project ID|FOLDER] [--json]
```

The questions of the MCP tools `search_symbols`, `find_usages`, `impact_of_change`, `get_context` and `find_unused` (`docs/mcp.md`), asked from the command line, for scripts, CI and agents without MCP (ADR 0099). The commands call the same tools with the same limits (`mcp.default_limit`, `mcp.max_limit`, `mcp.default_depth`, `mcp.max_depth`, `mcp.context_items_*`), so a script and an agent get the same answer. `<symbol>` is an id or a qualified name. The project is the one that was scanned from the current folder, or the one named with `--project` (an id, or the folder or Git URL it was scanned from).

- **`unused`.** Lists the symbols for which no usage was found, with the assessment and the reason (ADR 0101), without code in tests unless `--include-tests` is given. The symbols that the `[[unused.ignore]]` list of the project file takes out are printed below the list with the reason of their entry; with `--json` they are `left_out`. A broken list fails with `CFG-101` or `CFG-102`. The counterpart in the UI is the tab "No usage found" of the statistics screen.
- **Newest scan, no hidden scan.** The commands read the last scan of the project; they never scan by themselves. If a project folder has files that changed, were added or went away since that scan, the answer says so: `"stale": true` in JSON, a closing line in the text. `stale` is `null` if that cannot be told: a Git source, which only a scan updates, or a folder that cannot be read. `null` never means "current"; only `false` does. To find out, the command hashes the files like a scan does, without parsing them.
- **Text.** One line per hit; `context` shows the symbol, its callers, dependencies, files and notes under headings (a heading is left out when its list is empty). Texts that come from the project lose their control characters. A closing line says if the answer was cut or is preliminary (unchecked usages).
- **JSON.** The answer of the MCP tool with two more fields: `schema` (the version of the format, now `1`; it grows when a field changes its meaning or goes away) and `stale`. Nothing else is printed to standard output, no colours, no progress.
- **Failures.** With `--json` a failure is JSON on standard output, `{ "schema": 1, "error": { "code", "description", "params" } }` with the registered code (`MCP-202` for an unknown symbol, `PRJ-201` for a project that is not registered, `MCP-203` for one without a scan, `MCP-101` for an invalid argument), and the exit code is `1`. Without `--json` the failure is the usual `error[CODE]: text` on standard error. A usage error (an unknown option, a `--direction` or `--budget` outside the list) is exit code `2`.
- **UI.** The counterpart is the explorer, the search and the usages screen; `--json` is an exception of ADR 0092 whose counterpart is the REST interface.

## `codeskimr annotations` and `codeskimr alerts`

```
codeskimr annotations list <project> [--symbol NAME --symbol-kind KIND [--param TYPE]...] [--kind K] [--status S] [--limit N] [--offset N] [--json]
codeskimr annotations add <project> --symbol NAME --symbol-kind KIND --kind critical|note|task [--title T] [--text T] [--status S] [--due SECONDS] [--assignee NAME] [--json]
codeskimr annotations update <project> <id> [--symbol NAME --symbol-kind KIND] [--title T] [--text T] [--status S] [--due SECONDS] [--assignee NAME] [--clear-text] [--clear-due] [--clear-assignee] [--json]
codeskimr annotations remove <project> <id> [--json]
codeskimr alerts list <project> [--state open|dismissed] [--kind K]... [--limit N] [--offset N] [--json]
codeskimr alerts dismiss <project> <alert> [--json]
codeskimr alerts dismiss-all <project> [--kind K]... --yes [--json]
```

Notes, tasks and critical marks on symbols and the alerts of critical symbols that changed (ADR 0068). `<project>` is a project id or the folder or Git URL it was scanned from. The commands run the registered commands `annotations.*` and `alerts.*` as the local owner, so they answer the same as the REST routes (`docs/server.md`); `--json` prints that answer, otherwise one line per row in the language of the system. `--symbol` is a qualified name and `--symbol-kind` its kind label (`class`, `method`, ...); `--param` repeats for the parameter types of an overload. `dismiss-all` does nothing without `--yes` and ends with `API-101`. Lists are paged; the text says how to continue (`--offset`). Exit code 0, or 1 with the code on standard error. The parser is written by hand with the other commands; it is not generated from the registry.

## Global options

| Option | Environment | Default |
|---|---|---|
| `--data-dir <dir>` | `CODESKIMR_DATA_DIR` | `%LOCALAPPDATA%\codeskimr`, `~/Library/Application Support/codeskimr`, `$XDG_DATA_HOME/codeskimr` or `~/.local/share/codeskimr` |
| `--languages <dir>` | `CODESKIMR_LANGUAGES_DIR` | `languages/` next to the executable, otherwise the packs built into the program (written to `languages/<id>/` in the data folder); a `languages/` folder in the current folder is not used |
| `--config <file>` | `CODESKIMR_CONFIG` | `config.toml` in `%APPDATA%\codeskimr`, `~/Library/Application Support/codeskimr` or `$XDG_CONFIG_HOME/codeskimr` |
| `--set key=value` (repeatable) | | |

The data folder holds `codeskimr.db` and the Git cache `git/`. A config file named with the flag or the environment must exist (`CFG-102` otherwise); the default one is optional.

## Configuration chain

Blueprint section 14, in this order, later wins: the defaults of `config/default.toml`, the user config file, `.codeskimr.toml` in the scanned folder (folders only; a Git source is read after the fetch, which is not wired yet), then `--set key=value` flags (`--set scan.excluded_folders=["a","b"]`; the value is TOML). Unknown keys and invalid values fail with `CFG-103` and `CFG-101`. The architecture layers of a project are set with `[[layers]]` tables (`name`, `namespaces` = qualified-name prefixes, `.` or `::` as separator; the first layer that owns a name wins; optional `may_use` = names of the layers it may depend on, missing means no limit, an empty list means none, an unknown name is `CFG-101`) in the project's `.codeskimr.toml`; they apply at the next scan (ADR 0031, this repository's own file is an example).

The package added two keys: `scan.parse_timeout_ms` (default 5000, `PRS-501` on expiry) and `scan.max_source_mb` (default 2048, `SRC-501` above it).

## Language of the texts

`CODESKIMR_LANG`, then `LC_ALL`, `LC_MESSAGES`, `LANG`: the first one that is set decides; a value starting with `de` selects German, everything else English. On Windows, where `LANG` is usually missing, set `CODESKIMR_LANG=de`.

## Exit codes

`0` success, `1` a command failed (`error[CODE]: text` on standard error) or `check` found violations, `2` invalid arguments (`CLI-101`, with the parser's explanation).

## Known limits

- References come from the level-1 resolver (`docs/references.md`).
- The language packs are read from a folder (see `--languages`); the packs built into the binary are used (and written to the data folder) only if no folder was named and no `languages/` folder with at least one `<language>/language.toml` lies next to the executable. A `languages/` folder in the current folder (for example in a scanned repository) is ignored.
- `Ctrl+C` ends the process; the server cancels a scan cleanly through `CancelToken` (`projects.cancel`, `docs/server.md`); the CLI does not, so the process just ends.
- A user config error that is not TOML syntax (unreadable file) is reported as `CFG-102`.

## Caller identity

Both commands run as the local owner of the installation (`crates/app/src/principal.rs`), who holds every permission, so nothing changes for the user. The scanner and the MCP tools check the permission they need (`scan`, `read`) before they do anything; a caller without it gets `AUTH-401`. See blueprint 10.1 and ADR 0019.
