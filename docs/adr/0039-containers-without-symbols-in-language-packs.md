# 0039 – Containers without a symbol: `impl` blocks belong to their type

Status: proposed

## Context

ADR 0026 made Rust `impl` blocks invisible: they are no symbols, so the functions in them became methods of the enclosing module (`analyzer::new` instead of `analyzer::TreeSitterAnalyzer::new`). Issue #85 lists the consequences:

- Two `impl` blocks of one module with methods of the same name give two symbols with the same qualified name, although they belong to different types.
- `impl Trait for Type` reports `implement` from the module. The resolver builds the type hierarchy from the `inherit` and `implement` references of types, so it cannot learn from Rust impls that `Type` implements `Trait`.
- A call `Type::method()` finds the method only through the name search (probable), not through its owner.

Recall matters more than speed (usage analysis is the basis for planning changes, the product owner's decision of 13 November), so the owner of a method has to be right.

## Options

1. Make the `impl` block a symbol (a kind `impl`). A kind is a fact about a declaration and an `impl` declares nothing; the type would get two symbols with the same name, and every view (explorer, graph, search) would list blocks next to types.
2. Let the resolver derive the owner from the name of the method. The name carries no type today (that is the problem), and a resolver rule would have to exist per language.
3. A block that is no symbol, but names the type it belongs to (chosen). One new capture in the pack vocabulary, no new symbol kind, nothing special in the resolver.

## Decision

1. **A new capture `@container`** in `symbols.scm`. It marks a node that is no symbol; its `@name` capture is the simple name of the type the block belongs to. Without `@symbol.<kind>` the match is no declaration. A pack that does not use `@container` is unchanged.
2. **Members of a container** get the qualified name `<scope>::<Type>::<member>` (with the separator of the pack), where `<scope>` is the qualified name of the enclosing symbol.
3. **Parent.** If a type symbol (a kind for which `SymbolKind::is_type` holds) with that name is declared in the same scope earlier in the file, it is the parent of the members, and the container stands for it in references: an occurrence in the head of the block (the implemented trait, the type itself) is reported from the type, so `impl Trait for Type` gives `implement` from `Type`. If there is no such type (it lies later in the file, in another module or in another file) the parent stays the enclosing symbol and only the name carries the type.
4. **Visibility** is not taken from the type: the frame of the container does not use the parent defaults of the type kind, so a method of `impl Enum` without `pub` stays private (the enum default `public` is for its variants).
5. **Rust.** `impl Type`, `impl Trait for Type` and `impl<T> Type<T>` are containers. Heads that are not a plain or generic type name (`impl Trait for &Type`, `impl a::Type`, `impl Trait for [T]`) have no container; their members stay with the enclosing module as before.
6. **Naming revision.** Names of Rust methods change, so the naming rules recorded with a snapshot (ADR 0036) get a revision text. A scan whose revision differs from the previous snapshot is a full scan, so a project never mixes the old and the new names of one type.
7. **Golden tests and corpus.** The golden files of the Rust pack are regenerated; the counts of the Rust corpus do not change, two samples name the type as parent.

## Consequences

- The methods of a type are its children when the `impl` stands next to the type, which is the usual case in Rust. Metrics roll up to the type (cc and mi of a struct now include its methods).
- The resolver needs no change for the same-module case: references from the type and children of the type are what it already uses for C#, Java and Kotlin.
- **Limits (follow-up).** An `impl` in another module or file, or before its type, still has no link from the type to its methods in the symbol tree; the name carries it. A later step can link them by qualified name across files. Macro-generated impls are not seen.
- Level 2 (ADR 0033) maps definitions by place and name, not by the parent, so it is unaffected.
- Go and C++ methods written outside their type have the same problem (issue #65); that package may use the same idea with another capture and has its own ADR.
