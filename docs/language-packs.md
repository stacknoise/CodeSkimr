# Language packs: how the parsing adapter reads them

`codeskimr_parsing` (WP-23) implements the `FileAnalyzer` port with Tree-sitter. Parsing and extraction are one stage; the syntax tree never leaves the crate (ADR 0013). The pack format follows blueprint 8.1.

## Files of a pack

| File | Required | Purpose |
|---|---|---|
| `language.toml` | yes | metadata and rules (below); unknown keys are rejected |
| `symbols.scm` | yes | declarations to symbols |
| `references.scm` | no | occurrences of names; without it a file yields no pending references |

`language.toml` keys: `id`, `grammar` (built-in grammar name), `tier` (`full` or `basic`), `extensions`, `separator` (inside qualified names), `comment_nodes` (node kinds that are comments, used for the line classification), `doc_prefixes` (a comment captured as `@doc` counts only if it starts with one; the prefix is removed), optional `identifier_nodes` and `literal_nodes` (node kinds that syntax fingerprints abstract, see below), optional `manifest_files` and `test_patterns` (read by later stages), `static_keywords` (modifier keywords that make a declaration static), `[modifier_kinds]` (modifier keyword to symbol kind label, for example `const = "constant"`), `[doc_summary]` with `open` and `close` tags (the doc of a symbol is the text between them, whitespace collapsed; without the tags the whole comment) and the table `[visibility]`: `default`, `keywords` (keyword text to visibility label; several keywords together, such as `protected internal`, may have an entry of their own) and `parent_defaults` (parent symbol kind label to the visibility of a declaration without keyword).

## Capture vocabulary

- `symbols.scm` (a declaration found by several patterns is stored once): `@symbol.<kind>` marks the whole declaration (`<kind>` is a symbol kind label such as `class` or `enum_member`), `@name` its simple name (required). Optional: `@modifier` (may repeat; every modifier keyword, the pack decides what each one means), `@visibility` (like `@modifier`), `@static`, `@doc` (may repeat; lines are joined), `@signature` and `@scope_end` (the node whose end closes the scope of the declaration, for a file-scoped namespace that holds the rest of the file). Several matches of one declaration are merged, so repeated captures such as `(modifier)* @modifier` are safe.
- `references.scm`: `@reference.<kind>` marks an occurrence (`<kind>` is a reference kind label). `@name` narrows the target text; without it the marked node is the target.

A capture outside this vocabulary, an invalid query or an unknown grammar fails the load with `PRS-301`.

## Results

- One `module` symbol per file (qualified name = path without extension, folders joined with the separator); every declaration is nested below the innermost enclosing declaration.
- Pending references point at the innermost enclosing symbol; occurrences outside any declaration belong to the module.
- `loc` counts lines that contain at least one non-comment token. Lines inside multi-line tokens (strings) count as code.
- Failures: `PRS-101` (encoding; UTF-8 and UTF-16 with BOM are read), `PRS-201` (no pack), `PRS-301` (invalid pack), `PRS-501` (parse timeout), `SRC-201` (file cannot be read).

## The C# pack (WP-24)

`languages/csharp/` is the first real pack (full tier). `language.toml` documents the mapping onto symbol kinds. Decisions worth knowing:

- Qualified names start with the file module (`Folder.File.Namespace.Type.Member`), like every pack.
- `protected internal` and `private protected` count as `protected` (the domain has no combined value). A top-level type without keyword is `internal`, a member without keyword `private`, a member of an interface or enum `public`.
- A positional record parameter is a `public` property. A `const` field is a `constant`. A delegate is a `type_alias`. Not symbols: parameters, locals, lambdas, finalizers, conversion operators, primary constructor parameters of classes.
- The doc of a symbol is the text of its `<summary>`; inner markup such as `<see cref="X"/>` stays as written. A comment without `<summary>` (for example `<inheritdoc/>`) is kept whole.
- `references.scm` is level 1 (names only). `new Foo()` is a type use; a base type is `inherit` (the resolver can refine it to `implement`); reads are reported for names in value positions and for receivers of member accesses.
- `metrics.scm` holds the captures of ADR 0015 (`@comment`, `@decision`, `@operator`, `@operand`); `tests/csharp_metrics.rs` checks one case per counting rule. `highlights.scm` is for the UI.
- Golden tests: `languages/csharp/tests/*.cs` with the expected result in `<file>.json` (`CODESKIMR_BLESS=1 cargo test -p codeskimr_parsing --test language_packs` rewrites them; review the diff).
- Corpus: `corpus/csharp/corpus.toml` pins a public project (ADR 0016); `tests/corpus.rs` fetches it into `target/corpus/` with `git` and compares counts and a hand-checked sample. `CODESKIMR_CORPUS_DIR` points the test at another checkout (for the private reference: only locally, `CODESKIMR_BLESS=1` prints the counts).

## Syntax fingerprints (WP-34)

`TreeSitterAnalyzer::with_fingerprints(&config.similarity)` turns fingerprinting on; `FileFacts::fingerprints` then holds one entry per function, method or constructor with at least `similarity.min_tokens` tokens. The declaration is flattened into leaf tokens: comments are dropped, nodes listed in `identifier_nodes` all become one identifier token, nodes in `literal_nodes` become one literal token (and are not descended into), everything else keeps its node kind. The domain function `winnow` hashes all n-grams of `similarity.ngram_size` tokens and keeps the minimum of every window of `similarity.window_size` hashes. A shared run of `window_size + ngram_size - 1` tokens always yields a shared fingerprint. `Fingerprints::similarity` (Jaccard) and `containment` compare two bodies; `find_similar_code` and the duplicate check of `check_conventions` use these same functions. The hash is implemented in the domain (no new dependency, values stable across releases). A pack without the two keys still fingerprints, but names and values then influence the result. Persisting the fingerprints and the queries on top are separate packages.

## Not part of this package

The adapter does not read `metrics.scm` yet: the `FileFacts` port carries no metric inputs (ADR 0015 describes the delivery). The reference resolver does not exist yet either. The parse time limit is a constructor parameter until it is wired to the config (#32).
