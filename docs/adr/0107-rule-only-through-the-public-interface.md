# 0107 – Rule: only through the public interface

Status: accepted (decided as part of A1, see ADR 0104).

## Context

A module has an inside that others should not reach into. Without the kind of every name level (namespace, type, member) the scan cannot tell "the public part of a module" from "its inside" by itself, so the project says where the inside is.

## Decision

1. `[[rules.public_api]]` has `module` (a qualified-name prefix) and `internal`, a non-empty list of prefixes that each lie in `module` and are not `module` itself (`CFG-101` otherwise). Everything outside `internal` but in `module` is the public interface.
2. A reference violates the rule when the symbol it refers to lies in one of the `internal` prefixes and the referring symbol does not lie in `module`. References from inside the module, also from the inside to the inside, are fine.
3. One violation per referring symbol and target symbol, with every place (ADR 0104, point 3).
4. The concept's wording "only the root module of another" is the special case `internal = ["<module>.<each sub-namespace>"]`; listing the inside explicitly keeps the rule exact for languages whose names do not tell namespaces from types.

## Consequences

- The screen shows `Shop.Data.Internal.*` only within `Shop.Data`.
- No pattern language is needed; the prefixes are the same as for layers.
