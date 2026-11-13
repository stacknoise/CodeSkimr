# 0022 – TypeScript and JavaScript grammars, public TypeScript corpus and JSDoc in the pack vocabulary

Status: proposed

## Context

WP-60 delivers the full-tier language packs for TypeScript and JavaScript (blueprint 8.2). ADR 0011 names the corpus source (public repositories of the product owner), ADR 0016 is the pattern for a language ADR. Three things need a decision: the grammars and how the packs are cut, the corpus, and what the pack format cannot express yet (JSDoc blocks).

## Decision

### 1. Grammars and packs

`tree-sitter-typescript` 0.23 (MIT, delivers the TypeScript and the TSX grammar) and `tree-sitter-javascript` 0.25 (MIT) become dependencies of `codeskimr_parsing`, registered next to `rust` and `csharp`. Full-tier grammars are compiled in. The change carries the label `new-dependency`.

A pack has exactly one grammar, and TypeScript, TSX and JavaScript are three grammars that do not accept each other's files (`<T>x` casts against JSX, no type nodes in JavaScript). So there are three packs: `typescript` (`ts`, `mts`, `cts`), `tsx` (`tsx`) and `javascript` (`js`, `mjs`, `cjs`, `jsx`; the JavaScript grammar parses JSX). The `tsx` pack has the same `symbols.scm` and `metrics.scm` as `typescript` plus JSX patterns in `references.scm`; a test keeps the copies equal. A `queries` key in `language.toml` that lets a pack reuse another pack's files would remove the copies; it is not part of this package.

### 2. Public TypeScript reference

[stacknoise/detailed-energy-card](https://github.com/stacknoise/detailed-energy-card), MIT, a public repository of the product owner (ADR 0011), pinned at commit `c9ffe3a32a1c2a1d5e6831154628acff860e453c`: 64 TypeScript files of `src` with about 14,000 lines (Lit web components, models, editors). Tests, end-to-end tests, development stubs and scripts are skipped. Expected counts are in `corpus/typescript/corpus.toml`. They were measured with the pack and checked against an independent count with the TypeScript compiler API (same mapping rules, different parser): every kind agrees. The first run differed by 5 fields and 1 constant, all tagged templates (`css`…``), which the independent count had left out. A hand-checked sample of symbols and one reference is in the file. The corpus has no enum and no namespace; the golden tests cover them.

### 3. Mapping

Written down in `languages/typescript/language.toml`. Decisions worth knowing:

- A module-level `const` is a `constant`, a `const` (or `let`) with an arrow or function expression is a `function`, also inside functions (like a local function in C#, so that its decision points and Halstead counts count for it). The symbol of such a function is the function expression, so its span starts after the `=`.
- Getters and setters are `property` (a pair gives two symbols). A class field with an arrow function is a `method`. Interface members are `property` and `method`; overload signatures are no symbols.
- Visibility: `export` is public, a top-level declaration without it is private, members are public unless `private` or `protected` says otherwise.
- Limits, all as data in the pack: `#private` fields get the default (public) because they carry no keyword; parameter properties of constructors (`constructor(private x: T)`) are not symbols; `let` and `var` at module level, `export { x }` lists and anonymous default exports are not symbols; names that are strings, numbers or computed are skipped.
- Metrics follow ADR 0015: `?.` does not count, a `case` counts when it has a body, `&&=`, `||=`, `??=` count.

### 4. Additions to the pack vocabulary

One generic addition, which changes nothing for packs that do not use it: `[doc_block]` in `language.toml` (`open`, `close`, `line_prefix`, optional `stop_at`) for block documentation comments. The documentation is the text between `open` and `close` without the `*` at the start of each line, up to the first line that starts with `stop_at` (`@` for JSDoc, so `@param` and `@returns` are not part of the description). With `doc_block` and without `doc_prefixes`, only block comments that start with `open` are documentation.

A query learning (also relevant for other packs): inside a parent node, a quantified `(comment)* @doc .` followed by the declaration matches nothing when there is no comment, and an alternation of parent nodes in front of a child pattern is a sibling sequence, not an alternation of parents. The packs use `(comment)+ @doc .` with a second pattern without doc, and `[(parent child) (parent child)]`.

## Consequences

- `crates/adapters/parsing` changes beyond the packs: `grammar.rs`, `consts.rs`, `manifest.rs`, `pack.rs`, `symbols.rs`, the workspace dependencies, and the test helpers shared with the Python package.
- `references.scm` is level 1 (blueprint 9.2): `extends` is `inherit`, `implements` is `implement`, `new Foo()` and JSX components are type uses, imports are the imported names. Accuracy targets for references are measured when the resolver can be run on the corpus.
- Three packs share most of their queries by copy until a pack can include another pack's files.
