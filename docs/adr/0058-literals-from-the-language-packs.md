# 0058 – Literals in functions from the language packs

Status: accepted

## Context

`check_conventions` and `codeskimr check` could not check rule R-06 (no literals in production code, `CHK-101`). The Dylint lint `no_literals` covers Rust in CI, but C#, TypeScript, Python and Java had no check. The packs reported no literals with a position, and the exceptions of the rule (the integers 0 and 1, `true` and `false`, docstrings and annotations, constants) depend on the language (issue #93, ADR 0043).

## Decision

1. **`metrics.scm` gets two more captures**, which a pack may use. A pack without `@literal` keeps its counts and the rule is reported as not checked for its files.
   - `@literal` marks a literal node that the rule reports: a number, a string, a character.
   - `@literal_exempt` marks a node whose literal is an exception, although `@literal` matches it (a docstring, a string in a type annotation, `0` and `1`). The adapter takes it off before it counts. The pack decides the exceptions, because they differ by language; the engine knows none.
2. **Counting.** The adapter assigns a literal to the innermost callable, as for decision points (ADR 0015). A literal outside any callable (a field initialiser, a constant, a module-level value) counts for nobody: these are the places where the rule wants literals to live, so no `consts` file needs to be recognised. A node captured twice counts once. The arguments of a decorator or an attribute stand outside the callable they belong to and are not checked; this is a known gap, as is a literal in a default value of a language whose pack puts it outside the callable.
3. **`BodyCounts` carries `literals` as `Option`**: `None` means the pack does not report, never 0. It is not stored; only the convention check uses it.
4. **One finding per function**, `CHK-101` `literal_found`, with the number of literals in the function (`literals` in the report). It has no limit setting: the rule allows none. A function in a test folder is not checked (`conventions.test_folders`, as for the other rules).
5. **Rollout.** This decision comes with the Python pack. TypeScript, TSX, JavaScript and C# follow in a package of their own: the TypeScript file lists the value positions of a literal (arguments, operands, returned and assigned values, array elements and so on), which keeps out the literals of types, keys, module names and directives and is valid for the JavaScript grammar, which shares the file. Known gaps of the value-position list (a missing position is a silent false negative, so the list is extended when one is found): the default value of a TypeScript parameter, casts and JSX in TypeScript (TSX and JavaScript see them), and the parts of an expression in a `const` initialiser, which is exempt only when it is a literal itself. Java follows in a package of its own: numbers, strings and text blocks, characters; exceptions are the integers 0 and 1 (also `0L`, `1L`), the value of a local `final` variable when it is a literal itself, and the literals in the values of annotations (also the elements of an array value and the operands of one binary expression such as `60 * 1000`). Known gap: in a deeper expression of an annotation value (`1 + 2 * 3`, a nested call) the inner literals still count. `true`, `false` and `null` are no literals. Rust stays with the Dylint lint: a Rust file is reported as not checked for `CHK-101`, because `#[cfg(test)]` modules inside `src` are not recognised and would give false findings.

## Consequences

- `CHK-101` works for Python now and for every pack that adds the captures, without a change in the engine.
- The literals in a function are counted, not listed with their own line; the finding stands at the function. A later change may add positions if the count proves too coarse.
- The pack vocabulary is described in `docs/language-packs.md`; `BLUEPRINT.md` names the captures in the file tree of a pack.
