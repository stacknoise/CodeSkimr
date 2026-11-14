# Tool check on project load (ADR 0033, decision 8)

On every load of a project (registration, scan including rescan and watch, reopening) CodeSkimr checks, for each language of the newest snapshot, whether the tools of level 2 exist and fit. Nothing is downloaded or installed.

## What is checked

Each language pack lists its tools (`docs/language-packs.md`): an `indexer` (for example `scip-dotnet`) and a `toolchain` (for example `dotnet`). A tool shared by several languages (`node` for TypeScript, JavaScript and tsx) is started once and shown once.

States of a tool:

- `ready`: it ran, and its version fits.
- `missing`: the command was not found.
- `too_old`: the version is lower than the strictest of the pack's `min_version` and the version the project requires.
- `version_unknown`: it ran but no version could be read, it did not answer in time, or the project requires a version and none was read.

A language is `level2_ready` when all its tools are ready.

## The probe

`ProcessProbe` starts the command with the version arguments of the pack, never through a shell and never in the project folder (the working directory is the work folder of CodeSkimr). The environment is cleared; only the names in `tools.env_allowlist` are passed, so no token of CodeSkimr reaches a tool. Standard input is closed. After `tools.probe_timeout_ms` the process is killed and the tool counts as `version_unknown`; at most `tools.probe_max_output_bytes` of output are read. The first dotted number in standard output or error is the version.

## What the project requires

A pack may name a file in the project root: `global.json` (`sdk.version`), `rust-toolchain.toml` (`toolchain.channel`), `.python-version`, `.nvmrc`. The file is read only if it lies inside the project folder (no `..`, no symlink out of it) and is at most `tools.requirement_max_bytes`. The leading number of the value counts (`v20.11` gives 20.11); a value that is not a number, such as `stable`, asks for nothing. The comparison is numeric per segment, a missing segment counts as zero.

## Storage and outputs

The report is stored with the project (`tool_reports`, `tool_findings`, `docs/storage.md`) and replaced by each new check. It appears in the `codeskimr scan` summary (`docs/cli.md`), in the Projects screen (`docs/ui.md`, command `projects.tools`) and in the MCP tool `get_project` (`docs/mcp.md`). Purpose and install hint come from `locales/{de,en}/tools.ftl`.

A failing check never fails a scan: the error is logged and the project is shown as not checked.

## Known limits

- `global.json` is read as strict JSON; comments or trailing commas give no requirement.
- Requirement files are read only from the project root, not from nested solutions.
- `rollForward` of `global.json` is not modelled: installed must be at least the required version.
- On timeout only the started process is killed, not its children.
- The version flags and `min_version` of the indexers are confirmed by the spike of ADR 0033.
- A Git project reads its requirement files from the clone of the last scan.
