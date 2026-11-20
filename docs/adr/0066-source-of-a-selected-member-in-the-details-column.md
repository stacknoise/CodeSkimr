# 0066 – The source of a selected member sits in the details column

Status: proposed

## Context

Blueprint B.2 and the mock `Explorer.dc.html` show three columns: tree, details and source. Since PR #491 a member selected in the tree hides the facts and the member table, because the member list makes no sense there. The middle column then stayed empty and the source stayed on the right (finding of the owner, third test version, 2026-11-16).

## Decision

1. A type selected in the tree keeps the three columns, as in the mock.
2. A member selected in the tree shows two columns: the tree, and a middle column with breadcrumb, name, kind chip and the buttons (usages, graph, open in editor) on top and the source directly below, where the member table sits for a type. The source column on the right and its separator are gone.
3. The side panels of the dependency map and of the usages can be widened like the other columns (separator, keys, remembered width). This changes no rule of the blueprint.

## Consequences

- Blueprint B.2 names the two-column case. The mock stays as it is: it shows the type view.
- No new dependency, no new token.
