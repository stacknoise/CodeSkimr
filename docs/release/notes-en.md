# CodeSkimr 1.0.0

The first stable version. Built for Windows (x86_64), Linux (x86_64) and macOS (Apple Silicon).

## Install

One single file per system (UI and language packs are inside) plus a `.sha256` file.

Windows: run `.\codeskimr-1.0.0-windows-x86_64.exe` (SmartScreen warns because the file is not signed yet: "More info", then "Run anyway"). Linux: `chmod +x codeskimr-1.0.0-linux-x86_64`, then run it (needs a Secret Service such as GNOME Keyring or KWallet, otherwise `SEC-601`). macOS: `chmod +x codeskimr-1.0.0-macos-aarch64`, then `xattr -d com.apple.quarantine codeskimr-1.0.0-macos-aarch64` (the file is not signed yet) and run it. The program starts the local server and opens the browser. **Keep that window open as long as you use CodeSkimr:** closing it ends the server, and the UI in the browser then shows "Not connected".

Check the sum: `Get-FileHash <file> -Algorithm SHA256` (Windows), `sha256sum -c <file>.sha256` (Linux), `shasum -a 256 -c <file>.sha256` (macOS). Check the origin (optional): `gh attestation verify <file> --repo stacknoise/CodeSkimr`.

Manual: `docs/manual/en/`. Data: `%LOCALAPPDATA%\codeskimr` (Windows), `~/.local/share/codeskimr` (Linux), `~/Library/Application Support/codeskimr` (macOS); delete the folder to reset.

## Known limits

<!-- DRAFT: the statement on accuracy is the product owner's decision (recommendation: state it like this). Numbers: docs/accuracy.md, docs/acceptance/level2.md; check again before publishing. -->
- Usages are not compiler-exact everywhere. Measured with level 2 (exact indexers) on the test corpora: Rust, TypeScript and Java 99 % and above. **Python, C# and Kotlin are below that and are an approximation:** Python finds almost everything (recall 99 %) but also shows wrong hits (precision 63 %); for C# the indexer partly fails (bug in scip-dotnet, reported), Kotlin cannot be measured. The UI marks these languages.
- Level 2 runs foreign tools in your project and needs your explicit consent; you install the indexers yourself (`indexers.md`).
- Very large projects take longer: a full scan of 100,000 lines takes one to two seconds depending on the language (known limit, improvement after 1.0).
- No signature (Windows, macOS; first task after 1.0) and no installer: one single file per system.

Feedback as an issue in `stacknoise/CodeSkimr`.
