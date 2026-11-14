# HTML export

`codeskimr export` writes one **self-contained HTML file** with the explorer, the dependency map, the usages and the quick search of a scanned project. The file needs no server and no network: open it from disk, mail it or put it into a wiki. Use it for documentation and for planning changes with people who do not run CodeSkimr.

## Use

```
codeskimr export <project> --html <file>
```

- `<project>` is the id of the project (as in the project list) or the folder or Git address it was scanned from.
- The project must have been scanned; the **newest snapshot** is exported. Export while no scan is running, otherwise the file can mix two snapshots.
- The command prints where the file went and how many symbols it holds. A file that already exists is replaced.

The command needs the built export page of the UI: `npm run build` in `web/` makes it (`dist/export/export.html`), The page is part of the program; if there is a UI folder (`ui.dir`, see [Installation](installation.md)), the page from that folder is used instead. If it is missing in both places, the command stops with `EXP-201`.

## What is in the file

- The tree of layers, namespaces and types.
- For every type and member: breadcrumb, metrics, signature, documentation comment, file and line range.
- The usages of every symbol (callers and callees, as deep as `mcp.max_depth` allows, tests included; the toolbar filters work in the page) and the impact of a change.
- The dependency map as CodeSkimr laid it out.
- Name, commit and date of the scan.

The page looks and behaves like the app. Navigation uses the address fragment (`#/graph`), so it works from a file.

## What is not in it

- **No source text.** The code pane says so. Source text needs the right to read source and is not exported.
- No folder or address of the project, no installation token, nothing from the keystore.

> **Check before sharing:** signatures and documentation comments are text from your source. If a project keeps secrets or internal remarks in them, they are in the page too. Look at an export before it leaves your team.

## Unchecked usages stay visible

Every usage keeps its mark (verified, unchecked, presumed) and the page shows the same notice as the app when no indexer checked anything ([Usages](usages.md#certainty-verified-unchecked-presumed)). A strip under the top bar counts verified and unchecked direct usages of the whole export, so a page that is read months later still says how far it can be trusted.

## Limits

- It is a snapshot. Scanning, adding projects and settings are not part of it.
- Only types and the members of types are listed, like in the explorer; free functions outside a type are not.
- The quick search is a plain match on names (start, part, letters in order), not the ranking of the app.
- The size grows with the references, because the usages of every symbol are embedded. Large projects give files of many megabytes.

## Errors

| Code | Meaning | What to do |
|---|---|---|
| `PRJ-201` | the project is not registered | add and scan it first |
| `MCP-203` | the project has not been scanned | scan it |
| `EXP-201` | the built export page is missing or unusable | use a program built with the UI (`npm run build` in `web/`, then build the program), check `ui.dir` |
| `EXP-601` | the file cannot be written | check the path, the rights and the free space |
