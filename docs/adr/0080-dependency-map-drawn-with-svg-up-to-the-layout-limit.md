# 0080 – The dependency map is drawn with SVG up to the layout limit, canvas follows after 1.0

Status: accepted (Anton, 2026-11-21; issue #198)

## Context

Blueprint 15.5 says that the graph layout is computed in Rust and drawn on canvas. The dependency map (screen 3) is drawn with HTML and SVG instead: every box and every edge is an element of the page. That gives real buttons for the boxes (keyboard focus, screen reader names, tooltips, click targets) and lets the browser tests find boxes by their role. The layout itself is computed in Rust as the blueprint wants (`codeskimr_domain::LayoutGraph`).

The layout budget of blueprint 16.1 is set for up to 200 nodes (< 50 ms; the benchmark `graph_layout/nodes_200` needs about 1 ms). The map as built keeps drawing, zooming and moving smooth at that size. A page with thousands of elements gets slow when it is zoomed or moved; only then does canvas pay off.

Drawing on canvas looks the same when done well, but costs the elements: boxes are no longer buttons, so keyboard control, screen reader names, tooltips and hit testing have to be rebuilt, and the browser tests that find boxes as elements have to be rewritten.

## Decision

1. Up to the layout limit of 200 nodes the map is drawn with HTML and SVG. Blueprint 15.5 says so.
2. Drawing on canvas is a separate work package after 1.0, started when a real project goes beyond that limit or the measurement of the map (zoom, move) misses a frame budget. It has to keep what the SVG map offers: keyboard control, accessible names, tooltips and the tests of the map.
3. Until then the map shows at most the nodes the layout budget covers; a project with more namespaces uses the depth levels and the type focus to stay below it (already built, #215 and #233).

## Consequences

- Blueprint 15.5 changes: "Graph layout is computed in Rust and drawn with SVG up to 200 nodes; beyond that on canvas (ADR 0080)."
- `docs/ui.md` and `docs/graph-layout.md` no longer list the SVG drawing as an unexplained deviation.
- Issue #198 is closed; the canvas work becomes its own issue after 1.0.
