# Dependency map

The dependency map shows the **layers** of the active project as boxes and how they depend on each other. It answers: does the program follow its architecture, and where not?

It needs layers. Set them in the UI (the hint on the empty map leads there) or declare them in `.codeskimr.toml`, and scan again ([Projects and scan](projects-and-scan.md#layers)). Without layers the screen says "No layers to show".

## Reading the map

- One box per layer, one row per layer, outer layers on top (the order of your `[[layers]]`).
- A line between two boxes means "uses". Its label counts the references behind it.
- Lines against the declared layering are marked in the violation colour, and the legend shows how many there are. A banner above lists the violations ("Show" selects them). Lines that belong to a cycle are marked too.

The layout is calculated by CodeSkimr (layered drawing), so the same project always gives the same picture.

## Using it

- Mouse wheel zooms; drag the empty background with the left mouse button (or use the middle button or hold `Space`) to pan; `F` (or the button "Fit to view") fits the map to the window.
- Click a box to select it. Its lines are coloured by direction ("uses" and "used by"), all other lines turn weak. The side panel shows the layer, its number of references, what it **uses** and what it is **used by**, each with the count.
- The level "Layers" (default) or "Namespaces" in the toolbar reloads the map; the toolbar stays when a project has no layers, so the other levels can still be chosen. With only one layer the map says that a dependency map needs at least two. The namespace level is built like the type focus: one namespace in the middle (at first the one with the most references), the namespaces that use it above, the ones it uses below; a click selects a box and shows the details at the right, a double click makes it the middle. A name is split at its dots: the box shows the last part, the rest is the small breadcrumb. More than 24 neighbours per side are cut and noted; long rows wrap into several lines (`graph.row_boxes`). "Type focus" shows one type in the middle (its name in the box, the namespaces as a small breadcrumb below, the rows centred on the type), the types that use it above and the types it uses below; it is available when the explorer has a type selected. More than 12 types per side are cut and noted.
- The switch "all connections weak" shows or hides the lines that do not belong to the selection; "violations only" hides every line that does not break the layering.
- `Enter` with a layer selected opens the explorer for the project.

## Known limits

The map shows layer relations only. Which direction between two layers is **allowed** is not part of the project model yet; what counts as "against the layering" follows the layer graph that the scan recorded (see the open issue #45 about the layer rule). Use it as a hint, not as a verdict.
