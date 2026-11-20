# Tool check on project load (ADR 0033, decision 8)

On every load of a project (registration, every full scan, reopening, `projects.tools`, `get_project`) CodeSkimr checks, for each language of the newest snapshot, whether the tools of level 2 exist and fit. Nothing is downloaded or installed.

## What is checked

The tools are the ones the `[indexer]` table of the language pack names (`docs/language-packs.md`): the indexer (for example `scip-dotnet`) and its toolchain (for example `dotnet`). There is one format for the runner and for this check: the check asks `IndexRunner::check` (`docs/level2-runner.md`) with the very same `IndexerSpec` that a level 2 run would use. Languages with the same indexer (TypeScript, JavaScript and tsx) are asked once, and a tool they share (`node`) is shown once.

States of a tool:

- `ready`: it is found and its version fits. The runner reports the version it read and the check shows it.
- `missing`: the program is not found in the `PATH` of the user.
- `not_runnable`: the program is found, but its version command fails (a non-zero exit or no answer in time): it crashes on start (scip-python 0.6.x on Windows: `Invalid regular expression` in its own code) or it is the `rustup` shim of a component that was never installed. `found` carries the line of its output that names the error, redacted and shortened.
- `too_old`: the version is lower than the strictest of the pack's `min_version` and the version the project requires.
- `too_new`: the version is higher than a file of the project lets through (see below). It is judged only for a tool whose version the runner read; an unreadable version stays `version_unknown`.
- `version_unknown`: its version command succeeded but printed no version that could be read.

A language is `level2_ready` when all its tools are ready, so a `too_new` tool counts as a problem: the build of the project would fail with it.

## How tools are started

The check starts nothing itself. `IndexRunner::check` (`codeskimr_indexer`) looks the program up in the `PATH` of the user only (entries inside the project are dropped), starts it with the `version_args` of the pack, never through a shell and in a neutral folder, with the cleared environment of the runner (no token of CodeSkimr reaches a tool), a time limit, and kills the whole process group. The first dotted number in the output is the version. Nothing is downloaded.

## What the project requires

A `[[indexer.toolchain]]` of the pack may carry a `requirement`: `global.json` (`sdk.version`) for `dotnet`, `rust-toolchain.toml` (`toolchain.channel`) for `cargo`, `.nvmrc` for `node`. The check raises the `min_version` of that tool to the version the project names when that is newer, then asks the runner. The file is looked for in the project root and in every folder below it (nested solutions and workspaces have their own `global.json` or `.nvmrc`): down to 6 levels, at most 32 files of one name and 2000 folders looked into (the search stops there), and not in `.git`, `node_modules`, `target`, `bin`, `obj`, `vendor`, `dist`, `.venv` and `venv`, which hold files of other projects. When several files ask for a version, the newest counts. A file is read only if it lies inside the project folder (no `..`, no symlink out of it) and is at most 64 KiB (`REQUIREMENT_MAX_BYTES`). `global.json` may hold comments (`//`, `/* */`) and trailing commas, as the .NET SDK accepts them. Python has none: `.python-version` is not read since the pack's toolchain is `node`. The leading number of the value counts (`v20.11` gives 20.11); a value that is not a number, such as `stable`, asks for nothing. The comparison is numeric per segment, a missing segment counts as zero.

## How new a tool may be (`rollForward`)

`global.json` can also limit from above: its `rollForward` (default `latestPatch`) says which SDKs the project accepts. The pack maps each policy to a ceiling counted from `sdk.version` (`roll_forward` of the requirement, `docs/language-packs.md`): `disable` the named version only, `patch` and `latestPatch` the same feature band (8.0.1xx), `feature` and `latestFeature` the same major and minor, `minor` and `latestMinor` the same major, `major` and `latestMajor` no limit. A tool that passed the minimum but lies beyond the ceiling of any `global.json` is `too_new`; the finding shows the version found and, as required, the version that file names. A policy value that the pack does not list limits nothing.

## Storage and outputs

The report is stored with the project (`tool_reports`, `tool_findings`, `docs/storage.md`) and replaced by each new check. It appears in the `codeskimr scan` summary (`docs/cli.md`), in the Projects screen (`docs/ui.md`, command `projects.tools`) and in the MCP tool `get_project` (`docs/mcp.md`). Purpose and install hint come from `locales/{de,en}/tools.ftl`, under the keys `tool-<program>-purpose` and `tool-<program>-install`; a test checks that every program of a pack has both in German and English. (`install_hint` of the pack is an English line for the log and the runner's errors; the UI uses the translated text.)

An incremental scan (a rescan in watch mode or in the UI after a file changed) does not check: the tools do not change when a file is saved, and watch mode times the rescan against the 300 ms budget of blueprint 16.1. The stored report stays, and the UI keeps showing it (the empty view of an unchecked scan never replaces a checked one).

A failing check never fails a scan: the error is logged and the project is shown as not checked.

## Known limits

- The check sees the version that `dotnet --version` prints in a neutral folder, which is the newest SDK installed. If several SDKs are installed, an older one that fits the project may exist; then `too_new` is shown although the build would work. The check does not list installed SDKs. Several `global.json` files with different pins can never all be green at once, even with two SDKs installed, because the check compares one found version with every file.
- `allowPrerelease` of `global.json` is not modelled: the version type holds no prerelease part, so a prerelease SDK is judged by its numbers only.
- A ceiling is judged on the version of a tool that the runner read; the level 2 run itself does not refuse a too new tool (it does not read the project files), the build fails as before.
- Requirement files are searched with a depth and a count limit (see above); a manifest deeper than 6 levels is not seen.
- The version flags and `min_version` of the indexers are confirmed by the spike of ADR 0033.
- **A Git project has no clone before its first scan.** The requirement files are read from the clone of the last scan (`GitCache`). At registration and in every check before the first full scan there is no clone, so no requirement is known and only the `min_version` of the pack counts. The check that every full scan runs after the scan finds the clone and reads the files, so the report is complete from the first scan on. The check does not fetch anything itself.

## The listing of all indexers (settings)

The settings screen has a section "Indexers for exact usages (level 2)" (mock 17, `docs/mocks/Indexers.dc.html`). It is fed by the command `tools.indexers` (REST `POST /api/commands/tools.indexers`, permission `scan`, no input), which needs no project: for every language pack with an `[indexer]` it asks `IndexRunner::check` with the pack's own `IndexerSpec` (the version of the project files is not read, so only the pack's `min_version` counts) and joins the findings of all packs into one list, one row per tool with the languages that need it. `IndexRunner::locate` adds the path where the tool was found on the `PATH` of the user (`path` in each row, `null` if it is not found). Nothing is stored; the screen asks when it opens and with "Check again". A tool of state `not_runnable` carries the line of its error in `found`. As everywhere in the tool check, nothing is installed and no project code runs.
