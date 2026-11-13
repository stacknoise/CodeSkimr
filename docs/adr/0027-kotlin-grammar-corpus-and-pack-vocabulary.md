# 0027 – Kotlin grammar, public Kotlin corpus and one addition to the pack vocabulary

Status: proposed

## Context

WP-63 delivers the full-tier Kotlin pack. ADR 0011 left the public corpus project open. The pack format of ADR 0016, 0022, 0023 and 0025 covers almost everything Kotlin needs: KDoc is the `[doc_block]` of ADR 0022, `enum`, `data`, `annotation` and `const` change the kind through `[modifier_kinds]`, a function inside a type becomes a method through `[kinds_by_parent]`. One thing is missing: the Kotlin grammar has no `body` field, so the adapter could not tell a function with a body from an abstract one and would have measured none (ADR 0015: callables without a body are not measured).

## Decision

### 1. Grammar

Two Tree-sitter grammars exist for Kotlin on crates.io. `tree-sitter-kotlin-ng` 1.1 (the one of the tree-sitter-grammars organisation) was tried first and rejected: it reports a syntax error for a primary constructor with modifiers on its own line (`class Builder` newline `internal constructor(...)`), which the explicit-API style of KotlinPoet uses everywhere, and the error swallows the whole class body. `tree-sitter-kotlin-sg` 0.4 (MIT, the maintained fork of the fwcd grammar, used by ast-grep) parses all 39 files of the corpus without an error. It becomes a dependency of `codeskimr_parsing`, and the grammar `kotlin` is registered with the full-tier grammars. The change carries the label `new-dependency`.

### 2. Public Kotlin reference

The product owner has two Kotlin repositories (`chicken_vs_snakes`, `haac-android`); they were not available to this work package. ADR 0011 allows a well-known permissively licensed project in that case: [square/kotlinpoet](https://github.com/square/kotlinpoet), Apache-2.0, pinned at commit `f4e88d3655b0f5f795186ea8cf3cb5a8ceb6db01`. The library sources of the `commonMain` and `jvmMain` source sets of the `kotlinpoet` module are scanned: 39 files with classes, objects, companion objects, sealed classes, data classes, enums, extension functions, `const val` and KDoc. Tests, the other platforms, the interop modules, docs and build files are skipped. The product owner can replace the reference by one of his repositories later; that is a deliberate change of `corpus/kotlin/corpus.toml` with new counts.

Expected counts live in `corpus/kotlin/corpus.toml`. They were checked against an independent count with the PSI parser of the Kotlin compiler (`kotlin-compiler-embeddable` 2.0.21, parse only): every kind agrees (36 classes, 20 interfaces, 5 enums, 77 enum entries, 2 data classes, 15 secondary constructors, 129 functions, 539 methods, 385 properties, 10 constants, 39 packages).

### 3. Addition to the pack vocabulary

`body_nodes` in `language.toml`: syntax node kinds that make a child a body when the grammar has no `body` field. A callable with such a child is measured. Kotlin: `function_body` and, for a secondary constructor, `statements`. A pack that does not use it is unchanged. No symbol kind is added.

## Mapping decisions

Listed in `languages/kotlin/language.toml`:

- A `package` header is a `namespace` that closes with the file.
- `class` is `class`; `interface` and `fun interface` are `interface`; `enum class` is `enum`; `annotation class` is `interface` (as in Java); `data class` is `record`; `object` and a named `companion object` are `class`. An unnamed companion is not a symbol; its members belong to the surrounding class, which is how code refers to them (`Foo.MAX`).
- A function inside a class, interface, enum, record or enum entry is a `method`, every other one a `function` (top-level and local functions, functions of anonymous objects). Extension functions are named by the function.
- A secondary constructor is a `constructor` named like its class. The primary constructor is not a symbol; its `val` and `var` parameters are `property` symbols.
- A property declared in a file, class, object or enum is a `property`, `const val` is a `constant`. Local properties are no symbols. Accessors are no symbols.
- An enum entry is an `enum_member`; `typealias` is a `type_alias`.
- Visibility: `public` is the default (also in interfaces); `internal` maps to the domain's `internal`, `protected` and `private` as written. A package has none; enum entries are public.
- KDoc is a `/** */` comment directly above the declaration; the description up to the first line that starts with `@` is the documentation. Annotations sit inside the declaration and do not break the link.

## Limits (follow-up work)

- Not symbols: parameters, local variables, initializer blocks, type parameters, destructuring declarations, the members of anonymous objects as methods (they are functions of the enclosing callable).
- Kotlin has no `static`; members of objects are not marked static.
- `references.scm`: reads and writes are level-1 approximations; `Foo()` is a call of `Foo` (the constructor is named like its class); a qualified type `a.b.C` reports every part as a type use; the declared type of `val x = Foo()` is taken from a capitalized call; labels, `this@Outer` and delegated properties (`by lazy`) are not interpreted.
- `metrics.scm` follows ADR 0015: `if` expression counts like `if` and replaces the conditional expression, a `when` entry counts once also with several conditions, `else`, `try`, `finally`, `?.`, `is`, `as`, `!!` add nothing; `&&`, `||` and the Elvis operator `?:` count.
- Query patterns put alternatives inside the parent node (issues #77 and #81).

## Consequences

- `crates/adapters/parsing` changes beyond the pack: `grammar.rs`, `consts.rs`, `manifest.rs` and `pack.rs` (`body_nodes`), `symbols.rs` (`has_body`), and the new golden test `tests/kotlin_metrics.rs` that reuses the shared helpers.
- The adapter still does not read `metrics.scm` per ADR 0016/#44; the golden test runs the query through the analyzer.
