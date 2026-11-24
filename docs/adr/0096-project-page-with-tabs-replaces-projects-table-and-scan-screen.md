# 0096 – A list and a project page with tabs replace the projects table and the scan screen

Status: accepted (variant B chosen by the product owner on 2026-11-23; brought forward from 1.1 to before 1.0 by him on 2026-11-24)

## Context

The projects screen (blueprint B.1, mock 1) has eight columns, three of them overloaded. The state of a project is spelled out in four places, and the sections "Level 2", "Scan" and "Tools" of the selected project hang below the table. The separate screen "Scan and performance" (B.5, mock 5) shows the scan of the open project only and sits as one more button in the rail. The product owner looked at three variants (A: tidy table with a detail column, B: narrow list with a project page, C: cards) and chose B, with the scan screen folded into the project page. The concept and the mocks are in the project folder (`entscheidungen/projekte-ansicht.md`); the mocks are now mock 23 (`docs/mocks/ProjectPage.html`, screenshots in `docs/screenshots/project-page/`).

Blueprint B.1 and B.5 describe two screens that this decision merges and the rail changes, so a change of the blueprint is needed (CLAUDE.md rule 9).

## Options

1. A: keep the table, add a detail column.
2. B: narrow list on the left, a project page with tabs on the right; the scan screen moves into the tab "Scan".
3. C: cards.

## Decision

Option 2.

1. **Projects screen.** Left a narrow list: filter field, button "New", per project a state dot (current, new commits, scan running, problem, never scanned), the name, one line with the main language and the last scan, and a thin progress bar while a scan of that project runs. Right the page of the selected project: name, one state pill, source line (path or Git URL, branch and commit), buttons "Scan again", "Open in explorer" and the icons for rename and remove (the states of PR #714 stay: a removal stays visible, the actions are icons, there is no menu). "Cancel" (Esc) replaces "Scan again" while a scan runs. The form "New project" opens from the button "New" (key `N`); with an empty list it is open at once. The state dot is never the only sign: the state is also text (the pill on the page, the words of the list line) and the dot carries an `aria-label` with the state. Folder drop, the folder dialog and the keys `Enter`, `R`, `N` and the arrows stay.
   **Esc** closes an open input first (rename, the question before removing, the form "New project"); only when none is open does it cancel a running scan.
2. **Tabs of the project page** (`role=tablist`, `tab`, `tabpanel`, arrow keys move between the tabs; the scan progress is a `role=progressbar` with a value and a text): "Overview", "Scan", "Exact check (stage 2)", "Tools".
   - *Overview:* key figures (types, lines, files, last scan), a note for a failed stage 2 with its error code, one table of the languages (share, accuracy in words, files, lines, symbols; it replaces the separate language bar and the language table of the scan screen), cards "Last scan", "Tools" and "Database".
   - *Scan:* progress, throughput, cores and memory; the steps as one list with stage 2 as the last step (after the scan the fast steps are summed up); the language table with parse time; the files left out as a note. The page jumps to this tab when a scan of the selected project starts.
   - *Exact check:* the switch and the section of ADR 0071 and ADR 0072, unchanged in content.
   - *Tools:* the tool check of ADR 0033, unchanged in content.
3. **Database card.** The Overview shows where the database file of the project lies and how big it is (path, size, state), as the product owner asked on 2026-11-24. The data comes from the field `database` of the project (`path`, `size_bytes`, `state`) that ADR 0095 (PR #723) introduces; that PR is merged first. `state` has the values `ok` ("present", path and size shown), `missing` ("missing", path shown, size 0) and `removing` ("being removed", size shown as of now); while the field is absent the card says "Not available yet." The path is copyable. It is shown in the UI and the local API only; it does not go into exports, the MCP interface or reports.
4. **The scan screen goes away.** `/scan` has no button in the rail any more and redirects to the projects screen with the open project selected and the tab "Scan". The scan of any project in the list can be watched, also while several run at once. The recent-scans card is dropped (as in the concept the product owner chose on 2026-11-23; whether the tab should show a history of the last few scans stays an open question, issue to follow); the list shows the last scan of every project. Only the last report is stored per project, as today.
5. **Performance targets** apply to the whole app, not to a project. The status bar gets an entry "Performance"; a click opens the list of targets with measured values (mock 23, view 4). The entry shows how many targets were missed.
6. **No change in the server** for this ADR; the REST interface, the commands and the events stay. The only addition is the field of ADR 0095.
7. **Words.** "Lines" instead of "LOC"; one term in both languages for stage 2: "Exact check (stage 2)" and "Genaue Prüfung (Stufe 2)".
8. **Blueprint.** B.1 and B.5 are rewritten to this page; mock 1 and mock 5 are kept in `docs/mocks/` as the former state and marked as replaced in the mock list.

## Consequences

- The web package builds the page in three steps: list and page with Overview, then the tab "Scan" with the move of the scan parts and the redirect, then the status bar entry, the manual and the image comparisons. Each step keeps the tests green; texts in German and English, all three themes.
- Projects can no longer be compared side by side by size or scan time; the product owner accepted this. A sortable table can come back later as an alternative view if it is missed.
- There is room for later content (trends, rules, the change view of the after-1.0 plan) as further tabs.
- The panel "Background processes" (after 1.0) links to the tab "Scan" of the project.
- The role of the screen "Scan" in ADR 0072 (section of the scan in the projects screen) moves into the tab; its content stays.
