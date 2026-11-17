# 0069 – Time-limited exception for the unmaintained `chrono` advisory

Status: accepted

## Context

Since 2026-11-17 the required check `deps` fails on every pull request. `cargo audit --deny warnings` reports RUSTSEC-2026-0543: the crate `chrono` is unmaintained (informational, no vulnerability, no patched version; the advisory recommends `jiff` or `time`).

`chrono` 0.4.45 reaches the product only indirectly: `rmcp` (the MCP library, blueprint 4) depends on it in every version we checked (0.8, 1.0, 2.0, 3.0, 3.5.1) up to the newest, 3.10.0, and has no feature that switches it off. `rmcp` also enables the feature `chrono04` of `schemars`. `chrono` is in no other crate of ours and is not used by CodeSkimr's own code.

## Options

1. **Replace `rmcp`** by an own MCP implementation or another library. Large work package for an advisory without a vulnerability, and it blocks all open pull requests meanwhile.
2. **Time-limited exception** for exactly this advisory, with the reason next to it and a review date.
3. Keep `deps` red. Not possible: no pull request can be merged.

## Decision

Option 2. `deny.toml` and the `cargo audit` step of the `deps` job ignore exactly RUSTSEC-2026-0543, with the reason next to it, **until 2027-02-28**. Nothing else is loosened: `--deny warnings` stays, and every other advisory, yanked crate or license problem still fails the check.

## Consequences

- On or before 2027-02-28 the exception is reviewed: removed if `rmcp` has dropped `chrono`, otherwise renewed by a new ADR or `rmcp` is replaced.
- If `chrono` ever gets a vulnerability advisory (not "unmaintained"), it is a different identifier and is not covered; the check fails as usual.
- The exception follows the pattern of ADR 0043 (`paste`).
