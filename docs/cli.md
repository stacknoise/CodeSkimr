# CLI and composition root (WP-25)

Implements the first part of blueprint section 10.2: `codeskimr scan` and `codeskimr mcp`. `codeskimr serve` came with WP-41 (see `docs/server.md`) and `codeskimr check` with WP-35 (see `docs/conventions.md`). `codeskimr export` came with WP-75 (see `docs/export.md`). The other command, `codes`, follows with its package. The bare `codeskimr` starts the server and opens the browser (see `docs/server.md`).

## Structure

- `codeskimr_cli` knows no adapter. It defines the commands with clap (help texts come from `locales/{de,en}/cli.ftl`), turns `argv` into an `Invocation` and renders the summary and the error line.
- `codeskimr` (`crates/app`) is the composition root. `main.rs` only starts `run::execute`. `wiring.rs` is the only place that creates adapters: `SqliteStore`, `SourceFetcher`, `WalkDiscovery`, `TreeSitterAnalyzer` (with fingerprints from `config.similarity`, closes #41) and the `Scanner`. `scan.rs`, `mcp.rs` and `check.rs` are the commands; `build_analysis` creates the discovery and the analyzer that scan and check share.

## `codeskimr scan <path|git-url>`

```
codeskimr scan <path|git-url> [--branch <name>] [--json] [--watch] [--credential <id>]
```

1. The argument is an existing folder (made absolute) or a Git URL (`https://`, `http://`, `ssh://`, `git://`, `git@…`, or ending in `.git`). A Git URL with secrets in it (user info for HTTP(S), a password anywhere, a query or a fragment, in any letter case) is refused with `PRJ-101` and a second line that names the reason: they would be stored in the database and printed (blueprint 12). A bare SSH user name (`git@host:org/repo`) is fine. `--branch` only applies to Git sources and is ignored for folders. On Windows the `\\?\` prefix of the absolute path is removed (UNC paths keep it). The branch defaults to `main`.
2. The source is registered as a project (the same source is registered once; scanning again reuses the project).
3. One full scan runs: fetch, discover, analyze, resolve, measure, persist. A new snapshot is written; `--json` prints `{project, snapshot, files, symbols, references, seconds, tools}` on one line (`tools` is the result of the tool check, `docs/tools-check.md`), otherwise a two-line summary in the language of the user, followed by one line for each tool of level 2 that is missing, too old or whose version could not be read, with its purpose and install hint. A failed tool check never fails the scan.
4. Without `--watch` the scan is full and the command ends.
5. With `--watch` (ADR 0029) the command keeps running after the first full scan. The watcher (`watcher.debounce_ms`, `watcher.max_wait_ms`, `scan.excluded_folders`) publishes `FilesChanged`; `watch.rs` turns the changes that piled up into one incremental scan, or into a full scan if the operating system lost events. Scans never overlap. Each re-scan prints one line (`N changed files scanned again in M ms.`; with `--json` the usual summary line). A failed re-scan prints its error and watching goes on. Only folders can be watched: a Git URL fails with `CLI-102`. Stop with Ctrl+C.

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

## `codeskimr open`

```
codeskimr open [--no-browser]
```

Opens the UI of the running local server in the browser. It asks the server for a one-time launch code, writes it to a private launch file (mode 0600), prints `Launch file: …` and starts the system browser with it, then waits until the code is used or has expired and deletes the file. `--no-browser` does not start a browser. Without a running server it stops with `API-601`. Details: `docs/server.md`.

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

## `codeskimr export`

```
codeskimr export <project> --html <file>
```

Writes the explorer, the dependency map, the usages and the search of a scanned project into one self-contained HTML file, without source text (`docs/export.md`). `<project>` is a project id or the folder or Git URL it was scanned from. The UI folder (`ui.dir`) must hold the built export page (`npm run build` in `web/`), otherwise the command stops with `EXP-201`; an unwritable file is `EXP-601`.

## Global options

| Option | Environment | Default |
|---|---|---|
| `--data-dir <dir>` | `CODESKIMR_DATA_DIR` | `%LOCALAPPDATA%\codeskimr`, `~/Library/Application Support/codeskimr`, `$XDG_DATA_HOME/codeskimr` or `~/.local/share/codeskimr` |
| `--languages <dir>` | `CODESKIMR_LANGUAGES_DIR` | `languages/` next to the executable, then in the current folder |
| `--config <file>` | `CODESKIMR_CONFIG` | `config.toml` in `%APPDATA%\codeskimr`, `~/Library/Application Support/codeskimr` or `$XDG_CONFIG_HOME/codeskimr` |
| `--set key=value` (repeatable) | | |

The data folder holds `codeskimr.db` and the Git cache `git/`. A config file named with the flag or the environment must exist (`CFG-102` otherwise); the default one is optional.

## Configuration chain

Blueprint section 14, in this order, later wins: the defaults of `config/default.toml`, the user config file, `.codeskimr.toml` in the scanned folder (folders only; a Git source is read after the fetch, which is not wired yet), then `--set key=value` flags (`--set scan.excluded_folders=["a","b"]`; the value is TOML). Unknown keys and invalid values fail with `CFG-103` and `CFG-101`. The architecture layers of a project are set with `[[layers]]` tables (`name`, `namespaces` = qualified-name prefixes, `.` or `::` as separator; the first layer that owns a name wins) in the project's `.codeskimr.toml`; they apply at the next scan (ADR 0031, this repository's own file is an example).

The package added two keys: `scan.parse_timeout_ms` (default 5000, `PRS-501` on expiry) and `scan.max_source_mb` (default 2048, `SRC-501` above it).

## Language of the texts

`CODESKIMR_LANG`, then `LC_ALL`, `LC_MESSAGES`, `LANG`: the first one that is set decides; a value starting with `de` selects German, everything else English. On Windows, where `LANG` is usually missing, set `CODESKIMR_LANG=de`.

## Exit codes

`0` success, `1` a command failed (`error[CODE]: text` on standard error) or `check` found violations, `2` invalid arguments (`CLI-101`, with the parser's explanation).

## Known limits

- References come from the level-1 resolver (`docs/references.md`).
- The language packs are read from a folder (see `--languages`); embedding them in the binary comes with the release package.
- `Ctrl+C` ends the process; cancelling a scan cleanly (`CancelToken`) is wired with the server.
- A user config error that is not TOML syntax (unreadable file) is reported as `CFG-102`.

## Caller identity

Both commands run as the local owner of the installation (`crates/app/src/principal.rs`), who holds every permission, so nothing changes for the user. The scanner and the MCP tools check the permission they need (`scan`, `read`) before they do anything; a caller without it gets `AUTH-401`. See blueprint 10.1 and ADR 0019.
