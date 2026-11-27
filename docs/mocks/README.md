# UI mocks

Binding UI reference for the web frontend (blueprint section 15). One file per screen, plus `canvas.json` (position of each screen on the design canvas).

| # | Screen | File |
|---|---|---|
| 1 | Projects (replaced by 23, ADR 0096) | `Main.dc.html` |
| 2 | Explorer | `Explorer.dc.html` |
| 3 | Dependencies | `Graph.dc.html` |
| 4 | Quick search (Ctrl+K) | `Palette.dc.html` |
| 5 | Scan and performance (replaced by 23, ADR 0096; side column at least 400 px wide, so the target values fit; changed at the product owner's request) | `Scan.dc.html` |
| 6 | Usages (symbol level; the accuracy card shows an "Approximation" chip and a short note for languages below 99 %, from `ui.approximate_languages`) | `Usages.dc.html` |
| 7 | Sign-in (team server) | `Login.dc.html` |
| 8 | Users and project grants | `Users.dc.html` |
| 9 | Groups and roles | `Roles.dc.html` |
| 10 | Access tokens | `Tokens.dc.html` |
| 11 | Audit log | `Audit.dc.html` |
| 12 | Single sign-on | `Sso.dc.html` |
| 13 | Settings: themes and language | `Themes.dc.html` |
| 14 | Annotations: marks, notes, tasks | `Annotations.dc.html` |
| 15 | Alerts | `Alerts.dc.html` |
| 16 | Task board | `Tasks.dc.html` |
| 17 | Settings: indexers (tool check of level 2) | `Indexers.dc.html` |
| 18 | Projects: level 2 switch per project (ADR 0071) and the scan section: progress, cancel, reasons, report (ADR 0072) | `Level2Switch.dc.html` |
| 19 | Projects: folder dialog for a new project (ADR 0075) | `FolderPicker.html` |
| 20 | Statistics: overview, rankings, complexity (ADR 0081); a plain page that renders in a browser, all three themes, with sample data from a scan of CodeSkimr | `Statistics.html` |
| 21 | Error codes: list to look up with search, filters and explanation; reached from the settings; a plain page, all three themes, with sample data | `ErrorCodes.html` |
| 22 | Settings: effective settings (values of the configuration with their origin, read-only; a plain page that renders in a browser, all three themes, with sample data; screenshots in `docs/screenshots/effective-settings/`) | `EffectiveSettings.html` |
| 23 | Projects: list on the left and the project page with tabs on the right, tab "Scan" while a scan runs and after it, the targets of performance at the status bar, card "Database" (ADR 0096); a plain page that renders in a browser (`?v=uebersicht\|laeuft\|fertig\|leistung\|dbfehler&t=hell\|petrol\|dunkel`), screenshots in `docs/screenshots/project-page/`. Replaces mocks 1 and 5, which stay as the former state | `ProjectPage.html` |
| 24 | Projects: the window "New project" (variant C chosen by the product owner on 2026-11-25, then changed at his request into a centred modal dialog with a dimmed background, without the arrow to the button and wide enough that the key hint does not wrap; the mock still draws the first state, the screenshots show the real one; variants A and B stay as drafts) and the dialog "Remove project" (D1; D2, with typing the name, was not chosen); a plain page that renders in a browser (`?v=A\|B\|C\|D1\|D2&t=hell\|petrol\|dunkel&s=folder\|git`), screenshots in `docs/screenshots/new-project-popup/` | `NewProjectPopup.html` |
| 28 | Tab "Scan": the list of the last scans with duration and changed files, and the button "Compare with current" (variant A chosen by the product owner on 2026-11-26, ADR 0097); a plain page that renders in a browser (`ChangeCompare.html?v=m28a&t=hell\|petrol\|dunkel`), the approved pictures are in `docs/screenshots/scan-history/` (`m28a-hell.png`, `m28a-petrol.png`, `m28a-dunkel.png`); variant B (a small bar chart instead of the list) was not chosen | `ChangeCompare.html` |
| 29 | Screen "Change", tab "Comparison": the choice of base and target scan, the four counts (new, changed, removed, moved) and the list of the symbols (ADR 0097, approved on 2026-11-26); a plain page that renders in a browser (`?v=m29&t=hell\|petrol\|dunkel`), pictures in `docs/screenshots/change/` (`m29-*.png`) | `ChangeCompare.html` |
| 26 | Projects: the tab "Scope" (languages and folders a project covers, consequences of a change), the hint in the window "New project" for a folder that lies inside a project, and the stage 2 report text without "add as a project of its own" (ADR 0102, chosen by the product owner on 2026-11-25); a plain page that renders in a browser, all three themes | `Scope.html` |
| 38 | Settings: the five tabs Appearance, Analysis, Access, Effective settings and Info with the search in the tab bar (ADR 0098; variant B chosen by the product owner on 2026-11-24); a plain page that renders in a browser (`?v=b1\|b2&t=hell\|petrol\|dunkel`, b1 shows "Appearance", b2 "Analysis"; a1 to c2 are the variants that were not chosen) | `SettingsTabs.html` |
| 39 | Screen "Change", tab "Impact" (`erw-auswirkung`, ADR 0097, approved on 2026-11-26): the five figures, the table of the changed symbols with the state of their usages, the notes on layer violations and critical symbols, and the usages of the selected symbol with the checklist "not touched"; a plain page that renders in a browser (`?t=hell\|petrol\|dunkel`, all three themes without `t`), pictures in `docs/screenshots/change/` (`m39-*.png`) | `ChangeImpact.html` |
| 40 | Statistics, tab "No usage found": the filter "Assessment" as a list with the choices all, very likely unused, private, internal or protected, public and uncertain (asked for by the product owner on 2026-11-27); a plain page that renders in a browser (all nine boards one below the other: the views closed, open and private in Hell, Petrol and Dunkel), pictures in `docs/screenshots/unused-filter/` (`m40-*.png`) | `UnusedFilter.html` |

Mocks 7 to 13 have a `theme` property (`hell`, `petrol`, `dunkel`; values in blueprint 15.3, ADR 0021). Mocks 1 to 6 are drawn in Hell. Screens 7 to 12 belong to the team edition (ADR 0019), screen 13 to 1.0.

The files are written in a design-canvas format and do not render on their own in a browser. Read them for exact values (inline styles); to look at the rendered screens, open the published canvas: <https://claude.ai/artifact/FRpzF7hdgZVdHCm2mxRuNZ> (the link works for the owner of the canvas).

Mocks 14 to 19 (14 to 16: ADR 0068; 17: indexer section of the settings, approved by the product owner on 2026-11-17; 18: the level 2 switch in the projects screen, ADR 0071, added with the web package because mock 1 did not show it; 19: the "Browse" button of the new-project form, approved by the product owner on 2026-11-20) are plain static pages with several boards each; they open in any browser. The theme is the class of `body` (`hell`, `petrol`, `dunkel`), the default is Petrol. They are not part of `canvas.json`.

Mock 1 shows under "Last scan" the level and origin of the last scan and what stage 2 did (ADR 0072); mock 6 shows the same report on the accuracy card; mock 18 has the scan section with its states.

All names and numbers in the mocks are sample data.

The `THEMES` block at the top of each script in mocks 7 to 13 (and the miniatures in mock 13) holds the same values; when a theme value changes in blueprint 15.3, update all of them together. Petrol is the default theme of the product and is drawn as selected.
