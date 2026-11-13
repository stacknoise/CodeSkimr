# Installation

## State of the delivery

Installers and single executables for Windows, Linux and macOS are part of the 1.0 release (milestone M7) and are not published yet. Until then you build CodeSkimr from source. This chapter describes that path; it will be replaced by the installer instructions with 1.0.

## What you need

- The Rust toolchain (stable, installed with [rustup](https://rustup.rs); the repository pins the channel in `rust-toolchain.toml`).
- Node.js and npm, to build the browser UI in `web/`.
- A **keystore** of the operating system: Windows Credential Manager, macOS Keychain, or on Linux a Secret Service (for example GNOME Keyring or KWallet). CodeSkimr keeps its installation token and your Git credentials there. Without a usable keystore `codeskimr serve` stops with `SEC-601`; this mostly concerns headless Linux machines.
- Git is **not** required: CodeSkimr fetches Git sources itself.

## Build

```
git clone https://github.com/stacknoise/CodeSkimr
cd CodeSkimr
cargo build --release --locked
cd web && npm ci && npm run build && cd ..
```

The program is `target/release/codeskimr` (`codeskimr.exe` on Windows). The built UI is in `web/dist`. The language packs are read from the `languages/` folder, first next to the program, then in the current folder (option `--languages` or `CODESKIMR_LANGUAGES_DIR` names another folder). Embedding them in the program comes with the release package.

## First start

```
codeskimr --set ui.dir=web/dist serve
```

in one terminal prints `Listening on http://127.0.0.1:7421` and keeps running until Ctrl+C. In a second terminal:

```
codeskimr open
```

opens the browser. See [Projects and scan](projects-and-scan.md) for what to do next. On the first `serve` CodeSkimr creates the installation token in the keystore; you never see or type it.

If port 7421 is taken, the server stops with `API-301`; set another one with `--set server.port=7500`. The address cannot be changed: the server only listens on the loopback address.

## Where data lives

| What | Where |
|---|---|
| database `codeskimr.db` and the cache of Git sources `git/` | Windows `%LOCALAPPDATA%\codeskimr`, macOS `~/Library/Application Support/codeskimr`, Linux `$XDG_DATA_HOME/codeskimr` or `~/.local/share/codeskimr`; change with `--data-dir` or `CODESKIMR_DATA_DIR` |
| user configuration `config.toml` (optional) | Windows `%APPDATA%\codeskimr`, macOS `~/Library/Application Support/codeskimr`, Linux `$XDG_CONFIG_HOME/codeskimr`; change with `--config` or `CODESKIMR_CONFIG` |
| installation token, Git credentials | the keystore of the operating system, service name `codeskimr` |

## Configuration

Settings are applied in this order, later wins: the defaults built into the program, your user configuration file, the file `.codeskimr.toml` in the scanned folder, and `--set key=value` on the command line (the value is TOML, for example `--set scan.excluded_folders=["a","b"]`). A key that does not exist is refused with `CFG-103`, a value outside its range with `CFG-101`. The complete list of keys with explanations is `config/default.toml`.

## Uninstalling

Delete the program, the data folder and, if you want, the entries named `codeskimr` in your keystore.
