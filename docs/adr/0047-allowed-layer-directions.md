# 0047 – Allowed directions between layers: `may_use`

Status: accepted

## Context

ADR 0031 let a project list its layers, but left the allowed directions open (decision 6, issue #45). So `violates_layering` was always false and `get_architecture` could not say which dependencies are allowed (blueprint 10.4).

## Decision

1. A layer in `[[layers]]` may carry `may_use`, a list of layer names it may depend on. Missing means no limit; an empty list means the layer may depend on no other layer. A dependency inside one layer is always allowed.
2. Every name in `may_use` must be a declared layer (`CFG-101` on `layers` otherwise).
3. An edge of the layer graph violates the layering when its starting layer has a `may_use` list that does not name its target layer. This replaces the constant `false` of `violates_layering`.
4. The lists are saved with the project (schema version 11: `layers.restricted` and the table `layer_uses`). Layers saved before keep no limit.
5. `get_architecture` shows `may_use` per layer (`null` without a limit) and a `conventions` object with the limits of `check_conventions`.
6. The layer rule `CHK-103` of `check_conventions` is not part of this decision. It needs the layer of each reference and follows with #93's package.
7. This repository's `.codeskimr.toml` lists the directions of blueprint 4.2. The `i18n` adapter is a layer of its own, because the driving adapters may use it and no other adapter. The rule that adapters do not use each other cannot be written with one layer for all adapters; `architecture` in CI keeps checking it per crate.

## Consequences

- No behaviour changes for projects without `may_use`.
- The next scan of a project with `may_use` marks the violating edges.
