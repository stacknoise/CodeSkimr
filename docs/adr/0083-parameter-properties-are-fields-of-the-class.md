# 0083 – Parameter properties are fields of the class (`@lift`)

Status: accepted

## Context

`constructor(private x: T)` and `constructor(readonly y = 1)` declare a field of the class. ADR 0022 left them out of the symbol tree, and ADR 0047 repeated the reason: symbols are nested by the syntax tree, so such a field would become a child of the constructor, not of the class, with a wrong parent and a wrong qualified name. The result was visible in every TypeScript project that uses the style (Angular, NestJS, many libraries): `this.x.foo()` found no field `x` and so no type for its receiver, and `x` was missing in the list of members (issue #325, rest of #81).

## Decision

1. **New capture `@lift` in `symbols.scm`.** It marks a declaration that is written inside a symbol but belongs to the *parent* of that symbol. The declaration gets that parent and the qualified name `<parent>.<name>`, and its kind and visibility are decided as if it stood in the parent (a `field` of a class).
2. **The enclosing symbol stays the lexical parent.** The symbol table has two relations: the semantic parent (`Symbol.parent`, used for names and the member list) and the lexical one (used to find the innermost symbol around a byte offset). A lifted symbol changes only the first. A reference in the body of the constructor is still assigned to the constructor, not to the class.
3. **TypeScript and TSX packs.** A constructor parameter with an accessibility modifier (`public`, `protected`, `private`) or `readonly`, required or optional, is a `field`. Visibility follows the modifier, `readonly` alone is public (as for class fields). Plain parameters stay no symbols. JavaScript has no parameter properties.
4. The limit "parameter properties are not symbols" in ADR 0022, section 3, and the sentence about them in ADR 0047 no longer hold.

## Consequences

- New scans of TypeScript and TSX projects have more `field` symbols (corpus `detailed-energy-card`: 130 → 141, an independent count with the TypeScript compiler API gives 11 parameter properties). The golden files and `corpus/typescript/corpus.toml` are updated.
- A type use in the type of such a parameter (`private repo: Repo`) is now assigned to the new field, not to the constructor.
- Accuracy (150 symbols, seed 1, TypeScript service as reference): recall 78.5 % → 80.8 %, precision of the certain hits stays 100 %, precision over all hits 29.8 % → 28.6 % (more `probable` hits on the newly found fields). See `docs/accuracy.md`.
- No change of the data model or the storage. The names of the new fields do not exist in older snapshots, the next scan is a normal rescan.
- Other packs can use `@lift` where a language declares a member inside another declaration (Kotlin primary-constructor properties, Scala, PHP constructor promotion) without a change of the Rust code; they are not part of this ADR.
