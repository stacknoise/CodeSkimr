# 0106 – Rule: module A must not use module B

Status: accepted (decided as part of A1, see ADR 0104).

## Context

Layers (ADR 0054) say which layer may use which. They cannot say that two modules inside one layer or across layers must stay apart, for example reports and checkout.

## Decision

1. `[[rules.forbid]]` has `from` and `to` (qualified-name prefixes, valid like the namespaces of a layer) and an optional `reason` text that the screen shows. `from` and `to` must differ and neither may contain the other, otherwise the rule would forbid a module to use itself (`CFG-101`).
2. A reference violates the rule when its referring symbol lies in `from` and the symbol it refers to lies in `to`. "Lies in" is the one of layers: equal to the prefix or below it up to a separator, `.` or `::`.
3. One violation per referring symbol and target symbol, with every place of the reference (ADR 0104, point 3).
4. The rule is one direction. "A and B must not use each other" is two entries.
5. A reference whose target has several candidates is not certain and is not counted (the filter of the layer graph).

## Consequences

- The title on the screen is generated: "Verbot: Shop.Reports darf Shop.Checkout nicht nutzen" (a locale text with the two names).
