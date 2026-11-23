# HTML export

The export is available two ways: the **Export HTML** button in the app and the command `codeskimr export`. Both write one **self-contained HTML file** with the explorer, the dependency map, the usages and the quick search of a scanned project. The file needs no server and no network: open it from disk, mail it or put it into a wiki. Use it for documentation and for planning changes with people who do not run CodeSkimr.

## In the app

Open the project (Projects, choose the row, Enter). The top bar has the button **Export HTML** next to the name and the branch. A click makes the page from the newest scan, and the browser downloads it as `<project name>.html` into its download folder. Nothing is sent anywhere else. The button is grey while the project has not been scanned. The quick search does the same (`Ctrl K`, then `>` and "Export project as HTML"). While the export runs, the button shows the seconds that have passed, and **Cancel** stands next to it. Cancel and the time limit only end the wait of the app; the server may finish the export anyway. A large project can take several minutes because the usages of every symbol are read. After five minutes the app stops waiting and says so next to the button; the command `codeskimr export` in a console has no time limit. If the export fails, the reason with its error code appears next to the button.

## On the command line

```
codeskimr export <project> --html <file>
```

- `<project>` is the id of the project (as in the project list) or the folder or Git address it was scanned from.
- The project must have been scanned; the **newest snapshot** is exported. Export while no scan is running, otherwise the file can mix two snapshots.
- The command prints where the file went and how many symbols it holds. A file that already exists is replaced.

The command needs the built export page of the UI: `npm run build` in `web/` makes it (`dist/export/export.html`). The page is part of the program; if there is a UI folder (`ui.dir`, see [Installation](installation.md)), the page from that folder is used instead. If it is missing in both places, the command stops with `EXP-201`.

## What is in the file

- The tree of layers, namespaces and types.
- For every type and member: breadcrumb, metrics, signature, documentation comment, file and line range.
- The usages of every symbol (callers and callees, direct usages only, tests included; the toolbar filters work in the page; the depth select is not offered) and the impact of a change.
- The dependency map as CodeSkimr laid it out.
- Name, commit and date of the scan.

The page looks and behaves like the app. Navigation uses the address fragment (`#/graph`), so it works from a file.

## What is not in it

- **No source text.** The code pane says so. Source text needs the right to read source and is not exported.
- No folder or address of the project, no installation token, nothing from the keystore.

> **Check before sharing:** signatures and documentation comments are text from your source. If a project keeps secrets or internal remarks in them, they are in the page too. Look at an export before it leaves your team.

## Unchecked usages stay visible

Every usage keeps its mark (verified, unchecked, presumed) and the page shows the same notice as the app when level 2 checked none of the hits ([Usages](usages.md#certainty-verified-unchecked-presumed)). A strip under the top bar counts verified and unchecked direct usages of the whole export, so a page that is read months later still says how far it can be trusted.

## Limits

- It is a snapshot. Scanning, adding projects and settings are not part of it.
- Only types and the members of types are listed, like in the explorer; free functions outside a type are not.
- The quick search is a plain match on names (start, part, letters in order), not the ranking of the app.
- The size grows about linearly with the references, because the direct usages of every symbol are embedded. As a guide: about 1.4 MB for 400 symbols, about 5 MB for 1,300 and about 64 MB for 7,400 (the whole CodeSkimr source). Very large projects give files of many megabytes. Walks through other symbols (depth 2 and more) are only in the app: they grow much faster than the project and made the file too big to open.
- A project with more than `export.max_references` references (default 600000, a file of about 150 MB) is not exported; the export ends at once with `EXP-502`. Raise the limit in your own config file.
- Only one export per project runs at a time; a second one ends with `EXP-501`.

## Errors

| Code | Meaning | What to do |
|---|---|---|
| `PRJ-201` | the project is not registered | add and scan it first |
| `MCP-203` | the project has not been scanned | scan it |
| `EXP-201` | the built export page is missing or unusable | use a program built with the UI (`npm run build` in `web/`, then build the program), check `ui.dir` |
| `EXP-501` | an export of the project is already running | wait until it has ended |
| `EXP-502` | more references than `export.max_references` | raise the limit or export a smaller project |
| `EXP-601` | the file cannot be written | check the path, the rights and the free space |
