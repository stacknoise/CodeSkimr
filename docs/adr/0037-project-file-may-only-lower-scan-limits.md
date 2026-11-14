# 0037 – The project file may only lower the scan limits

Status: proposed

## Context

Blueprint 14 applies the project file `.codeskimr.toml` after the user file and before the command line flags. The file lies in the scanned repository, which may be foreign (#54). Two settings protect the machine that scans: `scan.max_source_mb` (largest source) and `scan.parse_timeout_ms` (longest parse of one file). A foreign repository that sets them to huge values removes the protection. ADR 0033 (decision 9) already makes the whole `level2` table untouchable for the project file.

## Options

1. Leave it: the project file wins like any layer.
2. Drop the two settings from the project file, as for `level2`. A project could no longer ask for a stricter limit.
3. Let the project file lower them but not raise them above what the defaults and the user file say.

## Decision

Option 3. While the layers are merged, a value of the project file for `scan.max_source_mb` or `scan.parse_timeout_ms` that is higher than the value merged so far is ignored. The user file and the command line flags can still raise them. Other settings keep the order of blueprint 14. Nothing is reported for an ignored value; it is documented in `config/default.toml`, `docs/server.md` and the manual.

## Consequences

- A scanned repository cannot raise the limits of the machine that scans it. A project that really is larger than 2048 MB needs `--set scan.max_source_mb=…` or an entry in the user file.
- Other project file settings that act on the machine (`editor.allowed_schemes`, `ui.dir`, `server.port`) are not covered here and are tracked in a separate issue.
- The rule is one function (`lower_only` in `crates/application/src/config/mod.rs`) with its tests; further limits join the list `LOWER_ONLY_SCAN_KEYS`.
