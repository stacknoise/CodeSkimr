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
- `src/features`: settings (theme cards with preview, language, editor), connection screen, projects (screen 1, WP-44), explorer (screen 2, WP-45).
- Themes Hell, Petrol (default) and Dunkel are CSS variables generated from `design/tokens.toml` by `cargo run -p codeskimr_codegen`; never edit the generated files.
- No literals in the code: values live in `consts.ts` files, texts in `locales/`.

## Projects (screen 1, WP-44)

The screen calls three commands over `POST /api/v1/commands/{id}` (`shared/api/commands.ts`): `projects.list`, `projects.add` and `projects.rescan` (see `docs/server.md`).

- `projects.ts` keeps the list, the failure of the last action and the scans started in this tab. While a scan runs, the list is read every 1.5 s, so a running scan shows up as "scan running".
- Keys: `N` focuses the source field, arrow keys move the selection, `Enter` opens the project (it becomes the active project, `shared/project/active.ts`, and the explorer is shown), `R` scans it again. Typing in a field and Ctrl, Alt or Meta combinations never trigger them.
- Drop: a `file://` address or text dropped on the window goes into the source field. A browser does not tell the path of a dropped folder in every system; the screen then says so and asks to paste the path.
- Deviations from the mock: a small "scan again" button sits next to the status pill (so the re-scan has a mouse path, not only the key R); the status "new commits" and the scan duration of earlier sessions are missing because the data model has neither (the duration of scans started in this tab is shown); language colours come from the layer tokens.

## Explorer (screen 2, WP-45)

The screen shows the active project (`shared/project/active.ts`) with two commands: `explorer.outline` (the tree) and `explorer.symbol` (one type with breadcrumb, metrics and member table); both need only the read right (see `docs/server.md`). The source column of the mock follows in WP-45b, which also brings the `read_source` right. Screenshots: `docs/screenshots/explorer-hell.png`, `explorer-petrol.png`, `explorer-dunkel.png`.

- Tree: layer (with its colour dot), namespace, type (nested types are rows below their type), then the groups "Fields etc.", "Methods" and nested types with their members. The tree is virtualised: only the rows in view (plus a margin) are in the DOM. It is one `tree` with `aria-activedescendant`; the group rows are expandable `treeitem`s as well. Click rules (blueprint 15.4): the caret only opens and closes; a click on an unselected type selects it; a click on the selected type opens or closes it; groups open and close on their own; selecting from inside the tree never changes what is open; at the start everything is closed and the overview is shown. The filter keeps the types whose name contains the text, with their layer and namespace opened.
- **Deviation from blueprint 15.4, rule 1:** the blueprint says a click on an unselected type "selects and expands it" but also that a selection from inside the tree never changes the open state. The explorer follows the second rule: the first click only selects, a click on the selected type opens or closes it. Issue #140 asks for a decision and, if needed, an ADR.
- Keys: Up and Down move the cursor, Right opens (or goes to the first child), Left closes (or goes to the parent), Home and End jump, Enter and Space select or toggle, `Ctrl+Left` closes everything, `Ctrl+.` shows the selection.
- Details: breadcrumb, name with kind and visibility, documentation, ten metric tiles, member table with the chips All, Properties, Methods and "public only". The colour of a metric and of the complexity of a member comes from the thresholds in `config/default.toml` (`metrics.*`), which `explorer.outline` delivers; the number is always shown too, and the tooltip of a column or tile names the limits. A failure of the symbol keeps the tree.
- Deviations from the mock: the source column is missing (WP-45b); the buttons "Usages", "Graph" and "Open in editor" are shown but off (`aria-disabled`, with a hint) until their screens exist; the tab "Dependencies", the metric "Test names the type" and the base type in the header are missing (the data model has none of them); the access column shows the visibility only; the usage count is not a link yet; the signature column shows a dash where the language pack stores none; kind and visibility are shown as the keywords of the language, not translated; the C# qualified name of a type carries a prefix (issue #137).

## Quick search (screen 4, WP-46)

`Ctrl+K` (`Cmd+K` on a Mac) or the search field of the top bar opens the overlay (blueprint B.4, mock `Palette.dc.html`). It searches the newest scan of the active project with the command `search.query` (read right, no source text; see `docs/server.md`). Screenshots: `docs/screenshots/search-hell.png`, `search-petrol.png`, `search-dunkel.png`.

- Input with hit count and time, chips All, Types, Members and Commands (a prefix in the input wins: `t:` types, `m:` members, `>` commands; the chip is reset to All each time the overlay opens), grouped hits with the match marked (the first place where the text occurs in the name), preview of the chosen hit with its actions, footer with the fuzzy-search hint. The list is virtualised and holds at most 50 hits.
- Keys: Up and Down choose, `Enter` opens the hit in the explorer (the rows down to it open, because external navigation opens nodes), `Tab` and `Shift+Tab` change the chip, `Ctrl+C` (`Cmd+C` on a Mac) copies the qualified name when no text of the input is selected; `Ctrl+K` while the overlay is open keeps text and hits, `Esc` closes and gives the focus back to where it was.
- Commands (`>`): the screens (projects, explorer, settings) and "scan project again" for the active project.
- Deviations from the mock: no group "Files" and no scope `@` (all projects), because the index holds names of symbols only and the UI works on one project; the preview shows kind, qualified name and file but not the doc and the metrics; "Show in graph" and "Open in editor" are listed but off until the graph screen exists and the hit carries a line; the key hints of the preview say `Ctrl` (the `Cmd` variants work on a Mac); the time shown is the round trip of the command, not the time of the query alone; the hit of a member of a nested type opens the member within the nested type.
