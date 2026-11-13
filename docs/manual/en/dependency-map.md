# Dependency map

The dependency map shows the **layers** of the active project as boxes and how they depend on each other. It answers: does the program follow its architecture, and where not?

It needs layers. Declare them in `.codeskimr.toml` and scan again ([Projects and scan](projects-and-scan.md#layers)). Without layers the screen says "No layers to show".

## Reading the map

- One box per layer, one row per layer, outer layers on top (the order of your `[[layers]]`).
- A line between two boxes means "uses". Its label counts the references behind it.
- Lines against the declared layering are marked in the violation colour, and the legend shows how many there are. A banner above lists the violations ("Show" selects them). Lines that belong to a cycle are marked too.

The layout is calculated by CodeSkimr (layered drawing), so the same project always gives the same picture.

## Using it

- Mouse wheel zooms; hold `Space` and drag to pan; `F` (or the button "Fit to view") fits the map to the window.
- Click a box to select it. Its lines are coloured by direction ("uses" and "used by"), all other lines turn weak. The side panel shows the layer, its number of references, what it **uses** and what it is **used by**, each with the count.
- The switch "all connections weak" shows or hides the lines that do not belong to the selection; "violations only" hides every line that does not break the layering.
- `Enter` with a layer selected opens the explorer for the project.

## Known limits

The map shows layer relations only. Which direction between two layers is **allowed** is not part of the project model yet; what counts as "against the layering" follows the layer graph that the scan recorded (see the open issue #45 about the layer rule). Use it as a hint, not as a verdict.
