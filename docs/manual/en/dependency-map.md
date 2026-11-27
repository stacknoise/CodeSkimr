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

## Modules

The level "Modules" in the toolbar shows the namespaces of the project as modules, one box per module in the row of its layer (ADR 0108). The panel at the right has a table and three lists:

- **Table.** For every module: how many places in other modules use it (`in`), how many places in it use other modules (`out`), its **stability** `out / (in + out)` and its number of types. A stability of 0 means many depend on the module and it depends on little, so it should change rarely; 1 means the opposite. A module without any dependency has none. The unit is places in the code, not distinct modules or types.
- **Cycles.** Modules that depend on each other in a ring. Each cycle names the dependencies to take away to end it, the one with the fewest places first; this is a suggestion by the number of places to change, not a statement about which direction is right.
- **Against the layering** and **Stands out.** Dependencies between modules that the layers do not allow, and modules that many places use and that depend on a clearly less stable module.

More modules than `graph.max_modules` are not drawn or listed (the most referenced stay); cycles and the two lists always count all modules. The button "Show the places" under a dependency that ends a cycle, or that goes against the layering, lists the places of that dependency: file and line, and which symbol of one module uses which symbol of the other. It lists at most `graph.max_places` of them and says how many more there are. The same numbers are available on the command line (`codeskimr modules`, `--json` for scripts; `--from` and `--to` name a dependency and list its places) and for agents (`get_dependencies` with `level` `module`; `from_module` and `to_module` name a dependency).

## Known limits

The map shows layer relations only. A line counts as "against the layering" when its source layer has a `may_use` list that does not name the target layer ([Projects and scan](projects-and-scan.md#layers)); a layer without `may_use` has no limit. Use the map as a hint, not as a verdict.
