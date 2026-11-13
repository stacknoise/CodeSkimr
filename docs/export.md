# HTML export (WP-75)

`codeskimr export <project> --html <file>` writes one self-contained HTML file with the explorer, the dependency map, the usages and the quick search of a scanned project. The file needs no server and no network: it can be opened from disk, mailed or put into a wiki. Its purpose is documentation and the planning of changes.

## Use

```
codeskimr export <project> --html <file>
```

`<project>` is the id of the project (`projects.list`) or the folder or Git URL it was scanned from. The project must have been scanned. The newest snapshot is exported. The command prints where the file went and how many symbols it holds. A file that exists is replaced.

| Code | Meaning |
|---|---|
| `PRJ-201` | the project is not registered |
| `MCP-203` | the project has not been scanned |
| `EXP-201` | the built export page is missing, see below |
| `EXP-601` | the file cannot be written |

## Structure

- The action is the registered command `export.html` (`crates/app/src/ui_commands/export.rs`, permission `read`). The CLI runs it through `Registry::execute`; the local server exposes it as `POST /api/v1/commands/export.html` and answers `{ "html": …, "symbols": … }`. It reads through the commands `explorer.outline`, `explorer.symbol`, `usages.query` and `graph.layers`, so the page shows exactly what the browser UI shows and nothing is computed twice.
- The page is the built UI. `npm run build` in `web/` makes a second build (`vite build --mode export`, `web/vite.export.ts`) with the script, the styles and all assets inlined into `dist/export/export.html`. That file has one place for the data (`__CODESKIMR_EXPORT_DATA__`); the command finds it below `ui.dir`, in `export/export.html`, and fills it. Without it the command stops with `EXP-201`.
- In that build two modules of the UI are swapped: `shared/api/commands.ts` (the call of the server) for `export/commands.ts`, which answers from the embedded data, and `app/router.ts` for a router that keeps the screen in the address fragment (`#/graph`), because a file cannot take another path. The screens do not know the difference.
- The data is JSON in a `<script type="application/json">` element. `<` is written as `<`, so no text of the project can end the element. The bundle is made safe for the same reason (`</script`, `<!--`).

## What the page holds

The tree of layers, namespaces and types; for every type and member the breadcrumb, metrics, signature, documentation comment, file and line range; for every one of them its usages (callers and callees, as deep as `mcp.max_depth` allows, tests included, with the filters of the toolbar applied in the page); the impact of a change; the dependency map as the server placed it; the name, commit and date of the scan.

**Not in it:** source text (the declaration bodies, `explorer.source`; the pane says so), the folder or address of the project, the installation token or anything from the keystore. Source text would need `read_source` (ADR 0032); a later option would have to ask for that on purpose.

**Note:** signatures and documentation comments are text from the source. If a project keeps secrets or internal remarks in them, they are in the page too. Check a page before it leaves the team.

## Unchecked usages

Every usage keeps its `verified` and `confidence` fields. The usages screen of the page shows the same pills as the app (`unchecked`, `presumed`) and the same notice that no indexer verified anything (ADR 0032). The strip under the top bar of every screen counts the verified and unchecked direct usages of the whole export, so a page that is read months later still says how far it can be trusted.

## Limits

- The page is a snapshot. Scanning, adding projects and the settings are not part of it.
- Only types and the members of types are listed, like in the explorer; free functions outside a type are not.
- The quick search is a plain match on names (start, part, letters in order), not the ranking of the server.
- The size grows with the references: the usages of every symbol are embedded. Large projects give files of many megabytes.
- The commands read the newest snapshot one after another; a scan that finishes during an export can mix two snapshots. Export while no scan runs.
