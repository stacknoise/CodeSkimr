# 0083 – Parameter properties are fields of the class (`@lift`)

Status: accepted

## Context

`constructor(private x: T)` and `constructor(readonly y = 1)` declare a field of the class. ADR 0022 left them out of the symbol tree, and ADR 0047 repeated the reason: symbols are nested by the syntax tree, so such a field would become a child of the constructor, not of the class, with a wrong parent and a wrong qualified name. The result was visible in every TypeScript project that uses the style (Angular, NestJS, many libraries): `this.x.foo()` found no field `x` and so no type for its receiver, and `x` was missing in the list of members (issue #325, rest of #81).

## Decision

1. **New capture `@lift` in `symbols.scm`.** It is meant for a declaration that forms a symbol without children of its own (a parameter, not a function): the engine takes the enclosing frame off the stack for the declaration and puts it back after it. It marks a declaration that is written inside a symbol but belongs to the *parent* of that symbol. The declaration gets that parent and the qualified name `<parent>.<name>`, and its kind and visibility are decided as if it stood in the parent (a `field` of a class).
2. **The enclosing symbol stays the lexical parent.** The symbol table has two relations: the semantic parent (`Symbol.parent`, used for names and the member list) and the lexical one (used to find the innermost symbol around a byte offset). A lifted symbol changes only the first. A reference in the body of the constructor is still assigned to the constructor, not to the class.
3. **TypeScript and TSX packs.** A constructor parameter with an accessibility modifier (`public`, `protected`, `private`), `readonly` or `override`, required or optional, is a `field`; decorators in front of it (`@Inject(TOKEN) private x: X`, `@Optional() private y?: Y`) do not matter. Visibility follows the modifier, `readonly` alone is public (as for class fields). Plain parameters stay no symbols. JavaScript has no parameter properties.
4. The limit "parameter properties are not symbols" in ADR 0022, section 3, and the sentence about them in ADR 0047 no longer hold.

## Consequences

- New scans of TypeScript and TSX projects have more `field` symbols (corpus `detailed-energy-card`: 130 → 141, an independent count with the TypeScript compiler API gives 11 parameter properties). The golden files and `corpus/typescript/corpus.toml` are updated.
- A type use in the type of such a parameter (`private repo: Repo`) is now assigned to the new field, not to the constructor.
- Accuracy, measured on the same symbols before and after (all 1342 symbols of the corpus that the tool draws from, TypeScript service as reference, `tools/accuracy/measure.py typescript --samples 10000`): the uses of every existing symbol are unchanged (0 of 1342 differ in found or confirmed uses), none is lost, three symbols are new (the others of the 11 new fields have a name that starts with `_`, which the tool leaves out). Those three: 38 of our uses, 5 the reference confirms, all 5 found; 33 are `probable` hits of the common name `start` on other objects (level 1 does not know the receiver). The certain hits stay at 100 %. See `docs/accuracy.md`.
- The revision of the naming rules goes up (5, as in ADR 0042), so the next scan of every project is a full scan and existing projects get the new fields at once, not only when a file changes.
- No change of the data model or the storage.
- Other packs can use `@lift` where a language declares a member inside another declaration (Kotlin primary-constructor properties, Scala, PHP constructor promotion) without a change of the Rust code; they are not part of this ADR.
