# Language packs: how the parsing adapter reads them

`codeskimr_parsing` (WP-23) implements the `FileAnalyzer` port with Tree-sitter. Parsing and extraction are one stage; the syntax tree never leaves the crate (ADR 0013). The pack format follows blueprint 8.1.

## Files of a pack

| File | Required | Purpose |
|---|---|---|
| `language.toml` | yes | metadata and rules (below); unknown keys are rejected |
| `symbols.scm` | yes | declarations to symbols |
| `references.scm` | no | occurrences of names; without it a file yields no pending references |

`language.toml` keys: `id`, `grammar` (built-in grammar name), `tier` (`full` or `basic`), `extensions`, `separator` (inside qualified names), `comment_nodes` (node kinds that are comments, used for the line classification), `doc_prefixes` (a comment captured as `@doc` counts only if it starts with one; the prefix is removed), optional `manifest_files` and `test_patterns` (read by later stages) and the table `[visibility]` with `default` and `keywords` (keyword text to visibility label).

## Capture vocabulary

- `symbols.scm` (a declaration found by several patterns is stored once): `@symbol.<kind>` marks the whole declaration (`<kind>` is a symbol kind label such as `class` or `enum_member`), `@name` its simple name (required). Optional: `@visibility`, `@static`, `@doc` (may repeat; lines are joined), `@signature`.
- `references.scm`: `@reference.<kind>` marks an occurrence (`<kind>` is a reference kind label). `@name` narrows the target text; without it the marked node is the target.

A capture outside this vocabulary, an invalid query or an unknown grammar fails the load with `PRS-301`.

## Results

- One `module` symbol per file (qualified name = path without extension, folders joined with the separator); every declaration is nested below the innermost enclosing declaration.
- Pending references point at the innermost enclosing symbol; occurrences outside any declaration belong to the module.
- `loc` counts lines that contain at least one non-comment token. Lines inside multi-line tokens (strings) count as code.
- Failures: `PRS-101` (encoding; UTF-8 and UTF-16 with BOM are read), `PRS-201` (no pack), `PRS-301` (invalid pack), `PRS-501` (parse timeout), `SRC-201` (file cannot be read).

## Not part of this package

Cyclomatic complexity and Halstead counts are not extracted yet: the `FileFacts` port carries no metric inputs and issue #15 (granularity, counting rules) is open. The parse time limit is a constructor parameter until it is wired to the config (#32).
