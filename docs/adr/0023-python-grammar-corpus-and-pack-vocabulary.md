# 0023 – Python grammar, public Python corpus and additions to the pack vocabulary

Status: proposed

## Context

WP-61 delivers the full-tier Python pack. ADR 0011 left the public corpus project open ("a public repository of the product owner"), and Python needs three things the pack format of ADR 0016 cannot express: visibility by naming convention, the kind of a `def` that depends on its parent, and docstrings, which are the first statement of a body instead of a comment. Blueprint 8.2 allows changes outside `languages/<lang>/` for the grammar registry and a few generic captures if they are written down.

## Decision

### 1. Grammar

`tree-sitter-python` 0.25 (MIT) becomes a dependency of `codeskimr_parsing` and the grammar `python` is registered with the full-tier grammars. The change carries the label `new-dependency`.

### 2. Public Python reference

The product owner has no Python project of his own that could serve (the only Python in his public repositories is a few helper scripts; the other repositories are TypeScript, Astro and a private C# project). ADR 0011 allows a well-known permissively licensed project in that case: [pallets/click](https://github.com/pallets/click), BSD-3-Clause, pinned at commit `e38188e9e205005d5af5f9e59df51904c984586a`. The library has 17 files and about 12,800 lines of modern, typed Python with decorators, properties, nested functions, dataclass-like classes and conditional definitions. Tests, docs and examples are not scanned. It is small enough for the CI budget.

Expected counts live in `corpus/python/corpus.toml`. They were measured with the pack and checked against an independent count with the `ast` module of CPython. Every kind agrees except constants: the independent count finds 4 more UPPER_CASE assignments, all inside `if`/`else` blocks at module level; the pack only reports constants that are direct statements of the module (see limits). Hand-checked samples and references are listed in the file.

The corpus test now walks every `corpus/<language>/corpus.toml`; `[project]` gets `language`, `extensions` and optional `skip_suffixes`. `CODESKIMR_CORPUS_DIR` points to a checkout of the C# project as before, or of the language named in `CODESKIMR_CORPUS_LANGUAGE`.

### 3. Additions to the pack vocabulary

All three are generic and change nothing for packs that do not use them:

- `[[visibility.name_rules]]` in `language.toml`: visibility by the spelling of the name (`prefix`, optional `suffix`, `visibility`). The first matching rule counts, after the keywords and before the defaults. Python: `__x__` public, `__x` private, `_x` internal, everything else public.
- `[kinds_by_parent.<kind>]`: a declaration of one kind gets another kind inside a parent of the given kind. Python: a `function` inside a `class` is a `method`. The query can then capture every `def` the same way. Kind overrides from `[modifier_kinds]` (`@property`, `__init__`) are applied first.
- Documentation text is split into lines, each trimmed, and the result is trimmed at both ends, so a captured docstring (`@doc` on the string content) reads like a comment block.

No symbol kind is added.

## Mapping decisions

Listed in `languages/python/language.toml`: `__init__` is the constructor; the decorators `property`, `cached_property`, `getter`, `setter` and `deleter` make a `property` (a getter and its setter are two symbols with the same name, like overloads in C#); `@staticmethod` is static; assignments directly in a class body are `field`s; UPPER_CASE assignments directly in a module are `constant`s. A symbol's span starts at `def`, not at its decorators (the decorator is a sibling node and the body check of ADR 0015 needs the definition node).

## Limits (follow-up work)

- Module docstrings are not read (the module symbol has no query), constants and fields inside `if`/`try` blocks are not found, instance attributes assigned through `self.x` are not symbols, enum members of an `Enum` class are fields.
- `references.scm`: reads and writes are level-1 approximations; locals are the parameters, direct assignments in a function body, loop variables and `as` targets; a class body is not a scope; `self` and `cls` are the own receiver, `super()` the base.
- `metrics.scm` follows ADR 0015: `match` arms count per `case` except `case _`; a guard (`if` after the pattern) and `for … else` add nothing; comprehensions count per `for` clause and per `if` clause.

## Consequences

- `crates/adapters/parsing` changes beyond the pack: `grammar.rs`, `consts.rs`, `manifest.rs`, `pack.rs`, `symbols.rs`, `visibility.rs`, and the test helpers (`tests/common/metrics.rs` is shared by the metrics tests of both languages).
- The adapter still does not read `metrics.scm` per ADR 0016/#44; the golden test runs the query through the analyzer.
