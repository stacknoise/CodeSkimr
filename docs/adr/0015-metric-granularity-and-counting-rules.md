# 0015 – Metric granularity, counting rules and Halstead counts

Status: accepted

## Context

WP-20 delivered pure metric functions in `codeskimr_domain::metrics` (lines of code, cyclomatic complexity, maintainability index). They take counts, not syntax trees. Blueprint 6.1 attaches a `Metric` to a symbol but does not say to which symbols, how values of members roll up to types and files, which syntax nodes count as decision points, or where the Halstead counts for the maintainability index (MI) come from (issue #15). Without a decision, every language pack would count differently and the results could not be compared across languages. Because of ADR 0013 the syntax tree never leaves the adapter, so the counts must be produced inside Analyze.

## Options

1. Metrics only per file. Simple, but the Explorer metric strip and the thresholds in `config/default.toml` are about functions and types.
2. Metrics per callable (function, method, constructor), rolled up to types, modules and files with fixed rules.
3. Like 2, but MI without Halstead (only CC and LOC). Fewer counts for language packs, but a different formula from the Visual Studio form that WP-20 implements, and results that no other tool reproduces.

## Decision

Option 2, with Halstead counts delivered by the language packs.

### Granularity

| Level | Symbols | `loc` | `cc` | `cc_max` | `mi` |
|---|---|---|---|---|---|
| Callable | `function`, `method`, `constructor` | code lines in its span | 1 + decision points of its own body | not stored | from its own Halstead counts, CC and LOC |
| Type | `class`, `interface`, `trait`, `record`, `struct`, `enum`, `union` | code lines in its span | sum of `cc` of its callables, including those of nested types | maximum `cc` of those callables | LOC-weighted mean of the `mi` of its callables |
| Module | `module`, `namespace` (one `module` per file) | code lines in its span (a file's module: all code lines of the file) | sum over its callables | maximum over its callables | LOC-weighted mean over its callables |

- Other symbol kinds (`property`, `field`, `constant`, `event`, `enum_member`, `parameter`, `type_alias`) carry no metrics. A property with an accessor body is a callable only if the language pack maps it to `method`.
- A type or module without callables has `cc` 0, no `cc_max` and no `mi`. The weighted mean is also undefined when all callables have 0 code lines; then there is no `mi`.
- A module (namespace) includes nested namespaces and types. Callables without a body (abstract, interface, extern, declaration only) are not measured: they get no metrics and take no part in sums, maxima or means.
- Code outside any callable (field initializers, static initializers, top-level code of a script or module) counts for `loc` of the enclosing type or module but contributes no `cc` and no `mi`.
- `loc` of a callable or type excludes the lines of its named local functions and local types, which have their own `loc`; the `loc` of a lambda stays inside its callable. Rollups sum `cc` over all callables below a symbol, so nothing is counted twice there.
- `File.loc` equals the `loc` of the file's module symbol.
- The thresholds for CC in `config/default.toml` apply to callables and to `cc_max`, never to the sum, because a sum grows with size and not with difficulty. The sum is shown as information only.
- A new metric kind `cc_max` is added to the `Metric` entity. Coupling metrics (Ce, Ca, instability) are unchanged; they belong to namespaces and modules and need no counts from the language pack. ADR 0037 defines them and stores them per snapshot.

### Nesting

- A lambda or anonymous function is not a symbol; its lines, decision points and tokens count for the callable that contains it.
- A named local function (and a local or anonymous class) is its own symbol with the enclosing callable as parent; its lines, decision points and tokens count for it and not for the enclosing callable. Type rollups still reach it.
- A callable's span runs from its first modifier or attribute to its closing token, so the signature and braces count as code.

### Line classification

Each physical line is `Blank`, `Comment` or `Code`:

- `Blank`: only whitespace.
- `Comment`: the only non-whitespace content is inside comment nodes (line comments, block comments, doc comments).
- `Code`: everything else, including a line with code and a trailing comment, and lines inside multi-line string literals.

The pack marks comment nodes with the capture `@comment`; the adapter derives the line kinds, so no language needs its own line logic.

### Cyclomatic complexity

`cc` = 1 + number of decision points. Each of the following counts once, in every language that has the construct:

- `if` and `else if` / `elif` (not a plain `else`)
- conditional expression (`?:`, `x if c else y`)
- every loop: `for`, `foreach`, `while`, `do … while`, comprehension clauses with a condition or loop
- every `case`, `when` or match arm (not `default` or the wildcard arm)
- every `catch` / `except` clause (not `try`, not `finally`)
- every short-circuit operator: `&&`, `||`, `and`, `or`, null-coalescing `??` and its compound forms (`??=`, `||=`, `&&=`), Kotlin Elvis `?:`
- `if let` and `while let` count like `if` and `while`; a `switch` expression counts per arm like a `case`; a `catch … when (…)` counts once, as the catch

Not counted: `else`, `default`, `try`, `finally`, `goto`, `return`, `assert`, `loop` (Rust, no condition), `for … else`, the Rust `?` operator, optional chaining (`?.`), `is` patterns outside a match arm, and a second label on the same case arm. A short-circuit operator inside a match guard counts as usual. The list is exhaustive: a construct that is not listed does not count, and adding one needs an ADR. A pack maps these rules to node types in `metrics.scm` with the capture `@decision` and lists the mapping in its `language.toml` documentation; the golden tests contain one case per rule.

### Halstead counts

The MI needs `HalsteadCounts`. The packs deliver them with two captures in `metrics.scm`:

- `@operator`: operator tokens (arithmetic, comparison, logical, bitwise, assignment, member access, call, indexing) and control-flow and creation keywords (`if`, `for`, `return`, `new`, `throw` …). Pure delimiters (`;`, `,`, braces, parentheses) do not count, but a call or an index expression counts once as an operator.
- `@operand`: identifiers and literals. Distinctness is by source text for operands and for operators with a token; a call counts as the operator `()` and an index expression as `[]`. Type names, `this`/`self` and annotations are not operands; a type name used as a value (`new Foo()`, `Foo.bar`) counts as an identifier operand.

Counts are taken per callable body, with the nesting rule above. Counts are not added up for types or modules, because distinct counts cannot be summed; types and modules use the LOC-weighted mean of their callables' MI.

### Tiers

Basic-tier packs deliver `loc`, `cc` and `cc_max` (they need `@comment` and `@decision` only). `mi` is a full-tier metric because it needs the `@operator` and `@operand` captures.

### Cost

The four captures run in one query pass per file inside Analyze, so no extra parse is needed. The scan and incremental budgets of blueprint 16.1 stay as they are, and the `bench` check covers the pass.

### Delivery

The Analyze stage in the adapter returns, per callable and per file, the counts: line kinds, number of decision points, Halstead counts. The Compute metrics stage applies the pure functions of `codeskimr_domain::metrics` and the rollups above; it does not need the tree.

## Consequences

- Blueprint 6.1 (`Metric` kinds and a granularity table), 8.1 (`metrics.scm` instead of `complexity.scm`), 8.2 (tier text) and the glossary change. The agent description `.claude/agents/language-developer.md` names the new file.
- The C# pack and every later pack write `metrics.scm` with `@comment`, `@decision`, `@operator` and `@operand`, and golden tests for each counting rule.
- Values are comparable across languages in kind but not exactly, because language constructs differ.
- The weighted mean ignores callables without code lines (their weight is 0).
