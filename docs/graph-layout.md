# Graph layout

The dependency map (blueprint 15, screen 3) needs positions for namespace and type boxes. They are computed in Rust (`codeskimr_domain::LayoutGraph`) and drawn on canvas by the UI (blueprint 15.5). The module is pure: nodes and edges in, positions out. It does not touch use cases, ports or storage.

## Interface

```text
LayoutGraph::new(nodes, edges)?   // checks that every edge names an existing node
graph.layout(&LayoutParams)       // -> Layout { placements, width, height }
```

- `LayoutNode` has a box size and an optional fixed `rank`. Fixed ranks are for layer bands (outer layers on top); without one the row follows from the edges.
- `NodePlacement` gives the top-left corner, the row (`rank`) and the position inside the row (`order`) of each node, in input order.
- `LayoutParams` (row gap, node gap, number of sweeps, number of placement passes) are tunable values. They move into `config/default.toml` when the API is wired (after WP-41); the module itself has no defaults.

## Algorithm (layered, Sugiyama style, no dependency)

1. **Rows.** Longest path from the sources. Cycles are broken by ignoring edges that point backwards in a depth-first order, so any graph gets a layout. Self loops are ignored.
2. **Long edges** become chains of invisible nodes, one per crossed row, so the ordering routes them around real nodes.
3. **Order.** Median sweeps down and up; the order with the fewest crossings (counted in O(n log n) with a Fenwick tree) is kept.
4. **Coordinates.** Rows are centred, then alternating passes pull nodes towards the middle of their neighbours while keeping the order and the gap.

The result is deterministic: same input, same output. Edge routing is not part of it; edges are drawn as straight lines or curves between boxes.

## Performance

Benchmark `graph_layout/nodes_200` is checked against `budgets.graph_layout_ms` (50 ms, blueprint 16.1) by the `bench` job; `graph_layout/nodes_1000` only guards against regressions. On the development machine 200 nodes take well below 1 ms.
