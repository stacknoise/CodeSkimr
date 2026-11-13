# Browser UI (WP-42)

SolidJS, TypeScript and Vite in `web/`. Decisions: ADR 0028 and ADR 0030.

## Develop

```
cd web && npm ci
npm run dev                   # vite build --watch, writes web/dist
codeskimr --set ui.dir=web/dist serve
codeskimr open
```

## Checks

`npm run typecheck`, `npm run lint`, `npm run format`, `npm test` (Vitest) and `npm run e2e` (Playwright against the real server, needs Chromium; `CODESKIMR_CHROMIUM` can point to a binary).

## Structure

- `src/shared`: API client, session (launch code exchange, token in `sessionStorage`), theme store, texts (Fluent files from `locales/`), editor link, small UI parts.
- `src/features`: settings (theme cards with preview, language, editor), connection screen, projects (screen 1, WP-44), explorer (screen 2, WP-45), usages (screen 6, WP-53).
- Themes Hell, Petrol (default) and Dunkel are CSS variables generated from `design/tokens.toml` by `cargo run -p codeskimr_codegen`; never edit the generated files.
- No literals in the code: values live in `consts.ts` files, texts in `locales/`.

## Projects (screen 1, WP-44)

The screen calls three commands over `POST /api/v1/commands/{id}` (`shared/api/commands.ts`): `projects.list`, `projects.add` and `projects.rescan` (see `docs/server.md`).

- `projects.ts` keeps the list, the failure of the last action and the scans started in this tab. While a scan runs, the list is read every 1.5 s, so a running scan shows up as "scan running".
- Keys: `N` focuses the source field, arrow keys move the selection, `Enter` opens the project (it becomes the active project, `shared/project/active.ts`, and the explorer is shown), `R` scans it again. Typing in a field and Ctrl, Alt or Meta combinations never trigger them.
- Drop: a `file://` address or text dropped on the window goes into the source field. A browser does not tell the path of a dropped folder in every system; the screen then says so and asks to paste the path.
- Deviations from the mock: a small "scan again" button sits next to the status pill (so the re-scan has a mouse path, not only the key R); the status "new commits" and the scan duration of earlier sessions are missing because the data model has neither (the duration of scans started in this tab is shown); language colours come from the layer tokens.

## Explorer (screen 2, WP-45)

The screen shows the active project (`shared/project/active.ts`) with two commands: `explorer.outline` (the tree) and `explorer.symbol` (one type with breadcrumb, metrics and member table); both need only the read right (see `docs/server.md`). The source column (WP-45b) adds `explorer.source`, which needs `read_source` and the installation token (ADR 0035). Screenshots: `docs/screenshots/explorer-hell.png`, `explorer-petrol.png`, `explorer-dunkel.png`.

- Tree: layer (with its colour dot), namespace, type (nested types are rows below their type), then the groups "Fields etc.", "Methods" and nested types with their members. The tree is virtualised: only the rows in view (plus a margin) are in the DOM. It is one `tree` with `aria-activedescendant`; the group rows are expandable `treeitem`s as well. Click rules (blueprint 15.4): the caret only opens and closes; a click on an unselected type selects it; a click on the selected type opens or closes it; groups open and close on their own; selecting from inside the tree never changes what is open; at the start everything is closed and the overview is shown. The filter keeps the types whose name contains the text, with their layer and namespace opened.
- **Deviation from blueprint 15.4, rule 1:** the blueprint says a click on an unselected type "selects and expands it" but also that a selection from inside the tree never changes the open state. The explorer follows the second rule: the first click only selects, a click on the selected type opens or closes it. Issue #140 asks for a decision and, if needed, an ADR.
- Keys: Up and Down move the cursor, Right opens (or goes to the first child), Left closes (or goes to the parent), Home and End jump, Enter and Space select or toggle, `Ctrl+Left` closes everything, `Ctrl+.` shows the selection.
- Details: breadcrumb, name with kind and visibility, documentation, ten metric tiles, member table with the chips All, Properties, Methods and "public only". The colour of a metric and of the complexity of a member comes from the thresholds in `config/default.toml` (`metrics.*`), which `explorer.outline` delivers; the number is always shown too, and the tooltip of a column or tile names the limits. A failure of the symbol keeps the tree.
- Source column (WP-45b): the right column shows file, position and code of the selected member, or of the selected type if no member is picked. `explorer.source` reads the lines of the declaration from the project folder as it is now (not from the scan), so a file edited since the scan can show shifted lines; the next scan fixes that. Only a project that is a folder on this machine has source; a Git project says so. A file that is gone gives the registered failure `SRC-201`, a file over 4 MiB `SRC-501`, a declaration over 400 lines is cut with a note. The path is resolved against the project folder and anything that lies outside it, also through a link, is refused. Line numbers and code are two plain `pre` blocks, so a long declaration costs two DOM nodes.
- Deviations from the mock: the code has no syntax colours and no hover preview of type names, and the keys Alt+Up and Alt+Down for the previous and next member are missing (the data model has no token spans yet); the button "Graph" is shown but off (`aria-disabled`, with a hint) until the explorer can show the selection in the dependency map; "Open in editor" (`E`, `Alt+Enter`) builds the link of the setting `editor` from the folder of the project and the file and first line of the symbol and hands it to the browser (only for a project that is a folder on this machine; unit tests of the link and the keys); the tab "Dependencies", the metric "Test names the type" and the base type in the header are missing (the data model has none of them); the access column shows the visibility only; the usage count opens the usages screen; the signature column shows a dash where the language pack stores none; kind and visibility are shown as the keywords of the language, not translated; the C# qualified name of a type carries a prefix (issue #137).

## Quick search (screen 4, WP-46)

`Ctrl+K` (`Cmd+K` on a Mac) or the search field of the top bar opens the overlay (blueprint B.4, mock `Palette.dc.html`). It searches the newest scan of the active project with the command `search.query` (read right, no source text; see `docs/server.md`). Screenshots: `docs/screenshots/search-hell.png`, `search-petrol.png`, `search-dunkel.png`.

- Input with a magnifier, hit count and time (green) and the key `Esc`, chips All, `t:` Types, `m:` Members and `>` Commands (a prefix in the input wins: `t:` types, `m:` members, `>` commands; the chip is reset to All each time the overlay opens), grouped hits with the match marked (the first place where the text occurs in the name), preview of the chosen hit with its actions, footer with the keys (arrows choose, `Tab` filter) and the fuzzy-search hint. The list is virtualised and holds at most 50 hits.
- Keys: Up and Down choose, `Enter` opens the hit in the explorer (the rows down to it open, because external navigation opens nodes), `Tab` and `Shift+Tab` change the chip, `Ctrl+C` (`Cmd+C` on a Mac) copies the qualified name when no text of the input is selected; `Ctrl+K` while the overlay is open keeps text and hits, `Esc` closes and gives the focus back to where it was.
- Commands (`>`): the screens (projects, explorer, settings) and "scan project again" for the active project.
- Deviations from the mock: no group "Files" and no scope `@` (all projects), because the index holds names of symbols only and the UI works on one project; the preview shows kind, qualified name and file but not the doc and the metrics; "Show in graph" is listed but off until the explorer can show the selection in the dependency map; `Alt+Enter` opens the declaration of the hit in the editor (the line comes from `explorer.symbol`); the key hints of the preview say `Ctrl` (the `Cmd` variants work on a Mac); the time shown is the round trip of the command, not the time of the query alone; the hit of a member of a nested type opens the member within the nested type.

## Top bar (blueprint 15.2, issue #167)

Logo and product name, then, once a project is open, a slash, the project name (a link to the projects screen) and a chip, then the search field with a magnifier, which takes the room left of it up to 520 px as in the mocks. The chip shows the branch of a Git source or "Working folder" for a folder, and how old the newest scan is (`app-chip-folder`). Nothing is shown for the project while none is open. The HTML export uses the same bar with the project name and no chip. Deviation: the chip never says "Beobachtung aktiv": the server of the UI does not watch folders (see below).

## Status bar and language switch (WP-47)

The footer of every screen (blueprint 15.2, 15.5, mock `Main.dc.html`) shows only measured values: the connection state, the number of projects and types (summed from `projects.list`), the start time and the memory of the program (command `status.info`, read right, read again every 5 seconds), and on the right a switch `EN | DE`. The switch calls `setLanguage`, like the language field in the settings, so both stay in step and the choice is stored in the browser. Numbers are written in the chosen language (`1,220` / `1.220`).

- Start time: milliseconds from the start of the program (`run::execute`) until the server answers on its port. A server that is not started by `codeskimr serve` (the test server) reports none, and the bar leaves the item out. Memory is the physical memory of the process (crate `memory-stats`, Linux, macOS, Windows); if the system does not tell, the item is left out.
- Deviations from the mock: "Watching for changes" and "Sample data" are not shown. The server of the UI does not watch folders (only `scan --watch` does) and there is no sample data, so showing them would be decoration. The texts `app-watching` and `app-sample-data` stay in `locales/` for the packages that add them.

Screenshots (1440 x 900): [Hell](screenshots/status-hell.png), [Petrol](screenshots/status-petrol.png), [Dunkel](screenshots/status-dunkel.png).

## Usages (screen 6, WP-53)

`U` in the explorer (or the "Usages" button, or a click on the usage count of a member) opens the usages of the selected member, or of the selected type if no member is picked. The screen (`/usages`, no button in the rail, like the mock) uses one command, `usages.query` (read right, no source text; see `docs/server.md`). Screenshots: `docs/screenshots/usages-hell.png`, `usages-petrol.png`, `usages-dunkel.png`.

- Toolbar: level (only "Symbol" exists here; "Namespaces" and "Types" belong to the graph screen and are shown off with a hint), direction (both, who calls, what is called), depth (1, 2, 3, all; "all" is cut to `mcp.max_depth`) and "Show tests". Every change reads the usages again.
- Graph: one column per step of callers to the left, the symbol in focus (yellow ring), one column per step of callees to the right. A card shows the type and member name, the kind badge, the layer and the number of places; a click on a card makes that symbol the focus. Lines are blue (calls, the accent colour), orange (the focus uses), violet (tests) and dashed grey (presumed). Per column at most 30 cards are drawn, the heading says how many more there are. A reference that closes a cycle is listed but not drawn backwards.
- Hit list: kind (call, read, write, test and so on), the calling symbol, file and line, and the certainty. F8 and Shift+F8 step through the hits, `Enter` (or a double click) shows the selected hit, or the symbol in focus if none is selected, in the explorer.
- Side panel: symbol, signature, counts (places, files, layers), the impact of a change (symbols reached, layers, public interfaces), the note that constants, properties and interface members work the same, the accuracy and the button "Open in Explorer".
- **Certainty (ADR 0033, proposed):** every hit says whether level 2 verified it. The pill of a hit reads `verified`, `unchecked` (found by name resolution only) or `presumed` (level 1 itself is unsure); the side panel counts them apart and, while no indexer exists (`level2: "unavailable"`, which is the case until ADR 0033 is implemented), says that the usages and the impact are preliminary and may be incomplete or contain wrong hits. Verified hits are sorted first. The field `verified` and the state are there already, so the indexer packages only have to fill them.
- Deviations from the mock: the hit list shows file and line, not the code line (reading code needs `read_source` and the token, which this screen does not use; follow-up issue); "Alt + click on a symbol in the code" has no effect because the source column has no symbol spans yet (same reason as in the explorer); the hit list shows the callers (or, for "what is called", the callees), not both; the graph shows type uses of the symbol as cards too; "Open in editor" is not on this screen; the footer shows the connection, not "Reference index" or the query time.

## Visual and keyboard checks (WP-43)

Two Playwright files check the screens against blueprint 15.4 and 17.1; both run in the `e2e` job.

- `web/e2e/ui-visual.spec.ts` (visual): every screen at 1440 x 900 in the light theme. The mocks are plain HTML with inline styles, so the test opens the mock in a browser, adds the one rule the design canvas adds (`box-sizing: border-box`), measures landmarks (top bar, rail, status bar, the columns, side panels, the dialog of the quick search) and compares them with the same landmarks of the real screen, within one pixel. It also compares the colours of the frame. Texts and numbers of the mocks are sample data and are not compared. A deviation listed in this file is not tested; a new one fails the test, so the list stays true.
- `web/e2e/ui-keyboard.spec.ts` (keyboard): the initial state, the click rules of the tree, the arrow keys, `Ctrl+Left`, `Ctrl+.`, hotkeys while typing, `Ctrl+K`, the prefixes `t:` `m:` `>`, `Esc`, `Enter` and `F` on the map. `N`, `R`, `U` and `F8` are in `projects.spec.ts` and `ui-usages.spec.ts`.
- `UPDATE_SCREENSHOTS=1 npm run e2e` writes the screens to `docs/screenshots/visual/` (`projects`, `explorer`, `search`, `scan`, `usages`, `graph`, all in Hell) for the review by eye. The older per-theme screenshots stay as they are.
- What the check found and fixed: the side column of the scan screen was 340 px wide instead of 300; the quick search was 860 px wide and 12 vh from the top instead of 880 px and 96 px, its input row was not 56 px high, its preview column was 260 px instead of 320 px and the active chip was blue instead of black; the frame grew with long content, so the status bar left the screen (the mocks scroll the main area only). The acceptance run of M3 (`docs/acceptance/m3.md`) found one more: with many hits the quick search shrank its input row to 27 px; the test now searches with 40 hits.
- Not covered because the feature does not exist yet: `G` in the explorer, `Ctrl+Enter`, `Alt+Up` and `Alt+Down`, the search prefixes `f:` and `@` (issue #171; `Esc` cancels a scan: #153; `Alt+click`: #158).
- Deviations seen by eye and not measured: the search field of the top bar has no magnifier icon (issue #167); the quick search has no chips `f:` Files and `@` all projects, no index size in the footer, and its preview shows neither the documentation nor LOC, member count and MI (the hit of `search.query` does not carry them; issue #178).
- Settings (13): the theme card fills the width of the content and the language card is 560 px wide, as in the mock (landmarks tested); the editor card has no mock and is 560 px wide too.

ESLint (`web/eslint.literals.config.js`, issue #26): `consts.ts` is free of the literal rules only as the value of a `const`; `let`, `var`, other statements and function bodies are refused there. A number above 1 is refused everywhere else even when it is assigned to a `const` (`no-magic-numbers` alone lets that pass). `web/src/tests/eslint-literals.test.ts` runs the config on snippets.
