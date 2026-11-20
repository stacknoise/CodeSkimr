# 0076 – The master switch of level 2 is the user configuration only

Status: accepted (Anton, 2026-11-20, decision card; issue #556)

## Context

ADR 0033 (decision 9) says level 2 runs if the user enabled it "in the user configuration or the user's own project record" and confirmed it once. ADR 0071 (decision 8) calls `[level2] enabled` in the user configuration the master switch: the program never writes it, and the screen shows `not_enabled` without it.

The two do not fit together. `level2::confirm` writes `enabled: true` into the project record, and `authorize_run` (and `RunPermit::recheck`) accepted `(enabled in config || record.enabled) && confirmed`. A confirmed project therefore ran at `codeskimr scan` although `[level2] enabled` was missing. The security review of PR #555 found it. ADR 0072 already closed the gap for scans from the browser (`Session::start_level2` requires the switch); the terminal was open.

## Options considered

1. **The switch is required everywhere (chosen).** `authorize_run` and `RunPermit::recheck` want `[level2] enabled = true` in the user configuration and a confirmation for the current source. The terminal, watch mode and the browser follow one rule. The `enabled` flag of the record stays as data (the confirmation writes it, the screens read it) but no longer opens anything.
2. **Change ADR 0071 instead:** the confirmation alone enables level 2. Then "not enabled" in the screen would mean nothing, and the master key would sit in a record that a confirmation writes. The user who never set the switch would find level 2 running after a single confirmation, which is not what the switch promises.

## Decision

1. `authorize_run` returns a permit only if the principal holds `RunIndexer`, `[level2] enabled` is true in the user configuration **and** the project record holds a confirmation for the current source, location and branch. `RunPermit::recheck` checks the same again before each process starts.
2. The result is `SEC-301` (`indexer_not_confirmed`) if the switch is off or the confirmation is missing; its description and texts say so.
3. ADR 0033, decision 9 is read as "user configuration" for the switch; the "or the project record" part is void. The record keeps the confirmation (decision 10 stays).
4. The project file `.codeskimr.toml` still cannot set the switch (ADR 0033, decision 9, second sentence).
5. A user who confirmed a project before this change and never set the switch gets `SEC-301` at the next level 2 run; the manual names the two lines for `config.toml` (ADR 0071). No data is migrated.

## Consequences

- One rule for all callers; the answer to "who may start build tools" is the switch in a file the program never writes plus the owner's confirmation.
- Tests: `crates/application/tests/level2.rs` (a confirmation alone does not run; switch plus confirmation runs until revoked).
- Blueprint: description of `SEC-301`; `docs/level2-runner.md`, `docs/level2-import.md` and the locale texts follow.
