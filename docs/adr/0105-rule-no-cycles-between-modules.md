# 0105 – Rule: no cycles between modules

Status: accepted (decided as part of A1, see ADR 0104).

## Context

The dependency map marks edges that lie on a cycle (`in_cycle`, ADR 0031), but a project cannot say that cycles are not allowed. A cycle between modules means they can only be changed, tested and released together.

## Decision

1. `rules.cycles` is `"off"` (default) or `"deny"`. Anything else is `CFG-101`.
2. **Module** is a node of the module graph (the namespace graph of a snapshot, ADR 0108). The detection is the one of the module graph, `find_cycles`; the rule does not detect cycles itself.
3. **A violation is a cycle**, a strongly connected group of at least two modules. Its identity in the baseline is the sorted list of its modules, so growing or shrinking a cycle by one module is a new violation.
4. The screen shows the cycle as a chain (`A → B → A`) and, as the module graph does, the weakest edge: the edge with the fewest references, with the places that would have to change ("resolvable over 1 edge with 2 places"). The advice is a hint, not a promise; the compiler decides what can be moved.
5. Edges inside one module and references of kind `test` do not count (same filter as the layer graph).

## Consequences

- The rule is built after the cycle detection of the module graph is merged (ADR 0104, step 4). It is built: the places of a violation are the references of the weakest edge, found by the nearest enclosing namespace of the referring and the referred symbol, and a ring is held in `.codeskimr-baseline.toml` as `from = "A,B,C"` with an empty `to`.
- Projects whose layers form a cycle on purpose keep `"off"` or hold the state.
