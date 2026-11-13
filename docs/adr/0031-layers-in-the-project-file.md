# 0031 – Layers of a project come from `.codeskimr.toml`

Status: proposed

## Context

The project model has layers (blueprint 6.1: `Project.layers`, `LayerEdge`), the `layer_graph` read model aggregates references per pair of layers, and `get_architecture`, `impact_of_change` and `get_symbol` report them. But nothing could set them: a project was registered with an empty list, so every answer said `layers: []` (acceptance report M1/M2, issue #111).

## Decision

1. **Layers are configuration.** The key `layers` of the configuration (blueprint 14) is a list of tables, written as `[[layers]]` in the project's `.codeskimr.toml` (or in the user file or with `--set`):
   ```toml
   [[layers]]
   name = "domain"
   namespaces = ["crates::domain"]
   ```
   `name` is the layer's display name, `namespaces` are qualified-name prefixes. The order is the order of the layers; the first layer that owns a name wins.
2. **A layer owns a name** that equals one of its namespaces or lies below it, up to a separator. The separator is `.` or `::`, so Rust module paths work like C# namespaces.
3. **Validation (`CFG-101` on `layers`):** a name must be valid and unique, and a layer needs at least one valid namespace.
4. **The configuration is the source of truth.** On every scan or reopen of a project, the layers of the resolved configuration are written to the stored project when they differ (an empty list removes them). The read models of the next snapshot are built with them.
5. **No detection.** CodeSkimr does not derive layers from folders or namespaces. A guess would be wrong for some project and silently so; an explicit list is short and reviewable.
6. **Allowed directions are not part of this decision.** They belong to issue #45 and will extend the table (for example a `may_use` list per layer); `violates_layering` stays `false` until then.
7. CodeSkimr's own layers are listed in the repository's `.codeskimr.toml` (blueprint 4.2).

## Consequences

- `get_architecture`, the layer graph and `layer` in `get_symbol` and `impact_of_change` have data for every project that lists its layers.
- Changing the layers takes effect with the next scan.
- The blueprint text is not changed: section 6.1 already has `layers`; section 14 only lacks the key, which `config/default.toml` and `docs/cli.md` now show.
