# Language indexers (level 2)

Level 2 makes usages **exact**: instead of guessing which `Save()` a call means, CodeSkimr takes the answer of the language's own tool, which knows the types, and marks those usages **verified**. This chapter explains what you install, how you switch it on and what happens when you scan.

> **State:** level 2 is built for C#, TypeScript, JavaScript, Python, Rust, Java and Kotlin and is **off by default**. **No language is formally accepted yet.** The measurements on public sample projects (150 symbols each, `docs/accuracy.md` in the source repository) show how far "verified" can be trusted today: Rust and Java 100 % precision and recall, TypeScript 99 % precision and 100 % recall; Python finds 99 % of the usages but lists extra hits for fields (precision 63 %); for C# level 2 misses most usages that level 1 finds (recall 34 %, a known problem of the C# indexer `scip-dotnet` with overloaded extension methods). Kotlin has not been measured at all. Languages below 99 % are approximations. "Verified" means: this usage is exact. It does **not** mean the list is complete, so a verified list can still lack usages. The decision record, ADR 0033, is accepted (status "accepted").

## Why

Without type information, finding the usages of a name is guesswork. CodeSkimr therefore imports the result of a compiler-grade indexer (a SCIP index) instead of rebuilding a compiler. For every file that the index covers, its references replace the guesses and are marked **verified**. Files without an index keep the name-based result, marked **unchecked** ([Usages](usages.md#certainty-verified-unchecked-presumed)).

## You install the indexers, CodeSkimr checks them

CodeSkimr **never downloads or installs** an indexer and never runs a package from the network (no `npx`). You install the indexer for each language you use, the way you install the compiler. The reason: an indexer needs your project's own dependencies and a matching SDK anyway (package restore, `node_modules`, a Python environment, the Gradle or Maven build), and only you can provide those.

| Language | Indexer | Also needed |
|---|---|---|
| C# | `scip-dotnet` (`dotnet tool install --global scip-dotnet`) | the .NET SDK the project asks for (`global.json`), restored packages |
| TypeScript, JavaScript | `scip-typescript` (`npm install --global @sourcegraph/scip-typescript`) | Node.js (`.nvmrc`), installed `node_modules` |
| Python | `scip-python` (`npm install --global @sourcegraph/scip-python`) | Node.js, the Python environment of the project |
| Rust | `rust-analyzer` (`rustup component add rust-analyzer`) | Cargo and the toolchain the project asks for (`rust-toolchain.toml`) |
| Java, Kotlin | `scip-java` (see its installation guide) | a JDK, the Maven or Gradle build of the project |

Go, C, C++, PHP and Ruby (basic tier) have no level 2; their usages stay unchecked. Every indexer has a minimum version, which the check below reports.

The indexer is found on the `PATH` of your user. Folders inside the project are dropped from the `PATH`, so a repository cannot plant its own `dotnet` or `node`. A virtual environment inside the project (`.venv`) is therefore **not** found for `scip-python`: activate it outside the project or install the tools globally.

## What CodeSkimr shows you: the tool check

Whenever a project is loaded (added, fully scanned, reopened), CodeSkimr checks per language whether the indexer and its toolchain are there and fit. Nothing is installed and no project code runs. The result is stored with the project and shown:

- in the **Projects screen**: a warning with every missing tool, why it is needed and how to install it, the languages that are ready, a button "Check again", and a pill "N tools missing" in the table;
- in the summary of `codeskimr scan` (and in `--json` as `tools`);
- over MCP with the tool `get_project`.

A tool is `ready`, `missing` (not on the `PATH`), `not_runnable` (found, but its version command fails, as with the `rustup` shim of a component that was never added, or an indexer that crashes on start; the panel shows the line of its error), `too_old` (below the minimum version or the version the project asks for in `global.json`, `.nvmrc` or `rust-toolchain.toml`), `too_new` (newer than the `rollForward` rule in `global.json` lets through, for example SDK 9 although the project names 8.0 and allows nothing above it) or `version_unknown` (it ran but no version could be read). A missing tool **never stops a scan**: you get level 1, marked unchecked. A quick re-scan after a file change does not repeat the check.

## Switching it on

Running an indexer runs the build tools of your project, which is project code. So it is opt-in, in two steps, and only you, the local owner, can do them:

1. **Enable it** in your user configuration (`config.toml`): 

   ```toml
   [level2]
   enabled = true
   ```

2. **Confirm it once per project:**

   ```
   codeskimr level2 confirm <project>
   ```

   `<project>` is the project id or the folder or Git address it was scanned from. The command needs a terminal. It prints what level 2 runs and with which rights (it can read what you can read and use the network, there is no sandbox), shows the name, location and branch of the project, and starts only if you type the **name of the project**. A Git address resolves only the default branch of that repository; for a project at another branch use its id. Without a terminal it fails with `SEC-403`.

   `codeskimr level2 revoke <project>` forbids it again at once.

**From the Projects screen.** If the switch in step 1 is on, the Projects screen shows the state of level 2 per project (`on`, `off`, `not enabled`) and a switch. "Switch on" asks the program to open a console window that runs `codeskimr level2 confirm` for this project (Windows and macOS). **You still type the name of the project in that window: the browser never gives the consent.** The window says that it was opened by the page in your browser; if you did not click, close it. The screen waits and shows the new state when you have confirmed. On Linux, in a remote session, or if the window cannot be opened, the screen shows the command to copy into a terminal. "Switch off" takes the confirmation back at once, without a terminal; results that level 2 already checked stay until the next scan, which then runs level 1 only. If step 1 is missing, the screen says so and shows the two lines for `config.toml` in the data folder; CodeSkimr never changes that file itself. After a window was opened there is a pause of `level2.confirm_window_cooldown_seconds` (default 30) before another one opens (`API-504`); the screen asks for the state every `ui.level2_poll_seconds` (2) for at most `ui.level2_poll_max_seconds` (300).

The confirmation is bound to the project's name, location and branch, **not to a commit**: if the project is pointed at another folder or repository, it is unconfirmed again, but a `git pull` at the same place stays confirmed. A `level2` setting in the project's own `.codeskimr.toml` is **ignored**, so a repository you scan cannot switch on running its own build scripts or raise a limit.

## When it runs

- **`codeskimr scan`** runs level 2 after level 1 is stored: for each language with an indexer it checks permission and tools, runs the indexer, imports the index and replaces the references per file. It prints one line per language, for example "Level 2: C# checked (120 files)" or "Level 2: C# skipped (not enabled and confirmed for this project)". Other reasons: indexer or toolchain missing or too old, every file changed since the scan, scan cancelled, failed with an error code. If only some files changed since the scan, the same line says so and they stay at level 1. If the index knows symbols in folders that the scan skips, a second line says how many uses of them were set aside ("uses of code in folders the scan skips, not counted"); they are not held against the index.
- Watch mode (`--watch`) runs level 2 with its **first** scan only; later re-scans are level 1.
- **Scans over MCP never run level 2**, even after you confirmed it. A scan or rescan in the browser runs level 2 after level 1 in the background, but only for a project you confirmed in the terminal and only if the switch is on; "Level 1 only" skips it. The browser cannot confirm anything. Only one run goes on at a time, and after a run the same project waits 60 seconds before the browser starts another (`level2.max_parallel_runs`, `level2.ui_cooldown_seconds`, set in your user configuration only). Cancel stops the indexer and keeps level 1.
- Level 1 is complete and usable before level 2 starts; the browser and MCP do not wait for the indexers.

Whatever goes wrong, the scan has finished and level 1 stays. A failing indexer, a timeout, a corrupt index or too low a mapping are reported per language and never break the scan.

## Limits

Every run is limited; an indexer that exceeds a limit is stopped together with all its child processes and level 1 stays. The values are in the file `config/default.toml` of the source repository, section `[level2]` (you can set them in your user configuration):

| Setting | Default | Error |
|---|---|---|
| `timeout_seconds` | 900 | `RES-502` |
| `max_memory_mb` (whole process tree) | 4096 | `RES-504` |
| `max_output_bytes` (console and files) | 512 MiB | `RES-503` |
| `max_index_bytes` (largest index that is read) | 256 MiB | `RES-501` |
| `min_mapped` (share of project occurrences that must map) | 0.5 | `RES-302` |
| `max_created_symbols` (most symbols that level 2 creates itself, e.g. types a macro generates) | 100000 | `RES-303` |

The indexer gets a cleared environment with an allow list (no token or secret of CodeSkimr reaches it), runs in its own process group, and its output is redacted before it is logged. **The network is not blocked**, because restoring packages needs it; this is the main residual risk and the reason for the confirmation.

## After a change

An index counts for a file only while its content is the one at indexing time. After a run CodeSkimr compares the files with the disk again; files you changed in the meantime keep level 1 and are reported as changed. A file you edit later falls back to level 1 (unchecked) until level 2 runs again, with the next `codeskimr scan`.

## Errors

`RES-201` indexer or toolchain missing or too old, `RES-501` index too large, `RES-502` timeout, `RES-503` output too large, `RES-504` memory limit, `RES-601` the indexer could not be started or failed, `RES-101` the index is corrupt, `RES-303` level 2 would create too many symbols of its own, `RES-302` too few occurrences map onto your symbols, `SEC-301` level 2 is not enabled and confirmed, `SEC-403` the caller may not confirm or start indexers, `PRJ-303` the scan was cancelled. In all of them level 1 stays. See [Error codes](errors.md).
