# CodeSkimr 0.9.0-preview.1 (preview)

First version to try out. **This is not 1.0.** Built for Windows (x86_64) and Linux (x86_64); macOS follows later (issue #131).

## Install

Windows: unpack the `.zip`, run `.\codeskimr.exe` in the folder (keep `languages\` next to it; SmartScreen may warn because the program is not signed yet). Linux: `tar xzf` the `.tar.gz`, run `./codeskimr` (needs a Secret Service such as GNOME Keyring or KWallet, otherwise `SEC-601`). The program starts the local server and opens the browser. Compare the SHA-256 with the `.sha256` file if you like.

Manual: `docs/manual/en/`. Data: `%LOCALAPPDATA%\codeskimr` (Windows), `~/.local/share/codeskimr` (Linux); delete the folder to reset.

## Known limits

- Usages are not compiler-exact everywhere. Measured on the test corpora: Rust and TypeScript 100 %, Python 99 %, Java 88 %, C# 34 % (level 2, bug in scip-dotnet). Read usages as an approximation for languages below 99 %.
- Level 2 (exact indexers) runs foreign tools in your project and needs your explicit consent; you install the indexers yourself (`indexers.md`).
- Milestones M4/M5 are not accepted yet; screens and reports may change. The database format may change before 1.0: delete the data folder and scan again.
- No macOS package, no signature, no installer (with 1.0).

Feedback as an issue in `stacknoise/CodeSkimr`.
