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

The screen shows the active project (`shared/project/active.ts`) with two commands: `explorer.outline` (the tree) and `explorer.symbol` (one type with breadcrumb, metrics and member table); both need only the read right (see `docs/server.md`). The source column of the mock follows in the next work package, which also brings the `read_source` right.

- Tree: layer, namespace, type, then the groups "Properties & fields" and "Methods" with their members. A type is read when it is opened. The filter keeps the types whose name contains the text, with their layer and namespace opened. Keys in the tree: arrows move and open, `Ctrl+Left` closes everything, `Ctrl+.` shows the selection.
- Details: breadcrumb, name with kind and visibility, documentation, metric strip, member table with the chips All, Properties, Methods and "public only". The colour of a metric and of the complexity of a member comes from the thresholds in `config/default.toml` (`metrics.*`); the number is always shown too.
- Deviations from the mock: the source column is missing for now; the buttons "Usages", "Graph" and "Open in editor" are shown but off until their screens exist; the tab "Dependencies" and the metric "Test names the type" are missing (the data model has neither); the metric strip shows the member count and the six stored metrics, so a project without coupling metrics shows a dash; the usage count in the member table is not a link yet; the signature column shows a dash where the language pack stores none.
