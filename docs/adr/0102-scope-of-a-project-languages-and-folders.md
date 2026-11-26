# 0102 – A project can leave out languages and folders (scope)

Status: accepted (way A confirmed by the product owner on 2026-11-25, mock 26 chosen on 2026-11-26)

## Context

A project covers everything below its root. In a large repository with several languages (a web front end next to a Rust core, a vendored tree, generated code) this is too much: the scan takes longer, the database grows, and stage 2 cannot use the project file of a subfolder. The product owner's rule of 2026-11-25 is that nobody is forced to load the same code twice into several projects; doing it stays allowed, at most with a hint, never a block. Roadmap 4.11 plans a choice of languages and folders per project for 1.1.

## Options

1. A list of what is covered (allow list).
2. A list of what is left out (deny list) for languages and folders.
3. Several projects per repository, one per subfolder.

## Decision

Option 2.

1. **Model.** The scope of a project is a set of left-out languages and a set of left-out folders (paths relative to the project root, with `/`). An empty scope covers everything, so new languages and new folders are covered without any action. A folder that is left out covers all folders below it. The type `Scope` lives in the domain crate.
2. **Storage.** The scope is stored in the database file of the project (ADR 0095), tables `scope_languages` and `scope_folders` (migration 0024). It does not enter the `Project` type; the port `ScopeStore` writes it, `ProjectRepository::scope` reads it.
3. **Effect.** Discovery applies the scope: the walk does not enter a left-out folder and skips files of a left-out language before it hashes them. A change of the scope takes effect with the next scan. Files that leave the scope are treated as removed files by an incremental scan.
4. **Commands.** `scope.get` (read) returns the scope and an overview: per language and per folder the number of files and whether it is covered, the project files found in folders (for stage 2), and a hint when the list is cut (`scan.overview_depth`, `scan.overview_max_folders`). For a folder project the overview is a live listing without filters, for a Git project it comes from the newest snapshot; stored entries are always shown so that they can be switched on again. `scope.set` (register projects) replaces the scope. The console has `codeskimr scope`, so the UI and the console can do the same.
5. **Limits.** `scan.max_scope_entries` caps the number of entries; an invalid scope is the error PRJ-103 (`project_scope_invalid`). Folders hidden by `.gitignore` or by `excluded_folders` are not offered in the overview. Folder names are compared as text, with case: on a file system that ignores case, `Src` does not match `src`.
6. **Stage 2 per workspace and the interface** (an own tab "Scope", the hint when a folder is added as a second project, the new text of the stage 2 report) follow in separate pull requests on this decision.

## Consequences

- One migration; projects without entries behave as before.
- The overview of a folder project walks the tree once per call; it is bounded by the two limits above.
- Notes and alerts attached to symbols of files that leave the scope behave as for removed symbols.
- `matches_project_file` and its wildcard constant moved from the indexer adapter to the application ports: the overview needs them to name the project files of a folder, and an adapter may not depend on another adapter.
- A project may cover a subfolder that another project also covers. No check blocks this.
- BLUEPRINT 6.2, 7 and 10.2 and the error catalogue are updated with this decision.
