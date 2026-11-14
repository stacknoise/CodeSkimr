# Graph layout

The dependency map (blueprint 15, screen 3) needs positions for namespace and type boxes. They are computed in Rust (`codeskimr_domain::LayoutGraph`) and drawn on canvas by the UI (blueprint 15.5). The module is pure: nodes and edges in, positions out. It does not touch use cases, ports or storage.

## Interface

```text
LayoutGraph::new(nodes, edges)?   // checks that every edge names an existing node
graph.layout(&LayoutParams)       // -> Layout { placements, width, height }
```

- `LayoutNode` has a box size and an optional fixed `rank`. Fixed ranks are for layer bands (outer layers on top); without one the row follows from the edges. Used ranks are renumbered without gaps (order kept), so a huge rank costs no memory and empty bands take no space.
- `NodePlacement` gives the top-left corner, the row (`rank`) and the position inside the row (`order`) of each node, in input order.
- `LayoutParams` (row gap, node gap, number of sweeps, number of placement passes) are tunable values. They are the `[graph]` settings of `config/default.toml` (`rank_gap`, `node_gap`, `sweeps`, `placement_passes`, plus the box size `node_width` and `node_height`); the module itself has no defaults.

## Command `graph.layers`

`codeskimr_domain::layer_map` turns the declared layers and the aggregated layer edges (`layer_graph` read model) into a `LayerMap`: one box per layer, each declared layer fixed to the row of its position, outer layers first (ADR 0031). The command `graph.layers` (`crates/app/src/ui_commands/graph.rs`, REST `POST /api/v1/commands/graph.layers`, permission `read`) reads the newest snapshot and answers with boxes, links and the picture size, see `docs/server.md`.

`graph.layers` takes an optional `level`: `layers` (default) or `namespaces`. The answer carries `bands`, one stripe per layer row (`name`, `rank`, `y`, `height`) that the UI draws behind the boxes. On the level `namespaces` (`codeskimr_domain::namespace_graph`) there is one box per namespace that holds types or has edges, with `types` (number of types in it) and `layer` (the owning layer, if any); an edge counts the references from the types and members of one namespace to those of another. A name is assigned to the longest known namespace that is a prefix of it (separator `.` or `::`). Rows come from the layered layout; the C# prefix of qualified names (issue #137) and the empty layer edges for C# (issue #145) limit how well namespaces match layers.

## Algorithm (layered, Sugiyama style, no dependency)

1. **Rows.** Longest path from the sources. Cycles are broken by ignoring edges that point backwards in a depth-first order, so any graph gets a layout. Self loops are ignored.
2. **Long edges** become chains of invisible nodes, one per crossed row, so the ordering routes them around real nodes.
3. **Order.** Median sweeps down and up; the order with the fewest crossings (counted in O(n log n) with a Fenwick tree) is kept.
4. **Coordinates.** Rows are centred, then alternating passes pull nodes towards the middle of their neighbours while keeping the order and the gap.

The result is deterministic: same input, same output. Edge routing is not part of it; edges are drawn as straight lines or curves between boxes.

## Performance

Benchmark `graph_layout/nodes_200` is checked against `budgets.graph_layout_ms` (50 ms, blueprint 16.1) by the `bench` job; `graph_layout/nodes_1000` only guards against regressions. `graph_layers/layers_200` measures the step the command adds (layers and edges to boxes, 200 layers in 200 rows) against the same budget; it takes about 0.4 ms. On the development machine 200 nodes take well below 1 ms.
