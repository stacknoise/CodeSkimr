# 0016 – C# grammar, public C# corpus and additions to the pack vocabulary

Status: proposed

## Context

WP-24 delivers the first real language pack. ADR 0011 left three things open for C#: the public corpus project for CI, the grammar dependency, and whatever the pack format of WP-23 cannot express yet. Blueprint 8.2 says a language must not need changes outside `languages/<lang>/`; the built-in grammar registry and a few query captures are an exception that has to be written down.

## Decision

### 1. Grammar

`tree-sitter-c-sharp` 0.23 (MIT) becomes a dependency of `codeskimr_parsing` and the grammar `csharp` is registered next to `rust`. Full-tier grammars are compiled into the binary (blueprint 8.2). The change carries the label `new-dependency`.

### 2. Public C# reference

[ardalis/GuardClauses](https://github.com/ardalis/GuardClauses), MIT, pinned at commit `5434b262a40ba26dae6e7f2cc93c6097b9cc4824`: 55 files, about 8,400 lines, modern C# (file-scoped namespaces, nullable types, local functions, explicit interface implementations, a byte order mark). It is small enough for the CI budget. It covers namespaces, classes, interfaces, structs, enums, enum members, constructors, methods, local functions, properties and fields; records, delegates, constants, events, operators and indexers are covered by the golden tests only.

Expected counts live in `corpus/csharp/corpus.toml`. They were measured with the pack and checked against an independent per-file count of the source; every file where the two disagreed (12 of 55) was reviewed by hand and the pack was right each time (the independent count missed explicit interface implementations, signatures with attributes in the parameters and constructors with initializers). A sample of symbols and references is checked by hand. Updating the commit is a deliberate change with new expected values (ADR 0011). The private reference stays local; the test can be pointed at it with `CODESKIMR_CORPUS_DIR` and prints counts only.

Alternatives: Humanizer (155,000 lines, too slow for a PR check), MediatR and AutoMapper (licence changed to commercial).

### 3. Additions to the pack vocabulary

C# needs four things the WP-23 format cannot express. They are generic and change nothing for packs that do not use them:

- `@modifier` (repeatable) with `static_keywords` and `[modifier_kinds]` in `language.toml`: C# has any number of modifier keywords in any order, and a `const` field is a constant.
- Several matches of one declaration are merged (union of modifiers and doc lines) instead of "first wins", so a query may use `(modifier)* @modifier` and `((comment)* @doc . (decl))` safely.
- `@scope_end`: a file-scoped namespace holds the rest of the file but is a sibling of its members in the syntax tree.
- `visibility.unrated_kinds` (a namespace has no visibility) and `visibility.parent_defaults`: C# defaults depend on the place (top-level type internal, member private, interface member public). `[doc_summary]` extracts the `<summary>` of XML documentation.

No symbol kind is added. Mapping decisions (delegate is a `type_alias`, a positional record parameter is a `public` property, `protected internal` is `protected`) are listed in `languages/csharp/language.toml`.

## Consequences

- `crates/adapters/parsing` changes beyond the pack: `grammar.rs`, `symbols.rs`, `manifest.rs`, a new `visibility.rs`, and the test helpers.
- `metrics.scm` follows ADR 0015, but the adapter does not read it yet; a golden test runs the query itself. Delivering the counts through `FileFacts` is a separate package.
- Qualified names contain the file module as prefix for every language (WP-23). For C# this makes them longer than the namespace-based names users know; changing it is a decision for the reference resolver.
- Reads, writes and `implement` are level-1 approximations in `references.scm`; accuracy targets for references are measured when the resolver exists.
