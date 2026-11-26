# 0104 – The screen "Rules" and the frozen state of old violations

Status: accepted (the scope was decided by the product owner on 2026-11-23, "Regelseite und neue Regelarten (A1)" in version 1.2, and the mock `erw-regeln` was approved on 2026-11-26). The steps below are built one pull request each; this record is written with the first. Each new kind of rule has an ADR of its own (0105, 0106, 0107), as the concept asks.

## Context

The layer rule exists (`CHK-103`, ADR 0054, ADR 0082), but its violations are visible only in the output of the convention check and in the layer diagram. `codeskimr check` does not read the store, so it does not report the layer rule at all. Principle of the product (2026-11-24): everything the command line can do must be possible in the UI. An old project has hundreds of violations on the first day, so a rule that fails the CI for all of them is switched off the same week.

## Decision

1. **Rules stand in the project file.** `.codeskimr.toml` gets the table `rules` (ADR 0105 to 0107): `cycles`, `[[rules.forbid]]` and `[[rules.public_api]]`. Like `[[layers]]` and `[[unused.ignore]]` the tables are trusted in a project file: they can only report, never run or open anything. The screen shows the rules; editing them in the UI is a later step.
2. **One evaluation for every door.** `codeskimr_application::rules` reads the certain references of the newest snapshot (not of kind `test`, the filter of the layer graph) and returns, per rule, its violations. A violation is the pair of the referring symbol and the symbol it must not reach, with every place that makes the reference (file, line, column). The screen, the command `codeskimr rules [--json]` and the MCP tool `check_conventions` read this one function, so they cannot drift apart. Like `CHK-103` it sees the state of the last scan, not edits that no scan has seen.
3. **Identity of a violation.** `rule`, referring symbol and target symbol, never the line, so that moving code does not make an old violation new. Several places of one pair are one violation with several places (the mock: "4 Stellen").
4. **"Stand festhalten" (hold the state).** The button writes the file `.codeskimr-baseline.toml` next to the project file: the date and one entry per violation that exists now (`rule`, `from`, `to`). The file is meant to be committed. A violation with an entry is an old burden ("Altlast"); one without is new. The CI (`codeskimr rules`, exit code) fails only for new violations. An entry whose violation is gone is shown as resolved and removed the next time the state is held; the number of old burdens must only go down. Holding the state again replaces the file as a whole; nobody is named in it, the history of the repository says who.
5. **The layer rule is a rule of the page too.** Its violations come from the same references (a violation per symbol and forbidden layer, as in ADR 0082) and can be held like the others. `check_conventions` keeps reporting `CHK-103` as before.
6. **Codes.** An invalid rule is `CFG-101` on the key `rules`. Violations of the new kinds get the code `CHK-107` (`rule_violation`) with the kind in the answer; it is registered with step 2, the first step that reports it.
7. **Written like the layer editor.** The baseline is written only for a folder project (a Git project has no file the user owns) and needs the right to manage, as `unused.keep` does (ADR 0101, step 4).

## Consequences

- No behaviour changes for projects without `rules`.
- The page and `codeskimr rules` need a scanned project; `codeskimr check` stays a check of files without a store.
- Steps: 1 the rules of the project file, the evaluation of "forbid" and "public interface" and the baseline in the core; 2 the doors (UI command, command line, MCP); 3 the screen; 4 the cycle rule once the cycle detection of the module graph (ADR 0108) is in; 5 holding the state in the UI.
