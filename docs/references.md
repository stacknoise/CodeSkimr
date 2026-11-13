# Reference resolution, level 1 (WP-30)

Blueprint 9.2. The resolver turns the names that the analysis found (`PendingReference`) into `Reference` rows with a confidence. It lives in `crates/application/src/resolve/` (`ScopedResolver`, wired in `crates/app/src/wiring.rs`) and sees only the global symbol table and the pending names, never a syntax tree (ADR 0013).

## What the parsing adapter adds

`references.scm` also reports facts that only the tree knows (`crates/adapters/parsing/src/references.rs`, `scopes.rs`):

- **Locals and parameters** (step 1). A bare name that a local variable or parameter of the enclosing scope hides, declared before the use, is not a reference to a symbol and is dropped.
- **Receiver** (step 3). For `receiver.name` the pending name carries a `Receiver`: `Typed` (a variable with a declared type in the file, or `new Foo()`), `Named` (a name as written, often a type for static access), `Own` (`this`) or `Base`. Fields and properties declared in the same file give the type of a receiver, but a use of them stays a reference.

## Order of the resolver

Three passes, because later ones need what earlier ones found: imports, then inheritance (base lists), then everything else.

1. A receiver gives the types in which the member is searched, with their base types (nearest type first). A receiver of a type outside the project finds nothing: its members are never mistaken for project symbols of the same name. This holds for a declared type, and for a receiver name that no project symbol bears (`Console.WriteLine`). A receiver name that is a variable or member of unknown type tells nothing, and the member falls back to the last resort. A receiver type that itself was found only by the last resort still gives a `certain` member if the member is found in it.
2. Without usable receiver: the enclosing symbols from the inside out (members of the enclosing type and its bases, enclosing namespaces including their parent namespaces), then the namespaces the file imports.
3. Last resort: the name alone.

Which kinds a name can point to depends on the kind of occurrence (`resolve/kinds.rs`); a name in a value position that is a type is reported as `type_use`, a base type that is an interface as `implement`.

## Confidence

`certain`: one candidate found in steps 1 to 4. `probable`: several candidates (all are stored), the last resort, a member of an interface or trait (the implementation is unknown), or a dispatch: if the target is a method, property or event that subtypes replace or implement, the members of those subtypes are added as candidates (a call through an interface or base class can end up in any of them). `RES-301` is not reported yet. A reference that is resolved again in an incremental scan keeps its earlier confidence, and the kept base lists are resolved again so that the hierarchy is complete (`docs/incremental-scan.md`).

## Known limits (follow-up work)

- Overloads are not told apart (no argument counts), so a call of an overloaded method is `probable`.
- Extension methods, `using static`, generic type parameters and members reached through a chain (`a.b.c()`) fall back to the last resort or to nothing.
- The kind `test` is not produced yet.
- Resolution is sequential; ids are handed out in a fixed order.
