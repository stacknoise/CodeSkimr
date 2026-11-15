# Installation

## State of the delivery

CodeSkimr is delivered as **one single file per system** (Windows x86_64, Linux x86_64, macOS Apple Silicon): `codeskimr-<version>-<system>` (with `.exe` on Windows). The UI and the language packs are inside the file; there is no installer. You find the files on the release page of the repository `stacknoise/CodeSkimr`, each with a `.sha256` file. Alternatively you build CodeSkimr from source (see below).

### Download, check, start

1. Download the file and its `.sha256` and compare the checksum: Windows `Get-FileHash .\codeskimr-<version>-windows-x86_64.exe -Algorithm SHA256`, Linux `sha256sum -c codeskimr-<version>-linux-x86_64.sha256`, macOS `shasum -a 256 -c codeskimr-<version>-macos-aarch64.sha256`.
2. Optionally check the origin: `gh attestation verify <file> --repo stacknoise/CodeSkimr` shows that GitHub built the file from this repository.
3. Start it. Linux and macOS: run `chmod +x <file>` first. Then `./codeskimr-<version>-<system> serve`; on Windows `.\codeskimr-<version>-windows-x86_64.exe serve`. You may rename the file to `codeskimr` and put it in a folder on your search path.

The files are **not signed yet** (signing is the first task after 1.0), so the operating system warns on the first start:

- **Windows (SmartScreen):** "Windows protected your PC" → "More info" → "Run anyway".
- **macOS (Gatekeeper):** a message that the file cannot be verified. Run `xattr -d com.apple.quarantine <file>` once in the terminal, then start it (or choose "Open Anyway" in System Settings under "Privacy & Security").
- **Linux:** no warning; a Secret Service is required (see below).

## What you need

- The Rust toolchain (stable, installed with [rustup](https://rustup.rs); the repository pins the channel in `rust-toolchain.toml`).
- Node.js and npm, to build the browser UI in `web/`.
- A **keystore** of the operating system: Windows Credential Manager, macOS Keychain, or on Linux a Secret Service (for example GNOME Keyring or KWallet). CodeSkimr keeps its installation token and your Git credentials there. Without a usable keystore `codeskimr serve` stops with `SEC-601`; this mostly concerns headless Linux machines.
- Git is **not** required: CodeSkimr fetches Git sources itself.

## Build

```
git clone https://github.com/stacknoise/CodeSkimr
cd CodeSkimr
cd web && npm ci && npm run build && cd ..
cargo build --release --locked
```

The program is `target/release/codeskimr` (`codeskimr.exe` on Windows). The built UI is in `web/dist` and is embedded when building, which is why `web/` comes first. Node.js 22 (`.node-version`). The language packs from `languages/` are embedded in the program as well; on the first start it writes them to the data folder (`languages/<id>/`). A `languages/` folder next to the program takes precedence if it contains at least one `<language>/language.toml` (a `languages/` folder in the current folder is not used), and so do the option `--languages` and `CODESKIMR_LANGUAGES_DIR`; this lets you change packs without rebuilding.

## First start

```
codeskimr
```

starts the server and opens the browser; the UI is part of the program (build `web/` before `cargo build`). The two steps also work on their own:

```
codeskimr serve
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
| language packs of the program `languages/<id>/` (written on the first start, may be deleted) | in the data folder |
| user configuration `config.toml` (optional) | Windows `%APPDATA%\codeskimr`, macOS `~/Library/Application Support/codeskimr`, Linux `$XDG_CONFIG_HOME/codeskimr`; change with `--config` or `CODESKIMR_CONFIG` |
| theme and language of the UI `preferences.json` | in the data folder |
| installation token, Git credentials | the keystore of the operating system, service name `codeskimr` |

## Configuration

Settings are applied in this order, later wins: the defaults built into the program, your user configuration file, the file `.codeskimr.toml` in the scanned folder, and `--set key=value` on the command line (the value is TOML, for example `--set scan.excluded_folders=["a","b"]`). A key that does not exist is refused with `CFG-103`, a value outside its range with `CFG-101`. The complete list of keys with explanations is `config/default.toml`.

## Uninstalling

Delete the program, the data folder and, if you want, the entries named `codeskimr` in your keystore.
