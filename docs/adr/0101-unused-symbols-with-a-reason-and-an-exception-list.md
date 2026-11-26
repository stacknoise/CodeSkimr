# 0101 – Unused symbols come with a reason and an exception list

Status: accepted (the scope was decided by the product owner on 2026-11-23, "ungenutzte Symbole mit Begründung (E3)" in version 1.1; the mock `erw-ungenutzt` was approved on 2026-11-26). The steps below are built one pull request each; this record is written with the first. Steps 1 and 2 are built; 3 and 4 follow.

## Context

The tab "No usage found" of the statistics screen (ADR 0081) lists the symbols nobody uses with an assessment: very likely unused, public, uncertain. A user cannot see why a row got its assessment, cannot say "this one stays on purpose" so that the number goes down for good, and neither a script nor an agent can ask the question (no CLI command, no MCP tool). The concept (`entscheidungen/erweiterungen-nach-1-0.md`, E3, K2, K3) asks for a reason per row, an exception list in the repository, and the doors `codeskimr unused` and `find_unused`.

## Decision

1. **Reason per row (step 1).** Every row of the ranking `unused` carries `reason`, one of `private`, `internal`, `public`, `approximate`. It follows from the visibility and from whether the language is an approximation (`UnusedReason::of`, in step with `Assessment::of`), so it never says more than the assessment knows. The UI shows it in the column "Reason" instead of the column "Visibility" (the reason names the visibility). Reasons the scan cannot know yet (entry point of a framework by attribute, interface member with implementations) are not invented; they can be added as further values when the scan stores what they need.
2. **Exception list (step 2).** `.codeskimr.toml` of the project may hold `[[unused.ignore]]` tables with `paths` (globs on the relative file path), `symbols` (globs on the qualified name) and a mandatory `reason` text; at least one of `paths` and `symbols` is needed (an entry with both needs a match in each; `*` is any run of characters, `?` one character), an entry without a reason is refused (`CFG-101`). The list stands in the repository, so the whole team sees the same. A symbol that matches an entry is no longer counted in "No usage found"; the tab lists such rows below the table in a group "Left out" with the reason of the entry. The tables are trusted in a project file (unlike `level2`, `editor`, `ui` and `export`): they can only hide rows from a list, never run or open anything.
3. **"Keep on purpose" (step 4).** The button writes an entry into the project file of a folder project (the same way as the layer editor, ADR 0065); for a Git project there is no file and the button is off. "Lift" removes the entry again. The author and the date of the entry go into its reason text.
4. **Doors (step 3).** `codeskimr unused [--json]` and the MCP tool `find_unused` answer the same as the tab: rows with assessment and reason, the left-out rows separately, `schema` version, `stale` when the files are newer than the scan. This is the first of the commands of K3; the MCP list grows by one tool, as the concept plans.
5. **Name.** The tab stays "No usage found"; "dead code" would promise more than CodeSkimr knows.
6. **Not decided here.** "No new unused symbols" as a rule of `codeskimr check` for CI (open question 5 of the concept).

## Consequences

- The number in the tab and on the overview card counts without the left-out symbols once step 2 is in; until then it is unchanged.
- With exception entries the answer is computed from the facts of the snapshot instead of the stored counts (ADR 0091); that is slower on a very large project and is measured when step 2 lands.
- Texts of the reasons exist in German and English in `locales/`.
