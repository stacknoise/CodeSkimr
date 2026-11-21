# 0082 – The layer rule (`CHK-103`) in the convention check

Status: accepted

## Context

`check_conventions` could not check rule R-04 (dependencies run in the allowed directions, `CHK-103`). The project model had no allowed directions until ADR 0054 added `may_use` to a layer, and the layer graph of a snapshot (`violates_layering`) records which edges between layers break it. The convention check analyses files afresh and has no layer of its own for a reference: level 1 only reports a reference as a name, and the layer of a target is known after the scan has resolved it (issue #93).

## Decision

1. **The rule reads the stored references of the newest snapshot.** `check_conventions` already has the newest snapshot of the project; the new port `ReferenceSites` (`codeskimr_application::ports`, adapter `codeskimr_storage`) returns the certain references that start in the checked files, with both ends named. References of kind `test` and references whose name matches several symbols are left out, the same filter as the layer graph (ADR 0054, issue #464).
2. **The layer of a reference end** is the layer that owns the qualified name (`Layer::owns`, first layer wins), as in the layer graph. A reference between two names of one layer, or to or from a name that no layer owns, is no violation. A reference violates the rule when the layer of its source limits its dependencies (`may_use`) and does not list the layer of its target (`Layer::allows`).
3. **One finding per function and forbidden layer**, `CHK-103` `layer_violation`, with the layer of the function, the layer it reaches into and the number of references (`from_layer`, `to_layer`, `uses` in the answer of the MCP tool, `cli-check-layer` for the text). The finding stands at the declaration of the function as the snapshot stored it.
4. **When the rule is not checked.** `CHK-103` stays in `not_checked` when the check has no stored project (the command line has no store) or when no layer of the project has a `may_use`; without a limit nothing can be violated and a clean answer would say nothing.
5. **The stored state is the state of the last scan.** Unlike the other rules, `CHK-103` does not see edits that no scan has seen: a changed reference or a moved function shows up after `rescan`. The line of a finding is the line of the last scan, too.

## Consequences

- The MCP tool reports layering violations of the checked files without a new scan of the whole project.
- `codeskimr check` still reports `CHK-103` as not checked; a later change can give it the store if it is wanted.
- A reference from a function that a scan did not resolve (an unknown name, a dynamic call) is invisible to the rule, like for the layer graph.
- `StoredProject` carries the references port and the layers of the project; `Ports` of the MCP crate has one more field, `references`.
