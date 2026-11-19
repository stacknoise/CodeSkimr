# 0074 – Coupling metrics for types

Status: accepted

## Context

ADR 0037 defines Ce, Ca and instability for namespaces and modules only. The explorer shows the three tiles for every symbol, so a type showed three empty tiles. The product owner decided on 2026-11-19 that types get the figures too.

## Decision

A **type unit** is a symbol for which `SymbolKind::is_type` holds. The definition of ADR 0037 applies with the unit changed:

- A symbol belongs to the **nearest enclosing type**, found by its qualified name; a type belongs to itself. A method, property or field counts for its type. Symbols outside every type (a namespace, a free function) belong to no type unit.
- Nested types are separate units: nothing is rolled up to the outer type.
- `ce` is the number of other types that a type's symbols refer to, `ca` the number of other types whose symbols refer to it, each counted once. References inside one type and references of kind `test` do not count. Types with the same qualified name (partial classes) are one unit and every declaring symbol carries the values.
- `instability` is `ce / (ca + ce)` and is missing without any dependency.
- Namespaces and modules keep their definition; both levels are computed in the same pass over the references and stored in `coupling_rows` per symbol (ADR 0037, storage unchanged).

## Consequences

- No migration. The scan computes the references twice (once per level); the rescan benchmark has to stay in budget.
- The tiles of a type show values instead of dashes. A method has none of its own.
- Counting by symbol (for example weighting calls) stays open and would need its own ADR.
