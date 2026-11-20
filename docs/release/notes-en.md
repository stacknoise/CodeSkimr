# CodeSkimr 1.0.0

The first stable version. Built for Windows (x86_64), Linux (x86_64) and macOS (Apple Silicon).

## Install

One single file per system (UI and language packs are inside) plus a `.sha256` file.

Windows: run `.\codeskimr-1.0.0-windows-x86_64.exe` (SmartScreen warns because the file is not signed yet: "More info", then "Run anyway"). Linux: `chmod +x codeskimr-1.0.0-linux-x86_64`, then run it (needs a Secret Service such as GNOME Keyring or KWallet, otherwise `SEC-601`). macOS: `chmod +x codeskimr-1.0.0-macos-aarch64`, then `xattr -d com.apple.quarantine codeskimr-1.0.0-macos-aarch64` (the file is not signed yet) and run it. The program starts the local server and opens the browser. On Windows and macOS it then runs in the system tray: the console window closes, and the tray icon has the menu "Open CodeSkimr", "Show log" and "Quit CodeSkimr". On Linux (and with `--console`) **keep the window open as long as you use CodeSkimr:** closing it ends the server, and the UI in the browser then shows "Not connected".

Check the sum: `Get-FileHash <file> -Algorithm SHA256` (Windows), `sha256sum -c <file>.sha256` (Linux), `shasum -a 256 -c <file>.sha256` (macOS). Check the origin (optional): `gh attestation verify <file> --repo stacknoise/CodeSkimr`.

Manual: `docs/manual/en/`. Data: `%LOCALAPPDATA%\codeskimr` (Windows), `~/.local/share/codeskimr` (Linux), `~/Library/Application Support/codeskimr` (macOS); delete the folder to reset.

## Known limits

- Usages are not compiler-exact everywhere. Measured with level 2 (exact indexers) on the test corpora: Rust and Java 99 % and above; TypeScript has precision 99 % but, since import lines are no usages, finds 92 % of what the TypeScript language service lists (the missing ones are import lines). **Python, C# and Kotlin are below that and are an approximation:** Python finds almost everything (recall 99 %) but also shows wrong hits (precision about 67 %); for C# the indexer scip-dotnet attributes overloads wrongly (recall 34 %, precision 60 %, see `docs/accuracy.md`); Kotlin cannot be measured (no reference tool).
- Level 2 runs foreign tools in your project and needs your explicit consent; you install the indexers yourself (`indexers.md`).
- **Large projects:** the goal "100,000 lines in under a second" is not met everywhere. Large C# projects take about 1.4 s per 100,000 lines (Newtonsoft.Json, 197,000 lines: about 3 s, measured in a Linux container); the improvement (#410) comes after 1.0.
- **HTML export:** the file holds only the direct usages of each symbol (no depth select in the page); walks over several steps are only in the app. The size grows about linearly with the project (whole CodeSkimr repository, about 7,400 symbols: 64 MB).
- **.NET preview SDK:** with a .NET preview SDK and `allowPrerelease` the tool check can show a wrong "too new" hint (known limit of the tool check, see #379).
- **macOS:** the version is built for macOS (Apple Silicon) and started in a clean environment automatically (CI), but it was not tested by hand on a Mac. Gatekeeper behaviour and the folder dialog are unchecked there.
- No signature (Windows, macOS; first task after 1.0) and no installer: one single file per system.

Feedback as an issue in `stacknoise/CodeSkimr`.
