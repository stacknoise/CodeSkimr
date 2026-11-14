# 0036 – Qualified names of languages with namespaces do not start with the file

Status: proposed

## Context

The symbol extraction (`crates/adapters/parsing/src/symbols.rs`) gives every file one `module` symbol named after its path, and builds the qualified name of every declaration from the qualified name of its parent. So the file is the first segment of every name. For Rust, Python, TypeScript and JavaScript that is right: the path of the file is the module path (`crates::domain::symbol::Symbol`), and layers (ADR 0031) name those prefixes.

For languages that declare their own namespaces it is wrong. The file `Calculator.cs` with `namespace Demo; class Calculator` gives `Calculator.Demo.Calculator` instead of `Demo.Calculator` (issue #137). Java (`package`) and Kotlin (`package`) are built the same way. Consequences: a layer with the namespace `Demo` owns nothing (`Layer::owns`), the namespace level and the depth of the dependency map show the file name as the first segment, the explorer lists `Calculator.Demo` as a namespace, and names differ from what the compiler, SCIP indexes (ADR 0033) and the developer use.

## Decision

1. **A pack key `file_in_names`** in `language.toml` (boolean, default `true`). With `true` nothing changes (Rust, Python, TypeScript, JavaScript).
2. **With `false` (C#, Java, Kotlin) the file is not part of the names.** The first namespace or top-level declaration starts the qualified name: `Demo.Calculator`, `Demo.Calculator.Add`. A declaration outside any namespace has its own name as qualified name.
3. **The module symbol keeps the full path of the file with its extension** as qualified name (`app/Calculator.cs`, written with the separator of the path, not of the pack). It cannot collide with a type of the same name, and it stays unique per file. Declarations are children of the namespace or, without one, of the module symbol (`parent` is unchanged), only the name no longer carries the module.
4. **Two declarations with the same qualified name** (partial classes, a type in two files) stay two symbols with their own ids. Names are not unique keys; code that needs one symbol uses the id.
5. **Existing snapshots** keep their old names until the next scan; the next scan of a project writes the new ones. No migration is needed because names are derived data.
6. **Golden tests** of the three packs are regenerated; layer examples in docs and tests for C# change from `Calculator.Demo` to `Demo`.

## Consequences

- Layers, the namespace level and its depth, the explorer outline and `get_symbol` show the names a C# developer expects. SCIP symbols match names directly.
- References and the resolver already work on ids and on names found by lookup; the implementation checks with tests that nothing relies on the file prefix (golden files, resolver tests, SCIP name matching).
- Changing the setting of a pack changes all names of its language, so it is part of the pack and tested with golden files.
- The blueprint text (section 6, `Symbol`, qualified name) is not changed; it never required the file as first segment.
