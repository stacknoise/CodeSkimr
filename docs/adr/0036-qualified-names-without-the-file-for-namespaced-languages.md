# 0036 – Qualified names of languages with namespaces do not start with the file

Status: proposed

## Context

The symbol extraction (`crates/adapters/parsing/src/symbols.rs`) gives every file one `module` symbol named after its path, and builds the qualified name of every declaration from the qualified name of its parent. So the file is the first segment of every name. For Rust, Python, TypeScript and JavaScript that is right: the path of the file is the module path (`crates::domain::symbol::Symbol`), and layers (ADR 0031) name those prefixes.

For languages that declare their own namespaces it is wrong. The file `Calculator.cs` with `namespace Demo; class Calculator` gives `Calculator.Demo.Calculator` instead of `Demo.Calculator` (issue #137). Java (`package`) and Kotlin (`package`) are built the same way. Consequences: a layer with the namespace `Demo` owns nothing (`Layer::owns`), the namespace level and the depth of the dependency map show the file name as the first segment, the explorer lists `Calculator.Demo` as a namespace, and names differ from what the compiler, SCIP indexes (ADR 0033) and the developer use.

## Options

1. Shorten the name only in the display and store the full name. Every consumer that matches names (layers, the namespace graph, SCIP, `get_symbol`) would still need the short form, so the logic would be written twice and drift.
2. Let the pipeline strip the file for all languages. It does not know which language has real namespaces; that is a property of the language pack.
3. Keep the name and store the namespace path in an extra field. Layers, search and tools read the qualified name; a second name would have to be kept in step everywhere.
4. Change the name in the pack (chosen below): one place, golden-tested, and every reader sees the same name.

## Decision

1. **A pack key `file_in_names`** in `language.toml` (boolean, default `true`). With `true` nothing changes (Rust, Python, TypeScript, JavaScript).
2. **With `false` (C#, Java, Kotlin) the file is not part of the names.** The first namespace or top-level declaration starts the qualified name: `Demo.Calculator`, `Demo.Calculator.Add`. A declaration outside any namespace has its own name as qualified name.
3. **The module symbol keeps the full path of the file with its extension** as qualified name (`app/Calculator.cs`, written with the separator of the path, not of the pack). It cannot collide with a type of the same name, and it stays unique per file. Declarations are children of the namespace or, without one, of the module symbol (`parent` is unchanged), only the name no longer carries the module.
4. **Two declarations with the same qualified name** (partial classes, a type in two files) stay two symbols with their own ids. Names are not unique keys; code that needs one symbol uses the id.
5. **Existing snapshots and incremental scans.** Snapshots share the rows of unchanged files (ADR 0032), so a file counts as unchanged only if its content hash is equal **and** the value of `file_in_names` of its language equals the one recorded with the previous snapshot. The scan records that value per language in the snapshot. For a snapshot without the record (all snapshots before this decision), the files of C#, Java and Kotlin count as changed once, so the first scan after the update rewrites all their names and a project never mixes `Calculator.Demo.X` with `Demo.Y`. The record needs a small schema change (a new column or table, so a new schema version with a migration); the implementation PR plans for it. No data migration is needed because names are derived data.
6. **Golden tests** of the three packs are regenerated; layer examples in docs and tests for C# change from `Calculator.Demo` to `Demo`.

## Consequences

- **Equal names become more common.** C# types without a namespace and Kotlin functions at file level in one package can now have the same qualified name. Names are not keys: search, the map and the usages screen use ids (#234). `get_symbol` by name returns the first symbol in id order and adds the number of other symbols with the same name (`ambiguous`), so a caller can switch to the id; `get_symbol` by id is always unambiguous.
- Layers, the namespace level and its depth, the explorer outline and `get_symbol` show the names a C# developer expects. SCIP symbols match names directly.
- References and the resolver work on ids and on names found by lookup. The implementation PR checks every place that reads a qualified name, as a checklist: `Layer::owns` (ADR 0031), the namespace graph (`namespace_graph::enclosing`) and its depth merge, `type_neighbours` (owner type by name prefix), the resolver lookup in `crates/application/src/resolve/`, the SCIP name match (`crates/adapters/scip`), the explorer outline, the MCP tool `get_symbol` and the golden files of the three packs.
- Changing the setting of a pack changes all names of its language, so it is part of the pack and tested with golden files.
- The blueprint text (section 6, `Symbol`, qualified name) is not changed; it never required the file as first segment.
