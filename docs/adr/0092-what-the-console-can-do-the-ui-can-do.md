# 0092 – What the console can do, the UI can do

Status: accepted (principle set by the product owner on 2026-11-24; the split of the open gaps into before and after 1.0 decided by him on a decision card the same day)

## Context

In the hand test of 1.0.0-rc.6 the product owner found that a full scan of a known project could only be started with `codeskimr scan <path>`: the buttons of the Projects screen always scanned incrementally. He set the principle that everything the console can do must also be possible in the UI in some way.

A comparison of the command line (`docs/cli.md`) with the UI on that day showed these gaps:

1. A full scan of a known project.
2. `scan --watch`: scanning again whenever files change.
3. `credential login`: signing in to GitHub or GitLab with the OAuth device flow.
4. `check`: the convention checks.
5. `codes`: the list of all error codes.
6. `--set` and `--config`: the settings beyond theme, language, editor, logins and layers.
7. `--version`: the UI does not show the version of the program.

## Options

1. No rule; close gaps when somebody notices one.
2. Full parity at once, before 1.0.
3. A binding principle with a list of reasoned exceptions, and a plan for the gaps that exist.

## Decision

Option 3.

- **Principle.** Every function of the command line has a counterpart in the UI, unless it is listed here as an exception. A work package that adds a command or an option to the command line adds the counterpart in the UI in the same package, or names the gap with an issue and adds it to the list below through a change of this ADR.
- **The counterpart need not look the same.** It reaches the same result through the same registered command (blueprint 10.1) where there is one, otherwise through the same use case or the same single source (for example the generated registry of error codes), so that the rules, permissions and error codes are the same and nothing is built twice.
- **Exceptions that stay in the console:**
  - `level2 confirm`: the consent to level 2 is typed in a terminal (ADR 0033, decision 10 and addendum). The UI opens that console window on Windows and macOS and shows the command to copy elsewhere (ADR 0071, decision 6); it never confirms itself.
  - `serve`, `open`, `mcp`, `--console`, `--data-dir`, `--languages`, and `--config` and `--set` as start options: they decide how and where the program starts, before there is a UI. The UI may show them.
  - `--json`: output for scripts; the counterpart is the REST interface.
  - `--help`: the counterpart is the manual.
- **Before 1.0:** gap 1 (button "Full scan", `projects.rescan` with `full`), gap 5 (a screen with the list of error codes) and of gap 6 the display of the settings in effect, read only.
- **After 1.0:** gap 2 (with the background jobs and the live view), gap 3, gap 4 (the extension "Rules"), changing settings in the UI (the rest of gap 6; `--set` at the start stays a start option) and gap 7 (issue #690). Each needs its own package; gaps 2 and 3 need their own ADR.

## Consequences

- Blueprint 10.2 names the principle and points here.
- A reviewer checks a change of the command line against this ADR.
- The exceptions are few and each has a reason; a new exception is a change of this ADR and needs the label `blueprint-change`.
- Until the gaps of "after 1.0" are closed, the manual names the console as the only way for them.
