# Comparison of two scans and impact (ADR 0097, work package 3)

One use case, `codeskimr_application::compare::ScanComparer`, compares two retained scans of a project and, if asked, lists the usages that the difference hits. The registry commands, the command line, the MCP tool `impact_of_diff` and the screen "Change" (work packages 4 and 5) only call it; they add no logic of their own.

## Where it lives

- `codeskimr_domain::compare_symbols` is the pure comparison. It takes the facts of the symbols of both snapshots (`SymbolFact`: id, kind, qualified name, signature, visibility, file, content hash) and returns a `Comparison` with the lists `added`, `removed`, `changed` and `moved`. It knows no database and no snapshot id.
- `codeskimr_application::ports::ComparisonSource` reads the facts and the naming rules of one retained snapshot (adapter: `codeskimr_storage`, see [storage](storage.md)).
- `codeskimr_application::compare::ScanComparer` chooses the two scans, checks that they can be compared, reads the facts, and adds the impact, the layer changes and the critical symbols. Its limits are `compare.max_symbols` and `compare.max_usages` of `config/default.toml`.

## Rules

- **Identity.** Two symbols are the same symbol when kind, qualified name and parameter types are equal (`signature_parameters`, ADR 0068). Only types, callables, properties, fields, constants, events and enum members take part; namespaces, modules, parameters and type aliases do not. Only symbols of origin level 1 (syntax) are compared; symbols that only level 2 delivers are not, so a scan with level 2 against one without reports no false change.
- **Classes.** A symbol only the target has is *added*, one only the base has is *removed*. A symbol in both is *changed* when the signature text differs (`signature`), the visibility differs (`visibility`, with the flag `narrowed`) or, if neither differs, the content hash differs (`content`). A symbol without a content hash can change by signature or visibility only.
- **Moved.** The same key in another file is *moved*, and it also carries its reasons if it changed on the way. A renamed symbol has another key, so it is *removed* and *added* (the content hash contains the name, so it cannot tie the two together); its usages come into the impact through *removed*. If one key occurs several times in one snapshot, the symbols are paired by file first; a single leftover on each side is a move, anything else stays removed and added.
- **Order.** Every list is ordered by file, qualified name and id, so the same two scans always give the same answer.

## Choosing the scans

`ComparisonRequest { project, base, target, with_impact }`. Without `target` the newest scan is the target, without `base` the scan before the target is the base. The base must be older than the target.

| Code | When |
| --- | --- |
| `CMP-201` | a named scan is not retained, the project has fewer than two scans, or retention removed a scan while the comparison read it |
| `CMP-101` | the base is not older than the target |
| `CMP-301` | the two scans used other naming rules (retryable: a full scan makes them comparable); a scan without a record (written before ADR 0036) only matches another one without |

A comparison that was cut (see below) is not an error: the answer says it.

## Impact

With `with_impact` the use case lists, for every removed symbol and every symbol that changed in `signature` or `visibility`, the **direct** usages in the **base** snapshot (`UsageIndex::usages`, one step, tests included) with their confidence (sure or probable). Every usage carries a status: *adapted* if the symbol that holds it is itself changed, moved or removed in the comparison, *not touched* otherwise. The *not touched* usages are the checklist (`SymbolImpact::not_touched`). Symbols changed in `content` only and added symbols have no usages listed: their usages cannot break by this change.

The checklist is exact to the symbol that holds the usage, not to the line: a usage in a method that changed for another reason counts as adapted. Only direct usages are known; calls by reflection, in strings or from other repositories are not seen, and every surface says so.

## Layers and critical symbols

- `ScanComparison::layers` lists the layer edges that break the declared layering (`new_violations`) or lie on a dependency cycle (`new_cycles`) in the target and not in the base. It compares the stored layer graph (`CodeIndex::layer_edges`) of both snapshots, edge by edge (from layer, to layer).
- `ScanComparison::critical` names the symbols with a critical mark (ADR 0068) that the difference removes, moves or changes, found by the same key as the mark.

## Limits

`compare.max_symbols` (default 200) cuts each list of symbols and the list of symbols whose usages are read; `compare.max_usages` (default 50) cuts the usages per symbol. Every list is a `Listed<T>` with its `total`, so `is_cut()` tells that entries were left out and by how much. `ScanComparison::is_cut` says it for the whole answer. The critical symbols and the layer changes are not cut.

## The four ways in (work package 4)

All four call `ScanComparer` and print the same answer (`codeskimr_application::compare::ComparisonAnswer`, `"schema": 1`); only the language of `note` differs.

| Way in | Name | Impact |
|---|---|---|
| Registry command (REST, screen) | `scans.list`, `scans.compare`, `scans.impact`, `scans.impact_markdown` (`docs/server.md`) | `scans.compare` without, `scans.impact` with (a superset: it also holds the lists of `scans.compare`); `scans.impact_markdown` gives the text of `change --markdown` |
| Command line | `codeskimr scans`, `compare`, `change` (`docs/cli.md`) | `change` with; `change --markdown` for a comment on a pull request |
| MCP | `impact_of_diff` (`docs/mcp.md`) | with |

The answer says what it does not see (`direct_usages_only`, `note_key` and `note`: "Only direct usages are known; reflection, strings and other repositories are not seen by the comparison") and says when it was cut (`total` and `is_cut` per list, `is_cut` for the whole). `base` and `target` name both scans. The `change --markdown` text is made from the same answer as `--json`: first line `<!-- codeskimr-change -->`, the numbers, the checklist of the usages that were not touched, new layer violations and cycles, critical symbols, a line if cut, and the note.

## Reading and locks

The use case reads the history, the naming rules and the facts of the two snapshots, the usages and the annotations. Every read is one short read transaction of its own; none is held while the next runs, so a running scan is never blocked. Snapshots are not changed once written (except by a level 2 import into the newest one), so the reads fit together; after the reads the use case asks the history once more and answers `CMP-201` if retention removed a scan in between.
