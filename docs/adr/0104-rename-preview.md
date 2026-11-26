# 0104 – Rename preview: what a new name would hit, without changing anything

Status: accepted (the scope was decided by the product owner on 2026-11-23, "Umbenennungs-Vorschau (E4)" in version 1.2; the mock `erw-umbenennen` was approved on 2026-11-26). Built in two pull requests: the core with the tool and the command first, then the dialog of the explorer.

## Context

A developer or an agent that renames a symbol needs the list of places that break, before the editor starts. The reverse index knows the direct usages; it does not know strings, configuration files and documentation that hold the old name, and nothing tells whether the new name is already taken in the same scope. The concept (`entscheidungen/erweiterungen-nach-1-0.md`, E4, K2, K3) asks for a preview that shows these things and changes nothing. CodeSkimr stays read-only: renaming is the job of the editor or the agent.

## Options

1. One use case in the application layer that the tool, the command and the dialog all call.
2. The usages tool plus a text search that every surface combines on its own.
3. A rename that CodeSkimr applies itself (refused by the concept).

## Decision

Option 1.

- **One function.** `codeskimr_application::rename::preview` takes the symbol (already looked up), the proposed name and the ports `CodeIndex`, `UsageIndex` and the new port `NameMentions`, and returns the places, the conflicts and the text hits. The tool `preview_rename`, the command `codeskimr preview-rename` (which calls the tool, like the commands of ADR 0099) and the dialog "Check rename…" read that one function, so they cannot drift apart. Nothing is written anywhere.
- **Places that have to change.** The declaration of the symbol, the declarations of its members that carry the same name (the constructors of a type), and the direct incoming usages of both, with tests included. A usage says whether level 2 verified it and how sure the resolver was; the places are grouped by file and layer in the counts. Only direct usages are counted: the places that name the symbol. A caller of a caller is not changed by a rename. A place seen twice at one position counts once, and the verified copy stays. `preliminary` is `true` while a listed place is unchecked, as for `find_usages`; the dialog says "stage 2: places checked by the indexer" only if every place is verified, otherwise "stage 1: preliminary".
- **Name conflicts.** The qualified name the symbol would get (its own with the last segment replaced) is looked up among the symbols of the snapshot. A hit is a conflict; `same_signature` says whether the two signatures are equal but for the name (they clash) or differ (an overload, which depends on the language). The new name must be a name (letters, digits, underscore, not starting with a digit) and differ from the old one, otherwise the call is `MCP-101` with `params.detail = new_name`. Not found by this check: a member of a base class or an interface that the renamed symbol would hide or implement under the new name, because the store has no such relation yet; the text hits below show the declaration of an interface method as "code".
- **Text hits.** The folder of the project is read as a scan walks it (`.gitignore` files inside the folder, `.git` and `scan.excluded_folders` skipped, links never followed), but every text file is read, not only those of a language pack, because configuration and documentation are where a name hides. A line counts if it holds the old name as a whole word (case-sensitive). Lines that a place above already explains (same file and line) are not listed again. Each hit says what it is: `documentation`, `configuration`, `string` (inside quotes in a code file), `nameof` (the compiler changes it with the name), `comment`, or `code` (code that no usage explains: another symbol of the same name, a place the resolver could not follow, the declaration of an interface). Binary files, files over 2 MiB and unreadable files are not read; the answer counts the unreadable ones (`skipped`). The text lines are source text: the tool needs `read_source` for them, and without it, and for a Git source (whose copy is only the scan's), `text_hits.searched` is `false` and the dialog and the command say so instead of showing an empty list.
- **Honest wording.** Everything says what it does not know: code outside the project is not seen (`public_interface` is `true` for a public symbol), strings and reflection are only found as text, and CodeSkimr never says that the code compiles afterwards. That is the compiler's answer.
- **Doors.** MCP tool `preview_rename` (`symbol`, `new_name`, `project?`, `limit?`; read-only; the list of tools grows from 15 to 16, below the limit of 17 of ADR 0097), command `codeskimr preview-rename <symbol> <new-name> [--limit N] [--project ID|FOLDER] [--json]` with the promises of ADR 0099 (`schema`, `stale`, failure as JSON with its code), and the dialog of the explorer opened from the context action "Check rename…" of a symbol (ADR 0092: the command line and the tool have the explorer as their counterpart). The lists are cut to `limit` (the configured default and maximum of the MCP tools) with the notice `MCP-501`; the counts stay whole.
- **No new code, no new dependency.** The new error is none: `MCP-101`, `MCP-202`, `SRC-201` and the codes of the ports are reused.

## Consequences

- `NameMentions` is a new port; the adapter `FolderMentions` lives in `codeskimr_source` and reuses the walk of the scan.
- Reading every text file of a project takes time on a very large project (the walk of the Linux kernel reads about 1.2 GB); the search stops at no limit of its own, but the dialog shows the places first and the text hits when they are found. A limit by total size is a later decision if a real project needs it.
- A rename of a symbol with many usages lists only `limit` of them; the counts say how many there are, and an agent can ask `find_usages` for the rest.
- Open (question 7 of the concept): whether CodeSkimr should later propose a patch. This ADR keeps to showing.
