# 0116 – Conventions check in the UI

Status: accepted (variant A of mock 44 was chosen by the product owner on 2026-11-27: a tab "Conventions" on the page Rules). Follow-up of ADR 0092.

## Context

`codeskimr check` (WP-35) and the MCP tool `check_conventions` (WP-37) check the files of a folder against the code rules of blueprint section 3 (`docs/conventions.md`): function length, cognitive complexity, parameters, literals and duplicates. The files are analysed afresh, so the check sees edits that no scan has seen. The UI had nothing like it; the page Rules shows only the architecture rules of the last scan. Principle of the product (ADR 0092): what the console can do, the UI can do.

## Options

1. A tab "Conventions" next to "Rules" and "Layers" on the page Rules, one button "Check now" for the whole project folder (mock 44, variant A).
2. A page of its own with the choice of project and scope: whole folder, changed files, chosen paths (mock 44, variant B; not chosen).

## Decision

Option 1.

- **One command, `conventions.check`.** Input `{project}`. It reads the folder the project was registered with and runs the same check as the console, through the same function (`check::check_folder`), with the configuration of the project folder (`.codeskimr.toml`). The answer is the answer of the MCP tool `check_conventions` (`conventions_answer`), held to `mcp.max_limit` findings of the configuration (`total` still counts all of them and the tab says how many are not listed): `files_checked`, `total`, `findings`, `unchecked`, `unmatched`, `not_checked`. There is no second copy of the mapping.
- **Right and token.** The command needs `read_source` (so the token, ADR 0035), like the MCP tool: the answer names files and functions of the project, and the check reads the code. It stores nothing, so it is `read_only`.
- **No path from the caller.** The input holds no path; the folder comes from the project. A project that is not a folder on this machine (a Git source, whose files live in the cache of the scan) answers `PRJ-101`, as the MCP tool does. Choosing paths or "changed files" (variant B) is a later step; the command can take `paths` then without a change of meaning.
- **No stored state.** Like the console, the check runs with `stored: None`: the layer rule `CHK-103` belongs to the rules (ADR 0082), and duplicates are searched among the checked files only. The tab says so under the findings (the list `not_checked`).
- **The caller counts.** `check_folder` takes the principal of the request, not that of the process, so the check judges with the rights of the caller who was asked for them.
- **One run per project.** A second check of a project that is being checked answers `PRJ-304`, so a double click or a second tab cannot start several full analyses of one folder. The lock is its own and does not mark the project as scanning.
- **Long runs.** The check runs on the server's blocking pool like every command. A very large project (tens of thousands of files) takes about as long as a scan's analysis stage; the tab shows that it is running and can be left. A cancel button is not part of this step.
- **REST only.** The MCP tool exists already.

## Consequences

- The console command and the tab give the same findings for the same folder, because both call `check::check_folder`.
- One new error code, `PRJ-304` (a check of the project is already running), because `PRJ-302` says "a scan" and the person has started none; no new dependency.
