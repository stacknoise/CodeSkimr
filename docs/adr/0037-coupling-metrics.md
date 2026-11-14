# 0037 – Coupling metrics: definition and per-snapshot storage

Status: proposed

## Context

ADR 0015 puts Ce, Ca and instability on namespaces and layers but does not say what is counted. ADR 0032 stores metrics once per symbol and lets a new value replace the stored one for all snapshots; it names the condition that a metric depends on the content of its own file only (issue #150). Coupling depends on references across files, so it breaks that condition.

## Options

1. Change ADR 0032 so that metrics are per snapshot. The scan would copy or rewrite all metrics again, which is the cost ADR 0032 removed (issue #108).
2. Keep ADR 0032 for the metrics of one file and store coupling in its own table with the snapshot id. Coupling is small (three rows per namespace or module), so writing all of it for every snapshot is cheap.

## Decision

Option 2.

### Definition

- A **unit** is a symbol of kind `namespace` or `module`. Units with the same qualified name are one unit (a C# namespace that several files declare); every declaring symbol carries the values.
- A symbol belongs to the **nearest enclosing unit**, found by its qualified name like in the namespace graph; a unit belongs to itself. Nested units are separate units: nothing is rolled up from a nested unit to its parent.
- `ce` is the number of other units that a unit's symbols refer to, `ca` the number of other units whose symbols refer to it. A unit counts once however many references there are. References inside one unit, references from or to symbols outside every unit, and references of kind `test` do not count. All other kinds count, imports included, whatever their confidence.
- `instability` is `ce / (ca + ce)`. A unit without any dependency in either direction has `ce` and `ca` of 0 and no `instability`, because 0 divided by 0 is no measurement.
- Layers (project file, ADR 0031) are not units here. Their dependencies are the `layer_graph` read model; a per-layer figure can be derived from it when the UI needs one.

### Storage

- Migration 0009 adds `coupling_rows (snapshot_id, symbol_id, kind, value)`. `MetricKind::is_per_snapshot` names the kinds that go there (`ce`, `ca`, `instability`); the store routes by kind.
- The view `metrics` is the union of the shared metrics of a snapshot's symbols and its coupling rows, so readers do not change.
- `StandardMetrics` computes coupling from all symbols and references of the new snapshot, also those of unchanged files, and every scan writes it completely. Nothing is carried over: a snapshot without coupling rows has none. The prune removes the rows of the pruned snapshot.

## Consequences

- ADR 0032's condition holds again: everything in `metric_rows` depends on one file.
- The level 2 import (ADR 0033) replaces references after the scan and does not recompute coupling; until the next scan the figures come from the level 1 references. Follow-up issue.
- Blueprint 6.1 lists the kinds already; its granularity text can point to this ADR when the blueprint is next edited.
