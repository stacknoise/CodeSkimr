# CodeSkimr 0.9.0-preview.1 (preview)

First version to try out. **This is not 1.0.** Built for Windows (x86_64), Linux (x86_64) and macOS (Apple Silicon).

## Install

One single file per system (UI and language packs are inside) plus a `.sha256` file.

Windows: run `.\codeskimr-0.9.0-preview.1-windows-x86_64.exe` (SmartScreen warns because the file is not signed yet: "More info", then "Run anyway"). Linux: `chmod +x codeskimr-0.9.0-preview.1-linux-x86_64`, then run it (needs a Secret Service such as GNOME Keyring or KWallet, otherwise `SEC-601`). macOS: `chmod +x codeskimr-0.9.0-preview.1-macos-aarch64`, then `xattr -d com.apple.quarantine codeskimr-0.9.0-preview.1-macos-aarch64` (the file is not signed yet) and run it. The program starts the local server and opens the browser.

Check the sum: `Get-FileHash <file> -Algorithm SHA256` (Windows), `sha256sum -c <file>.sha256` (Linux), `shasum -a 256 -c <file>.sha256` (macOS). Check the origin (optional): `gh attestation verify <file> --repo stacknoise/CodeSkimr`.

Manual: `docs/manual/en/`. Data: `%LOCALAPPDATA%\codeskimr` (Windows), `~/.local/share/codeskimr` (Linux), `~/Library/Application Support/codeskimr` (macOS); delete the folder to reset.

## Known limits

- Usages are not compiler-exact everywhere. Measured on the test corpora: Rust and TypeScript 100 %, Python 99 %, Java 88 %, C# 34 % (level 2, bug in scip-dotnet). Read usages as an approximation for languages below 99 %.
- Level 2 (exact indexers) runs foreign tools in your project and needs your explicit consent; you install the indexers yourself (`indexers.md`).
- Milestones M4/M5 are not accepted yet; screens and reports may change. The database format may change before 1.0: delete the data folder and scan again.
- No signature (Windows, macOS; first task after 1.0) and no installer: one single file per system.

Feedback as an issue in `stacknoise/CodeSkimr`.
