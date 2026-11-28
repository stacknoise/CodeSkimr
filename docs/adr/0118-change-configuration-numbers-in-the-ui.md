# 0118 – Change numbers and switches of the configuration in the UI

Status: accepted (card of the product owner on 2026-11-28: gap "Einstellungen bearbeiten (E)" and "Variante A" of mock 45; follow-up of ADR 0092 and ADR 0103)

## Context

ADR 0092 says that what the console can do, the UI can do. The console changes any value of the configuration with `--set key=value` and in `config.toml`. The tab "Effective settings" of the UI only shows the values (ADR 0098), and ADR 0103 added a file of the UI (`ui-settings.json`) that holds one value: the memory limit per indexer. ADR 0103 decision 7 left room: "Other values may use the file later, each with its own decision". This is that decision.

Two kinds of value are different from the rest. Values that open a door (`level2.enabled`, the server port and bind address, the editor link and its allowed schemes, the token) must stay with the owner of the files (ADR 0071, ADR 0033). Values that only tune the program (limits, thresholds, switches of analyses) are harmless to change from the page that already has the permission `manage`.

## Options

1. **A dialog per row (variant A of mock 45).** In the tab "Effective settings" every row gets "Change"; a dialog edits one value.
2. **Groups of fields on a tab "Configuration" (variant B).** Several values are edited and saved together.
3. **Show only.** Nothing changes; the principle of ADR 0092 stays unmet.

## Decision

Option 1.

1. **Which values.** Only a number (whole or with a fraction) or a switch whose shipped default is a number or a switch, in one of these tables: `scan`, `conventions`, `similarity`, `export` (these count from the next scan or check) and `metrics`, `mcp`, `snapshots`, `watcher`, `annotations`, `graph`, `compare`, `keys` (read when the program starts). Everything else is not editable here: `level2`, `server`, `editor`, `ui`, `budgets` and `layers`, every text, list, path and secret. The allowlist is code (`ui_settings::editable`); a new table needs a decision of its own.
2. **The file.** `ui-settings.json` gets a second member `values`, a flat map `"table.key": number | switch`, written and read like the rest of the file (ADR 0103 decision 1). When the file is read, an entry that is no longer editable or whose kind does not fit the default is dropped, so no file can keep the program from starting. The size limit of the file stays; the layer `ui` of the override chain is built from the file as TOML text.
3. **Command.** `ui_settings.set_value` (permission `manage`, input `key` and `value`, where `null` forgets the saved value). A key outside the allowlist or a value of another kind is `API-101`. The configuration with the new value must resolve (`Config::resolve`, so a value out of range or a default above its maximum is `CFG-101`) before anything is written, so a saved value can never make the next start fail. If the file cannot be written: `APP-201`.
4. **When it counts.** The values of the tables read per scan or check count from the next scan (the layers are read again at every call, as for the memory limit, ADR 0103 decision 4). The values read at the start of the program keep their running value; `settings.effective` shows the running value and names the saved one as `pending`, with the note that it counts from the next start.
5. **What `settings.effective` adds.** Each row carries `default` (the shipped default), `edit` (`scan`, `start`, `cli` for a value a `--set` flag sets, `locked` for what the UI may not change) and `pending`. The view re-reads the layers at every call, so a saved value shows at once.
6. **The dialog.** "Change" opens a dialog with the key, a field for the value (a choice on/off for a switch), the kind and the default as hint, the pill "Applies from the next scan" or "…from the next start", and the buttons Cancel, Back to default (only for a value saved in the app) and Save. A value the UI may not change keeps a grey "Change" button with the reason as tooltip and note: "Security relevant: only in the configuration file", or "Set with --set at the start, the command line wins".
7. **Precedence is unchanged.** defaults → user file → saved by the UI → project file → `--set`. A value saved here wins over `config.toml`, loses against a flag, and the pill "Saved in the app" shows where a value comes from.

## Consequences

- The user file stays the owner's; whoever also edits it sees the app value win and can read the origin in the table.
- A saved value that the configuration would refuse is never written; a hand-edited file with a bad entry does not stop the program, the entry is ignored.
- Whoever has the permission `manage` can change limits and thresholds of this computer. That is the person who could edit the user file; a team edition (ADR 0019) decides in its own ADR who may do it.
- The command reads the file, changes one entry and writes it back without a lock: the last writer wins (as in ADR 0103).
- Security-relevant values are not touched, so there is no new way to widen what the program may read, run or expose.
