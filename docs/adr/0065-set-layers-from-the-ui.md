# 0065 – Setting layers from the UI: a confirmed choice is written to `.codeskimr.toml`

Status: proposed

## Context

ADR 0031 made the project file the only source of layers and ruled out any detection: "A guess would be wrong for some project and silently so". Without `[[layers]]` the explorer puts every type under "(no layer)", and the dependency map is empty. In the first test of the preview release the owner met exactly this: the user has to know the file format and the namespaces before the screens show anything. Wish of the owner (2026-11-16): the UI shows the detected namespaces, the user builds layers from them, and the result is written to the configuration.

## Decision

1. **The configuration stays the source of truth.** The UI never changes how layers are resolved. It only edits the `[[layers]]` tables of the project's `.codeskimr.toml` and the scan reads them as before (ADR 0031, decision 4). Layers apply from the next scan, so the UI saves and scans again in one step.
2. **A choice is not a detection.** The UI lists the namespaces of the newest scan as a tree of prefixes (separator `.` or `::`) and proposes nothing: no layer exists until the user ticks prefixes and names a layer. A tick on a prefix selects it with everything below it, as the prefix does in the file. This keeps decision 5 of ADR 0031.
3. **Two commands.** `layers.get` (right `read`) gives `editable`, `scanned`, the layers of the project and the namespaces of the newest scan with their type counts and the layer that owns each. `layers.set` (right `manage`, not read-only) takes a project id and the full list of layers (name, namespaces; outermost first) and replaces the layers of the file. Only a folder project is editable; a Git project answers `PRJ-101`, because its file lives in a cache, not in the repository.
4. **What the write touches.** The server replaces only the `[[layers]]` tables: a table runs to the next table header, everything else in the file stays byte for byte. The new tables are appended at the end. Comments inside the replaced tables are lost; this is said in the UI. A `may_use` of a layer is kept for a layer of the same name, minus the names of layers that no longer exist (ADR 0054); the UI does not edit `may_use` (it stays in the file).
5. **Validation before the write.** The new text is parsed and resolved like any project file (`CFG-101` for an invalid or duplicate name, a missing namespace, an unknown `may_use`). A file that is not valid TOML is not touched (`CFG-102`); a file over 1 MiB is not read. The write goes to a new file beside it (created exclusively, never following a link) and is renamed over the project file, so a reader never sees half a file. A failure to write is the new code `CFG-104` (`project_file_not_writable`).
6. **Where the UI offers it.** A screen "Layers" (route `/layers`), reached from the settings, from the overview of the explorer and from the empty dependency map, each of the last two with a hint only while the project has no layers.
7. **Security.** The command writes inside the registered project folder only, the path is built on the server from the registered folder and the fixed file name, never from the input. It needs `manage` and the installation token like every command that changes something. It does not run or read anything of the project.

## Consequences

- A user without knowledge of the file format reaches working layers, explorer groups and a dependency map in one screen.
- A project file that the user maintains by hand stays valid and keeps its other settings; only the layer tables are rewritten, in the order of the screen.
- Blueprint appendix A gets `CFG-104`; the error code catalogue and the manuals (de, en) are updated.
- Not decided here: editing `may_use` in the UI (issue to follow), suggesting layers from folders or from the dependency graph (stays ruled out by ADR 0031), Git projects (they would need a place to write).
