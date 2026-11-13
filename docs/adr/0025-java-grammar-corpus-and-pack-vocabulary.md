# 0025 – Java grammar, public Java corpus and one addition to the pack vocabulary

Status: proposed

## Context

WP-62 delivers the full-tier Java pack. ADR 0011 left the public corpus project open ("a public repository of the product owner"). The pack format of ADR 0016, 0022 and 0023 covers almost everything Java needs: Javadoc is the `[doc_block]` of ADR 0022, a `static final` field becomes a constant through `[modifier_kinds]`, and members of an interface get their visibility from `[visibility.parent_defaults]`. One thing is missing: enum constants and record components are always public, whatever the parent is, but the enum also holds package-private methods, so the default of the parent kind cannot express it.

## Decision

### 1. Grammar

`tree-sitter-java` 0.23 (MIT) becomes a dependency of `codeskimr_parsing` and the grammar `java` is registered with the full-tier grammars. The change carries the label `new-dependency`.

### 2. Public Java reference

The product owner has no Java project (his public repositories are TypeScript, Astro and a private C# project; the private project is never used). ADR 0011 allows a well-known permissively licensed project in that case: [google/gson](https://github.com/google/gson), Apache-2.0, pinned at commit `a961b0fe088f8558d00cc1b56e506a3656907a91`. The `gson` module has 87 Java files and about 19,700 lines with classes, nested and anonymous classes, interfaces, enums with bodies, annotation interfaces, generics and Javadoc. The other modules, tests and the Maven template are not scanned. It is small enough for the CI budget.

Expected counts live in `corpus/java/corpus.toml`. They were measured with the pack and checked against an independent count with the `com.sun.source` tree API of the JDK (parse only, no attribution): every kind agrees. Gson targets Java 8, so the corpus has no records; the golden files in `languages/java/tests/` cover records, compact constructors and annotation interfaces.

### 3. Addition to the pack vocabulary

`[visibility.kind_defaults]` in `language.toml`: the visibility of a declaration without a keyword, by its own kind. It applies after the keywords and name rules and before `[visibility.parent_defaults]`. Java: `enum_member` and `property` (record components) are public. A pack that does not use it is unchanged. No symbol kind is added.

## Mapping decisions

Listed in `languages/java/language.toml`:

- A `package` declaration is a `namespace` that closes with the file, like the file-scoped namespace of C#.
- `class` is `class`; `interface` and `@interface` are `interface`; `enum`, `record` map to their own kinds; an enum constant is an `enum_member`; a record component is a public `property`.
- A method or annotation element is a `method`; a constructor or compact constructor is a `constructor`.
- A field that is `static final`, or any field of an interface, is a `constant`; every other field is a `field`. A declaration such as `int a, b;` makes one symbol per declarator. `final` is captured as a modifier only together with `static` (`[modifier_kinds] final = "constant"`).
- No keyword means package-private, which the domain calls `internal`; members of an interface are public unless they say `private`.
- Javadoc is a `/** */` comment directly above the declaration; the description up to the first line that starts with `@` is the documentation. Inline tags such as `{@link X}` and HTML stay as written. Other block comments (licence headers) and line comments are not documentation.
- Members of anonymous classes and enum constant bodies are symbols with the enclosing method or enum constant as parent; local classes are symbols of their enclosing method. Lambdas count for their callable (ADR 0015).

## Limits (follow-up work)

- Not symbols: parameters, local variables, initializer blocks, type parameters, the `module` of `module-info.java`, package documentation in `package-info.java`.
- The implicit `static` of nested enums, records and interfaces and the implicit `final` of interface fields are not set; interface fields are constants by their parent.
- `references.scm`: reads and writes are level-1 approximations; `new Foo()` is a type use of `Foo` (as in C#); `this(...)` and `super(...)` are not reported; generic bounds are type uses of the bound only; the locals are parameters, local variables, loop variables, catch parameters, resources and patterns, and `var x = new Foo()` tells the type of `x`.
- `metrics.scm` follows ADR 0015: a `case` group counts when it has statements (a label without statements falls through into the next group), an arrow `case` counts per rule, `default` and the guard `when` add nothing; `catch` counts once also for `A | B`; `try`, `finally`, `synchronized`, `assert` and `instanceof` patterns add nothing.
- Query patterns put alternatives inside the parent node (issues #77 and #81).

## Consequences

- `crates/adapters/parsing` changes beyond the pack: `grammar.rs`, `consts.rs`, `manifest.rs`, `visibility.rs` (with a unit test), and the new golden test `tests/java_metrics.rs` that reuses the shared helpers.
- The adapter still does not read `metrics.scm` per ADR 0016/#44; the golden test runs the query through the analyzer.
