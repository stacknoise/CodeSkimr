# 0058 – Literals in functions from the language packs

Status: accepted

## Context

`check_conventions` and `codeskimr check` could not check rule R-06 (no literals in production code, `CHK-101`). The Dylint lint `no_literals` covers Rust in CI, but C#, TypeScript, Python and Java had no check. The packs reported no literals with a position, and the exceptions of the rule (the integers 0 and 1, `true` and `false`, docstrings and annotations, constants) depend on the language (issue #93, ADR 0043).

## Decision

1. **`metrics.scm` gets two more captures**, which a pack may use. A pack without `@literal` keeps its counts and the rule is reported as not checked for its files.
   - `@literal` marks a literal node that the rule reports: a number, a string, a character.
   - `@literal_exempt` marks a node whose literal is an exception, although `@literal` matches it (a docstring, a string in a type annotation, `0` and `1`). The adapter takes it off before it counts. The pack decides the exceptions, because they differ by language; the engine knows none.
2. **Counting.** The adapter assigns a literal to the innermost callable, as for decision points (ADR 0015). A literal outside any callable (a field initialiser, a constant, a module-level value) counts for nobody: these are the places where the rule wants literals to live, so no `consts` file needs to be recognised. A node captured twice counts once.
3. **`BodyCounts` carries `literals` as `Option`**: `None` means the pack does not report, never 0. It is not stored; only the convention check uses it.
4. **One finding per function**, `CHK-101` `literal_found`, with the number of literals in the function (`literals` in the report). It has no limit setting: the rule allows none. A function in a test folder is not checked (`conventions.test_folders`, as for the other rules).
5. **Rollout.** This decision comes with the Python pack. TypeScript, JavaScript, C# and Java follow in packages of their own. Rust stays with the Dylint lint: a Rust file is reported as not checked for `CHK-101`, because `#[cfg(test)]` modules inside `src` are not recognised and would give false findings.

## Consequences

- `CHK-101` works for Python now and for every pack that adds the captures, without a change in the engine.
- The literals in a function are counted, not listed with their own line; the finding stands at the function. A later change may add positions if the count proves too coarse.
- Blueprint 8.2 (pack vocabulary) names the new captures.
