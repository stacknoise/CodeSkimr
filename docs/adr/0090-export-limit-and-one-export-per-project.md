# 0090 – Size limit of the HTML export and one export per project

Status: accepted (decided by the product owner on a decision card, 2026-11-23: "600,000 references, adjustable"; extends ADR 0035 and the export of blueprint 15, changes none of their decisions)

## Context

The HTML export reads every symbol of a project one by one through the commands of the screens and embeds the answers. Measured on 2026-11-23, the page takes about 250 bytes per reference (148,000 references give 37 MB). A project of the size of `microsoft/vscode` (about 3 million references, 600,000 symbols) would give a page of about 750 MB that no browser opens, and the export would run for a very long time while it holds the server (issues #658 and #662). The app also let two exports of the same project run at once, for example after a click that was given up by the client.

## Options

1. Do nothing: the export of a very large project runs for hours and gives a file nobody can open.
2. Refuse above a fixed limit that cannot be changed.
3. Refuse above a limit that is a setting, with a clear message, and let only one export per project run.

## Decision

Option 3.

- New setting `export.max_references` (default 600000, about 150 MB of page). The export compares it with the number of references of the newest snapshot before it reads anything and refuses with the new code `EXP-502` (`export_too_large`). The user can raise the value in the own config file at their own risk.
- Only one `export.html` per project runs at a time. A second call fails at once with `EXP-501` (`export_running`, retryable). The guard is released when the export ends, whether it succeeds or fails.
- The count of the direct usages without tests, which the export needs for every symbol, is computed with a walk of depth 1 only, not with the full usages answer including the impact analysis. The page is byte-identical, the export takes about half the time.

## Consequences

- A project above the limit cannot be exported as one HTML page until the limit is raised; a future export in parts or on request per screen is possible but not part of this decision.
- Two new error codes are registered.
- The setting is not one of the project-lowerable keys: it protects the user's own machine from the user's own request, not from a scanned repository.
