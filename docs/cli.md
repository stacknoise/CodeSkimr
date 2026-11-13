# CLI and composition root (WP-25)

Implements the first part of blueprint section 10.2: `codeskimr scan` and `codeskimr mcp`. The other commands (`serve`, `export`, `check`, `codes`, and the bare `codeskimr` that starts the app) follow with their packages.

## Structure

- `codeskimr_cli` knows no adapter. It defines the commands with clap (help texts come from `locales/{de,en}/cli.ftl`), turns `argv` into an `Invocation` and renders the summary and the error line.
- `codeskimr` (`crates/app`) is the composition root. `main.rs` only starts `run::execute`. `wiring.rs` is the only place that creates adapters: `SqliteStore`, `SourceFetcher`, `WalkDiscovery`, `TreeSitterAnalyzer` (with fingerprints from `config.similarity`, closes #41) and the `Scanner`. `scan.rs` and `mcp.rs` are the two commands.

## `codeskimr scan <path|git-url>`

```
codeskimr scan <path|git-url> [--branch <name>] [--json]
```

1. The argument is an existing folder (made absolute) or a Git URL (`https://`, `http://`, `ssh://`, `git://`, `git@…`, or ending in `.git`). A Git URL with secrets in it (user info for HTTP(S), a password anywhere, a query or a fragment, in any letter case) is refused with `PRJ-101` and a second line that names the reason: they would be stored in the database and printed (blueprint 12). A bare SSH user name (`git@host:org/repo`) is fine. `--branch` only applies to Git sources and is ignored for folders. On Windows the `\\?\` prefix of the absolute path is removed (UNC paths keep it). The branch defaults to `main`.
2. The source is registered as a project (the same source is registered once; scanning again reuses the project).
3. One full scan runs: fetch, discover, analyze, resolve, measure, persist. A new snapshot is written; `--json` prints `{project, snapshot, files, symbols, references, seconds}` on one line, otherwise a two-line summary in the language of the user.
4. The mode is always full. The incremental mode is not complete yet (issues #20 and #29).

## `codeskimr mcp`

```
codeskimr mcp [--project <folder>]
```

Serves the three tools of WP-26 over stdin and stdout for the project in `--project` (default: the current folder). The folder must have been scanned before; otherwise the command stops with `PRJ-201` (not registered) or `MCP-203` (registered, but no snapshot). Nothing but the protocol is written to standard output; errors go to standard error.

## Global options

| Option | Environment | Default |
|---|---|---|
| `--data-dir <dir>` | `CODESKIMR_DATA_DIR` | `%LOCALAPPDATA%\codeskimr`, `~/Library/Application Support/codeskimr`, `$XDG_DATA_HOME/codeskimr` or `~/.local/share/codeskimr` |
| `--languages <dir>` | `CODESKIMR_LANGUAGES_DIR` | `languages/` next to the executable, then in the current folder |
| `--config <file>` | `CODESKIMR_CONFIG` | `config.toml` in `%APPDATA%\codeskimr`, `~/Library/Application Support/codeskimr` or `$XDG_CONFIG_HOME/codeskimr` |
| `--set key=value` (repeatable) | | |

The data folder holds `codeskimr.db` and the Git cache `git/`. A config file named with the flag or the environment must exist (`CFG-102` otherwise); the default one is optional.

## Configuration chain

Blueprint section 14, in this order, later wins: the defaults of `config/default.toml`, the user config file, `.codeskimr.toml` in the scanned folder (folders only; a Git source is read after the fetch, which is not wired yet), then `--set key=value` flags (`--set scan.excluded_folders=["a","b"]`; the value is TOML). Unknown keys and invalid values fail with `CFG-103` and `CFG-101`.

The package added two keys: `scan.parse_timeout_ms` (default 5000, `PRS-501` on expiry) and `scan.max_source_mb` (default 2048, `SRC-501` above it).

## Language of the texts

`CODESKIMR_LANG`, then `LC_ALL`, `LC_MESSAGES`, `LANG`: the first one that is set decides; a value starting with `de` selects German, everything else English. On Windows, where `LANG` is usually missing, set `CODESKIMR_LANG=de`.

## Exit codes

`0` success, `1` a command failed (`error[CODE]: text` on standard error), `2` invalid arguments (`CLI-101`, with the parser's explanation).

## Known limits

- The reference resolver and the metrics calculator do not exist yet (blueprint 9, ADR 0015, issues #39 and #44). `crates/app/src/placeholders.rs` stands in for both: a scan stores files and symbols but no references and no metrics, and `get_symbol` shows none. The placeholders are replaced in that one file.
- The language packs are read from a folder (see `--languages`); embedding them in the binary comes with the release package.
- `Ctrl+C` ends the process; cancelling a scan cleanly (`CancelToken`) is wired with the server.
- A user config error that is not TOML syntax (unreadable file) is reported as `CFG-102`.

## Caller identity

Both commands run as the local owner of the installation (`crates/app/src/principal.rs`), who holds every permission, so nothing changes for the user. The scanner and the MCP tools check the permission they need (`scan`, `read`) before they do anything; a caller without it gets `AUTH-401`. See blueprint 10.1 and ADR 0019.
