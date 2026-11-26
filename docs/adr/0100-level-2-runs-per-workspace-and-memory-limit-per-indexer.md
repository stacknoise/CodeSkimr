# 0100 – Level 2 runs per workspace and a memory limit per indexer

Status: accepted (follow-up of ADR 0033 and ADR 0087; asked for by the product owner in issues #738 and #739, 2026-11-26)

## Context

Measured with release candidate 13 on `vscode-main` (16,717 files, Windows): the level 2 indexers of Rust, C# and TypeScript all ended with `RES-601`, also with a memory limit of 16 GB (issue #738). The log gave three different causes:

- rust-analyzer: `no projects`. The only `Cargo.toml` lies in `cli/`, and rust-analyzer finds its workspace from the working folder or above, never below it.
- scip-dotnet: no solution or project file in the working folder. Same shape.
- scip-typescript: Node ran out of heap at about 4 GB, whatever `level2.max_memory_mb` said. Release 1.0 sets `--max-old-space-size` from the limit, but one global limit has to serve the small indexers and the one that needs three times as much (issue #739).

## Options

1. Keep one run per indexer at the project root and tell the user to add the subfolder as a project of its own. Works, but a project is one folder, and the user would have to scan the same files twice.
2. Run the indexer once per workspace: the outermost folders below the project root that hold the project file the indexer needs.
3. Run it once per folder that holds a project file, however deep.

For the memory limit: one global limit (today), a limit per language, or a limit per indexer.

## Decision

- **Runs per workspace (option 2).** If the project folder or a folder above it holds a project file of the indexer, or the pack names none, nothing changes: one run at the root. Otherwise the runner takes, for the source files of the languages of the indexer, the outermost folder below the root that holds a project file, and the second pass runs the indexer once in each. An inner `Cargo.toml` is a member of the outer workspace, which rust-analyzer finds from the outer manifest, so option 3 would run the same code several times.
- A run is one working folder: `IndexJob::workspace` (`None` is the root). The working folder and `{project}` are that folder. The folder is built from paths of the snapshot; the runner still resolves it and refuses one that is no folder inside the project (a link out of it, a folder that is gone), so an indexer never runs outside the project (`FailureReason::Workspace`, reported as `RES-601`). The resolved path is used for the check only; the run gets the form of the project folder (on Windows the resolved form starts with `\\?\`, which not every indexer understands). Between the check and the start of the process a folder could be replaced by a link; that is accepted, because the project content is local and an indexer runs project code in the project anyway.
- The index names its files relative to the working folder. The importer puts the folder in front of every path (`ImportRequest::workspace`) and from there on works with project-relative paths as before. Each import replaces only the files below its workspace, so the runs do not disturb each other.
- The runs go one after the other, each with the limits of an indexer run (time, memory, output). The result of the indexer is the sum of the workspaces that ran through. A workspace that failed does not take the others down: its files keep level 1 and the report counts the failed workspaces (`failed_workspaces`, schema version 23, issue #752). The count says how many failed, not why: the reason of a single failure stays in the log. Only if no workspace ran through, the first failure is the reason of the indexer. A cancel stops the rest.
- **Memory limit per indexer.** `[level2.indexer_memory_mb]` holds one entry per shipped indexer, by the name from `[indexer]` of the pack (`rust-analyzer`, `scip-dotnet`, `scip-java`, `scip-python`, `scip-typescript`). `0`, the default, follows `level2.max_memory_mb`. The key is the indexer and not the language, because TypeScript, JavaScript and TSX share one process, and so do Java and Kotlin. The entries are in the default file, so a misspelt name is an unknown key and refused when the configuration is read, and the effective settings screen lists them with their origin. Like all `level2` keys, only the user configuration and command line flags may set them; the project file cannot. A new pack with an indexer adds its line to the default file.
- The limit also feeds `memory_env` (Node's heap), so a limit of 12288 for `scip-typescript` gives Node a heap of 9216 MB.
- No new dependency. The indexer adapter now depends on the domain crate (for the path type), which the dependency direction of blueprint 4.2 allows for adapters.

## Consequences

- `vscode-main` gets a Rust run in `cli/` and a TypeScript run with its own limit; whether the C# and TypeScript runs pass on that corpus is measured by the product owner after release, not by CI.
- The same code may be indexed twice if two workspaces overlap by path dependencies; the files are counted once because each workspace replaces only its own files.
- A project with many workspaces runs the indexer that many times in series; the time limit holds for each run. A list of workspaces in the settings and a line per workspace in the report are not part of this decision.
- The settings screen (tab "Analysis") edits the table with a field per indexer (issue #752, ADR 0103); until then it was only in the user file and in the effective settings.
