# 0048 – Cognitive complexity and parameter counts from the language packs

Status: accepted

## Context

`check_conventions` and `codeskimr check` could not check two rules of blueprint 3 (R-02): the cognitive complexity of a function (at most 15) and its number of parameters (at most 5). The packs counted only decision points (cyclomatic, ADR 0015), and nesting does not show in them. Mapping `CHK-105` onto the cyclomatic complexity would be a different, looser rule, and the product aims at the analysis depth of specialised tools. Issue #93.

## Decision

1. **`metrics.scm` gets six more captures**, which a pack may use. A pack that has none of them keeps its counts and the rule is reported as not checked for its files.
   - `@cognitive` marks a structure that adds one to the cognitive complexity, plus one more for every level it is nested at (`if`, loops, `match` or `switch`, `catch`, conditional expression).
   - `@cognitive_flat` marks a part that adds one at any depth (`else`, `goto`, a jump to a label).
   - `@cognitive_sequence` marks the operator token of a boolean operation. The operators of one expression count one, and one more at every change of the operator in source order (`a || b && c || d` counts three). The engine groups them by the expression they belong to; a parenthesis or a call starts a new expression, so `a && (b && c)` counts two.
   - `@nesting` marks a node that nests what is inside it (the same structures, and lambdas, which add nothing themselves).
   - `@continuation` marks a structure that continues the chain around it (`else if`): it neither counts nor nests, so that the `else if` adds one through its `else` and stands as deep as the first `if`.
   - `@parameter` marks one parameter of a callable declaration, without a receiver (`self`, `this`).
2. **Counting.** The adapter assigns the captures to the innermost callable, as for decision points (ADR 0015). The nesting of a structure is the number of `@nesting` nodes between it and the declaration of its callable. A named local function counts for itself; a lambda counts for its callable and nests. Recursion, `?`-like operators and `match` arms add nothing; this follows the published definition of cognitive complexity except for recursion, which needs name resolution and is left out.
3. **`BodyCounts` carries `cognitive` and `parameters` as `Option`**: `None` means the pack does not count, so a missing count is never read as 0. They are not stored; only the convention check uses them.
4. **Two limits** in `[conventions]`: `max_cognitive_complexity` (15) and `max_parameters` (5), both positive. A function above them is reported as `CHK-105`, and as the new code `CHK-106` `too_many_parameters`. The report lists a rule under `not_checked` when a function of a language that does not count it was among the checked files.
5. **Rollout.** This decision comes with the Rust pack; the Python pack follows in the next package. TypeScript, C# and Java follow in packages of their own, the other packs later. `CHK-101` (literals with positions) needs another capture and its own decision.

## Consequences

- `CHK-105` and `CHK-106` work for Rust now and for every pack that adds the captures, without a change in the engine.
- The cognitive complexity is not part of the stored metrics or the UI.
- Blueprint 8.2 (pack vocabulary) and appendix A get the new captures and code.
