# 0034 – Side goal after 1.0: top-league analysis depth in every supported language

Status: proposed

## Context

The product owner wants CodeSkimr, in later versions, to play in the top league in every language it supports: for each language, the analysis depth of the best specialised tools of that language (13 November 2026). C# is the first, not the only, target. The yardstick for C# is NDepend: dependencies between types and between members, metrics at every level, rules over the code model, overloads, generics and similar. 1.0 does not change (blueprint 1 and 19): it ships level 1 references, the metrics of ADR 0015 and `check_conventions`. Level 2 references from SCIP indexes (ADR 0033) is the first step on this road and is already decided.

## Options considered

- **A. No stated goal.** Depth work then happens as isolated fixes and the models grow without a direction.
- **B. A fixed list of features copied from one tool.** Gives a checklist, but ties the product to a commercial tool and ignores that each language has its own concerns.
- **C. A depth ladder with the same rungs for every language, one language at a time, C# first (chosen).** The rungs are language independent; each language climbs them in its own packages.

## Decision

**The ambition is the top league in all supported languages.** A language counts as top league when it reaches the rungs below and passes its acceptance against the yardstick tool that the product owner names for it. Languages that are only in the basic tier are promoted to the full tier first (ADR 0011), then climb the ladder.

### The depth ladder

| Rung | What it adds | Where it lives |
|---|---|---|
| **D1** | Level 1 references, metrics of ADR 0015, conventions | 1.0, done |
| **D2** | Compiler-precise references (SCIP, Roslyn) | ADR 0033 |
| **D3** | Semantic facts: overloads and signatures, generic parameters and arguments, base and implemented types, extension methods, partial types, attributes or annotations, member-level dependency kinds (reads, writes, calls, creates, implements, overrides) | domain model, language packs, import |
| **D4** | A wider metric set on the facts: coupling per type and per member, cohesion, inheritance depth, abstractness, instability and distance, nesting, parameter count | domain metrics, read models |
| **D5** | Rules over the code model: the built-in conventions grow into a rule set, projects can add their own rules, results go to CLI, MCP, UI and CI | application layer, `check_conventions` |
| **D6** | Comparison of two snapshots and trends, impact of a pull request | storage, application, UI |

A rung is reached for a language when its acceptance report (below) is met. Rungs are delivered per language; a language can be at D2 while another is at D4.

### Work packages after 1.0 (rough split, to be refined when the milestone starts)

Language independent (the model and engine, built once, with C# as the first user):

- **WP-100 Depth model.** ADR for the new symbol and reference kinds and facts (D3), domain types, storage migration, language pack format extension. Architect and core developer.
- **WP-101 Metric suite on the model** (D4). Type and member coupling, cohesion, inheritance depth, abstractness, distance, nesting.
- **WP-102 Rule engine** (D5). Rule definitions over the model, project rules in the project file, results in CLI, MCP and CI; the 1.0 conventions become the first rules.
- **WP-103 Snapshot comparison and trends** (D6), with the pull request impact analysis.
- **WP-104 UI for depth.** Dependency matrix, type and member graph, rules view, trend view. Needs mocks first (`docs/mocks/`).

C# first (the reference for the ladder):

- **WP-110 C# semantic facts** (D2 and D3): overloads, generics, extension methods, partial types, attributes, records, interface implementations; read through the Roslyn based indexer path of ADR 0033.
- **WP-111 C# rule set and metrics** (D4 and D5): a default rule set at the level of the common C# analysers, configured through the project file.
- **WP-112 C# dependency views** (D3 in the UI).
- **WP-113 C# acceptance against the specialised tool.** The product owner runs NDepend locally on a private project and compares figures; NDepend, its output and the private project never enter this public repository (rule 11 of `CLAUDE.md`, ADR 0017). The public C# corpus gets expected values that are checked by hand.

The other languages follow in this order, one depth package per language each (facts, metrics and rules for that language, acceptance against the tool the product owner names): **WP-120 TypeScript/JavaScript, WP-121 Java, WP-122 Kotlin, WP-123 Python, WP-124 Rust, WP-125 Go** (Go also gets its full tier, ADR 0011). The yardstick tool for each language is confirmed by the product owner when its package starts.

### Guard rails

- **1.0 is not delayed.** No package of this ADR is in the 1.0 plan and none blocks M7.
- **Keep the seams open in 1.0.** New facts and kinds are additive: a symbol or reference kind needs its ADR (blueprint rule), storage changes are migrations, the language pack format stays backwards compatible, and read models stay replaceable. A 1.0 package that would close one of these seams is reviewed against this ADR.
- **No dependence on a commercial tool.** Comparisons with such tools happen locally and by the product owner; results enter the repository only as expected values that were checked by hand.
- **Honesty rule stays.** Anything not checked by a compiler-grade source stays marked as `probable` (blueprint 9.4).

## Consequences

- Blueprint 1 (out of scope for 1.0), 9.5 (new, short) and 19 (row "later") refer to this ADR; the planning document gets a section "After 1.0: depth".
- The package numbers above are a plan, not a commitment; each package gets its own issue and, where it changes the model, its own ADR.
- NDepend is named as a yardstick only; this ADR does not copy its features one to one.
