# 0017 – Counting rule for the private C# reference

Status: proposed

## Context

Blueprint 17.2 named fixed counts for the private C# reference (433 types, 825 methods, 1,667 properties/fields in version 0.3.1). Nobody could say how they were counted: they appear in no file, commit or pull request of the application, and no counting variant reproduces them (measured at the release commit 0ff5367 with the C# pack and, independently, with a regex generator: 416 to 423 types, 671 to 801 methods). A reference nobody can reproduce cannot check anything.

## Decision

The counts of the private reference are the baseline measured with one fixed rule, not the old numbers. The product owner keeps the measured values locally; they never enter this repository (ADR 0011).

Rule: all `*.cs` files below `src`, without `bin`, `obj` and generated files (`*.g.cs`). The groups of symbol kinds are:

| Group | Symbol kinds |
|---|---|
| types | class, struct, interface, record, enum, type_alias (delegate), nested types included |
| methods | method (incl. operators), constructor, function (local function) |
| properties | property (incl. indexers and positional record parameters) |
| fields | field, constant |
| events | event |
| enum members | enum_member |

How to measure, locally: `CODESKIMR_CORPUS_DIR=<path to src> CODESKIMR_BLESS=1 cargo test -p codeskimr_parsing --test corpus -- --nocapture` prints the counts per kind and a `SUMMARY` line with these groups. For the public reference the same groups follow from `corpus/csharp/corpus.toml`.

Blueprint 17.2 changes accordingly: the fixed numbers are replaced by the baseline of this rule.

## Consequences

- Version 0.3.1 of the application (commit 0ff5367, `src`) is the first baseline: 433 types would only match by chance at another version, so the baseline is measured at the version the product owner names.
- Later scans must reproduce the baseline or list each difference as an improvement; the rule changes only through a new ADR.
- Positional record parameters are properties, which is why the property group is larger than a plain regex count.
