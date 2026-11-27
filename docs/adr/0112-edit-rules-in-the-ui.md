# 0112 – Creating and changing the rules in the UI

Status: accepted (variant A of mock 41 was chosen by the product owner on 2026-11-27). Labelled `security-sensitive`: it writes into a file of the project folder.

## Context

The screen Rules shows the rules of `.codeskimr.toml` (ADR 0104 to 0107), but they could only be created by editing the file by hand; ADR 0104 left "editing them in the UI" for a later step. Principle of the product (2026-11-24, ADR 0092): everything the file or the command line can do must be possible in the UI.

## Options

1. Per rule: a pen and a bin on every card and a dialog that changes one rule and writes at once (mock 41, variant A).
2. One edit page with all rules as form rows and a single "Save" (mock 41, variant B; not chosen).

## Decision

Option 1.

- **Two commands, one input.** `rules.set` (right to manage) writes, `rules.preview` (right to read) only judges. Both take the rules as a whole: `cycles`, the `forbid` entries, the `public_api` entries and, for the layer rule, the `may_use` of each layer (`limits`; no list is "no limit", an empty list is "nothing"). The screen builds the whole set from the last answer with the one change in it, so the two commands cannot drift apart and a dialog never writes half a rule.
- **Only a folder project.** The path is the registered folder plus the fixed name `.codeskimr.toml`, never a path from the caller. A Git project is refused with `PRJ-101`, like `layers.set` (ADR 0065). The write is the same atomic replace (new file beside it, created exclusively, then renamed).
- **The rest of the file stays.** Only the tables `rules` (`[rules]`, `[[rules.forbid]]`, `[[rules.public_api]]`) are written anew at the end; every other line, comment included, stays. Comments inside the old rules tables are rewritten. A change of `may_use` rewrites the `[[layers]]` tables with the same names, namespaces and order. The result must be a valid project file: an invalid rule (`CFG-101`), a file that is not TOML (`CFG-102`) or a text over the size limit leaves the file as it was.
- **Same validation as reading.** Names, "neither inside the other", twins and `may_use` naming an unknown layer are judged by the one code that reads the file; there is no second set of rules for the editor. A limit for a layer that does not exist is ignored.
- **Preview before writing.** The dialog asks `rules.preview` a moment after the last key and says how many violations the last scan would find, and how many of them the held state already has. Nothing is written by it.
- **Layers.** Names and namespaces of the layers stay in the layer editor (ADR 0065). The dialog "Layers" only sets what each layer may use.
- **No new tool for agents.** Both commands are exposed on the REST API of the app only, not as MCP tools: an agent edits the file as it edits any other file, and `check_rules` judges it.

## Consequences

- No behaviour changes for projects that do not use the editor.
- A rule written by hand and a rule written by the editor look the same in the file.
- Steps: 1 the writers in the core and the two commands; 2 the menu "Add rule", pen and bin on the cards and the dialogs.
- **The layer editor moves onto this screen.** The product owner decided on 2026-11-27 that layers belong to the project and are rules too: the screen Rules has two tabs, "Rules" and "Layers", and the layer editor (ADR 0065) is the second one. The entry "Edit layers" in the settings and the route `/layers` are gone; the hints that said "Set layers" open the tab.
