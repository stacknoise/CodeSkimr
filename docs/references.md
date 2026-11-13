# Reference resolution, level 1 (WP-30)

Blueprint 9.2. The resolver turns the names that the analysis found (`PendingReference`) into `Reference` rows with a confidence. It lives in `crates/application/src/resolve/` (`ScopedResolver`, wired in `crates/app/src/wiring.rs`) and sees only the global symbol table and the pending names, never a syntax tree (ADR 0013).

## What the parsing adapter adds

`references.scm` also reports facts that only the tree knows (`crates/adapters/parsing/src/references.rs`, `scopes.rs`):

- **Locals and parameters** (step 1). A bare name that a local variable or parameter of the enclosing scope hides, declared before the use, is not a reference to a symbol and is dropped.
- **Receiver** (step 3). For `receiver.name` the pending name carries a `Receiver`: `Typed` (a variable with a declared type in the file, or an initializer that names the type: `new Foo()`, `(Foo)x`, `x as Foo`), `Named` (a name as written, often a type for static access), `Own` (`this`), `Base`, or `Unknown` (there is a receiver but its type is not written: `var x = make()`, a call result, a chain). Fields and properties declared in the same file give the type of a receiver, but a use of them stays a reference.

## Order of the resolver

Three passes, because later ones need what earlier ones found: imports, then inheritance (base lists), then everything else.

1. A receiver gives the types in which the member is searched, with their base types (nearest type first). A receiver of a type outside the project finds nothing: its members are never mistaken for project symbols of the same name. This holds for a declared type, and for a receiver name that no project symbol bears (`Console.WriteLine`). A receiver of unknown type (`Unknown`, or a name that is a variable or member of unknown type) tells nothing, and the member falls back to the last resort: the name alone, never more than `probable`, and the enclosing type is not searched for it. If that name belongs to members of types that have nothing to do with each other (neither derives from the other and they share no base type), none of them is reported. This holds for every language: a wrong usage is worse than a missing one. A receiver type that itself was found only by the last resort still gives a `certain` member if the member is found in it.
2. Without usable receiver: the enclosing symbols from the inside out (members of the enclosing type and its bases, enclosing namespaces including their parent namespaces), then the namespaces the file imports.
3. Last resort: the name alone.

Which kinds a name can point to depends on the kind of occurrence (`resolve/kinds.rs`); a name in a value position that is a type is reported as `type_use`, a base type that is an interface as `implement`.

## Confidence

`certain`: one candidate found in steps 1 to 4. `probable`: several candidates (all are stored), the last resort, a member of an interface or trait (the implementation is unknown), or a dispatch: if the target is a method, property or event that subtypes replace or implement, the members of those subtypes are added as candidates (a call through an interface or base class can end up in any of them). `RES-301` is not reported yet. A reference that is resolved again in an incremental scan keeps its earlier confidence, and the kept base lists are resolved again so that the hierarchy is complete (`docs/incremental-scan.md`).

## Known limits (follow-up work)

- The type of a method result (`var x = svc.Make()`) is not known: the pack does not report return types. A call on `x` is `probable` if the name exists in one type only, and not reported if several unrelated types have it.
- Overloads are not told apart (no argument counts), so a call of an overloaded method is `probable`.
- Extension methods, `using static`, generic type parameters and members reached through a chain (`a.b.c()`) fall back to the last resort or to nothing.
- The kind `test` is not produced yet.
- Resolution is sequential; ids are handed out in a fixed order.
