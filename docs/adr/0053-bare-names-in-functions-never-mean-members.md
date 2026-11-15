# 0053 – A bare name in a function never means a member, in languages that need the receiver

Status: accepted

## Context

The resolver (blueprint 9.2, step 2) looks for a name without receiver in the enclosing scopes from the inside out, and a type counts as a scope: `count` inside a method finds the field `count` of the class. That is right for Java, C# and Kotlin (`count` is `this.count`). It is wrong for Python (also JavaScript, TypeScript, Rust and Go): a member there always needs its receiver (`self.count`), so a bare `count` in a function is a parameter, a local variable or a name of the module. The accuracy measurement of issue #81 (`certain` precision against Jedi, corpus pallets/click) showed the effect: 52 % of the `certain` uses were wrong, mostly parameters and locals that were taken for a field of the same name, and the declaration `name: str` of a field that counted as a write to it.

## Decision

1. The pack vocabulary gets one key in `language.toml`: `bare_names_reach_members` (default `true`). With `false`, the analysis reports a name without receiver inside a method, function, constructor or property with the new receiver `Free` (port `Receiver::Free`).
2. For a `Free` name the resolver drops every candidate that is declared directly in a type. Scopes outside the types, the imports and the last resort work as before, so such a name can still end `probable` on a module-level symbol.
3. The Python pack sets `bare_names_reach_members = false`. The packs of the other languages are not changed by this ADR; each can opt in after its own measurement.
4. A write whose name stands at the start of the declaration of the symbol it would resolve to (`name: str` in a class body) is the declaration, not a use. The resolver drops it for all languages.
5. The Python pack also hides parameters with a union or other complex type (`x: str | None = None`), which the earlier patterns missed.

## Consequences

- Python `certain` precision rises from 52 % to 75 % against Jedi, and to 98.8 % over the symbols for which Jedi answers at all (see `docs/accuracy.md`); the rest are fields that Jedi cannot see through `self.x`.
- The golden files of the Python pack change: name occurrences in functions carry the receiver `free`.
- Counts of symbols do not change, so `corpus/python/corpus.toml` stays as it is.
- No change of the data model or the storage.
