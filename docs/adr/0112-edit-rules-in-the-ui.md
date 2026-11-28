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
- **The layer editor moves onto this screen** (superseded by the addendum of 2026-11-27, "the layers get their own place in the rail"). The product owner decided on 2026-11-27 that layers belong to the project and are rules too: the screen Rules has two tabs, "Rules" and "Layers", and the layer editor (ADR 0065) is the second one. The entry "Edit layers" in the settings and the route `/layers` are gone; the hints that said "Set layers" open the tab.

## Addendum (2026-11-27, issue #811): files changed while a dialog is open

- **Version of the file.** `rules.show` and `layers.get` answer `file_version`, a number computed from the text of the project file (empty for a Git project). `rules.set` and `layers.set` take it back as the optional `file_version` (`rules.preview` writes nothing and does not check it); if the file is different by then (edited by hand, `git pull`), nothing is written and the error is the new `CFG-106` `project_file_changed`. The check runs under the same lock as the read and the write, so no change can slip in between. A client that sends none (the CLI, a script) writes as before.
- **Version, layers and rules come from one read.** `rules.show` and `layers.get` take the rules, the layers and the version from the same text of the project file (a Git project has no file: its layers stay those of the scan). Before, the layers were those of the last scan, so a file edited after the scan paired its new version with old layers and a save overwrote the edit unseen. The screen, holding the state and the preview judge with those layers; `check_rules` and `codeskimr rules` keep the layers of the last scan (ADR 0065: they apply from the next scan). A failure of the random source is logged; the name then keeps pid and counter.
- **Not for `unused.keep` and `unused.lift`.** They add or take out one entry on the text they read under the lock, so a change made elsewhere is kept; there is nothing to lose.
- **Temp file name.** The new file beside the project file carries a random number besides the pid and the counter, so a leftover of a crashed run can never block a later write with `CFG-104`.

## Addendum (2026-11-27): the layers get their own place in the rail

- **Own screen again.** The product owner decided on 2026-11-27 that the layers are not a tab of the screen Rules but a screen of their own in the rail, between "Change" and "Rules". It is at `/layers`, shows the same editor (ADR 0065) and needs an open project like the other project screens. The screen Rules has no tabs any more; the hints that said "Set layers" open the new screen. Nothing changes in the commands or in the file.
