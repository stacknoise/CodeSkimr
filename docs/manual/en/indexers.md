# Language indexers (level 2)

Level 2 makes usages **exact**: instead of guessing which `Save()` a call means, CodeSkimr takes the answer of the language's own tool, which knows the types, and marks those usages **verified**. This chapter explains what you install, how you switch it on and what happens when you scan.

> **State:** level 2 is built for C#, TypeScript, JavaScript, Python, Rust, Java and Kotlin and is **off by default**. It has been measured on sample projects (`docs/accuracy.md`), but it is not formally accepted yet, C# in particular. Treat verified usages as much better than unchecked ones, not as a guarantee. The decision record, ADR 0033, still has the status "proposed", so details can change.

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

A tool is `ready`, `missing` (not on the `PATH`), `too_old` (below the minimum version or the version the project asks for in `global.json`, `.nvmrc` or `rust-toolchain.toml`) or `version_unknown` (it ran but no version could be read). A missing tool **never stops a scan**: you get level 1, marked unchecked. A quick re-scan after a file change does not repeat the check.

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

   `<project>` is the project id or the folder or Git address it was scanned from. The command needs a terminal. It prints what level 2 runs and with which rights (it can read what you can read and use the network, there is no sandbox), shows the name, location and branch of the project, and starts only if you type the **name of the project**. Without a terminal it fails with `SEC-403`.

   `codeskimr level2 revoke <project>` forbids it again at once.

The confirmation is bound to the project's name, location and branch, **not to a commit**: if the project is pointed at another folder or repository, it is unconfirmed again, but a `git pull` at the same place stays confirmed. A `level2` setting in the project's own `.codeskimr.toml` is **ignored**, so a repository you scan cannot switch on running its own build scripts or raise a limit.

## When it runs

- **`codeskimr scan`** runs level 2 after level 1 is stored: for each language with an indexer it checks permission and tools, runs the indexer, imports the index and replaces the references per file. It prints one line per language: "Level 2: … checked" or "skipped (reason)", for example not confirmed, tools missing, files changed, cancelled, or an error code.
- Watch mode (`--watch`) runs level 2 with its **first** scan only; later re-scans are level 1.
- **Scans started from the browser or over MCP never run level 2**, even after you confirmed it. MCP, REST tokens and the browser cannot start the build tools of a project.
- Level 1 is complete and usable before level 2 starts; the browser and MCP do not wait for the indexers.

Whatever goes wrong, the scan has finished and level 1 stays. A failing indexer, a timeout, a corrupt index or too low a mapping are reported per language and never break the scan.

## Limits

Every run is limited; an indexer that exceeds a limit is stopped together with all its child processes and level 1 stays. The values are in `config/default.toml`, section `[level2]`:

| Setting | Default | Error |
|---|---|---|
| `timeout_seconds` | 900 | `RES-502` |
| `max_memory_mb` (whole process tree) | 4096 | `RES-504` |
| `max_output_bytes` (console and files) | 512 MiB | `RES-503` |
| `max_index_bytes` (largest index that is read) | 256 MiB | `RES-501` |
| `min_mapped` (share of project occurrences that must map) | 0.5 | `RES-302` |

The indexer gets a cleared environment with an allow list (no token or secret of CodeSkimr reaches it), runs in its own process group, and its output is redacted before it is logged. **The network is not blocked**, because restoring packages needs it; this is the main residual risk and the reason for the confirmation.

## After a change

An index counts for a file only while its content is the one at indexing time. After a run CodeSkimr compares the files with the disk again; files you changed in the meantime keep level 1 and are reported as changed. A file you edit later falls back to level 1 (unchecked) until level 2 runs again, with the next `codeskimr scan`.

## Errors

`RES-201` indexer or toolchain missing or too old, `RES-501` index too large, `RES-502` timeout, `RES-503` output too large, `RES-504` memory limit, `RES-601` the indexer could not be started or failed, `RES-101` the index is corrupt, `RES-302` too few occurrences map onto your symbols, `SEC-301` level 2 is not enabled and confirmed, `SEC-403` the caller may not confirm or start indexers, `PRJ-303` the scan was cancelled. In all of them level 1 stays. See [Error codes](errors.md).
