# 0103 – Settings the UI saves in the data folder

Status: accepted (card of the product owner on 2026-11-26: "Einstellungsdatei der Oberfläche"; follow-up of ADR 0092 and ADR 0100, issue #752)

## Context

ADR 0092 says that what the console can do, the UI can do. The memory limit per indexer (`level2.indexer_memory_mb.<indexer>`, ADR 0100) can be set in the user configuration file and with `--set`, but the UI only shows it in the effective settings. The settings screen has no way to change a value of the configuration, and the program never writes the user configuration file: the file belongs to its owner, and the master switch `[level2] enabled` in it is read again at every call precisely because the program must not set it itself (ADR 0071, decision 8).

The UI does save two other choices on the server: the theme and the language, in `preferences.json` in the data folder (ADR 0030, decision 8).

## Options

1. **A file of the UI in the data folder.** The UI saves the value in a small file of its own, and the configuration reads it as one more layer.
2. **Write the user configuration file.** The program edits the TOML file of the user. Comments and formatting can be lost, and the file would no longer be only the owner's.
3. **Show only.** The field shows the value in effect and says how to set it in the file. No new write path, but the principle of ADR 0092 stays unmet.

## Decision

Option 1.

1. **The file.** `ui-settings.json` in the data folder holds `{"indexer_memory_mb": {"<indexer>": <megabytes>}}`. It is written like `preferences.json`: to a new file beside it that is created exclusively and renamed over the old one, readable by the owner only. A missing, damaged, too large (over 4 KiB) or foreign file counts as no settings. Only a known indexer name and a value up to 1,048,576 MB are kept; anything else is dropped when the file is read, so that no file can keep the program from starting.
2. **A layer of the configuration.** At every start the file becomes the layer `ui` of the override chain: defaults → user file → **saved by the UI** → project file → command line flags. A value saved by the UI wins over the user file, because it is the later and deliberate choice made in the app, and loses against a `--set` flag, which is the strongest. The project file still cannot set the key (the `level2` table of the project file is dropped, ADR 0033). `settings.effective` names the origin `ui`; the effective settings screen shows it as "Saved in the app".
3. **Commands.** `ui_settings.get` (permission `read`) and `ui_settings.set_indexer_memory` (permission `manage`, input `indexer` and `megabytes`, where `null` forgets the saved value). An unknown indexer or a value above the limit is `API-101`, an unwritable data folder `APP-201`.
4. **A new value counts from the next scan.** A scan started in the app already reads the configuration anew, and the running server reads the limit again for every run of an indexer, also for the `rescan` of an agent (the layers are read again; a `--set` flag still wins). A scan that is running keeps the value it started with, and a separately started `codeskimr mcp` reads the file at its start. The field says so. Other values of the configuration still count from the next start.
5. **An upper bound.** `level2.indexer_memory_mb.<indexer>` may not exceed 1,048,576 MB (1 TiB) in any layer; a larger value is `CFG-101`. Until now a value was only checked for the name of the indexer.
6. **The field.** The tab "Analysis" of the settings screen gets one field per indexer with the value in effect, a note that 0 follows `level2.max_memory_mb`, and a button to go back to the default. A value that a `--set` flag sets is shown but not editable.
7. **The master switch stays out.** `[level2] enabled` and the confirmation of a project are not part of this file and never will be (ADR 0071). Other values may use the file later, each with its own decision.

## Consequences

- The user file stays the owner's. Whoever edits it and also saves a value in the app sees the app value win; the effective settings show where each value comes from.
- A settings file that is edited by hand and holds something the configuration would refuse does not stop the program; the offending entry is ignored.
- A value saved while the program runs counts from the next scan. If the files cannot be read at that moment, the run keeps the value the program started with.
- Whoever has the permission `manage` can change the memory limit of this computer. That is the same person who could edit the user file, so nothing is lifted. A team edition (ADR 0019) has to decide in its own ADR who may do it.
- `ui_settings.set_indexer_memory` reads the file, changes one entry and writes it back without a lock. Two calls at the same moment can overwrite each other: the last writer wins. One user with one page makes this a rare case, so it is accepted.
- The same file can hold more settings that the UI may change (ADR 0092), each added with its own check at the write and at the read. Numbers and switches of the tuning tables follow in ADR 0118.
