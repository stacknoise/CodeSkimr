# CI

The pipeline is `.github/workflows/ci.yml` and implements blueprint section 17.3.
All actions are pinned to commit SHAs, the workflow runs with `contents: read`, and Rust builds are cached.

## Required checks

These names must be entered as required status checks in the ruleset `main` (names match the job names exactly):

| Check | What it does | Blueprint |
|---|---|---|
| `pr-title` | PR title follows Conventional Commits | CLAUDE.md rule 3 |
| `fmt` | `cargo fmt --check`, `prettier --check` once `web/` exists | 17.3 (1) |
| `clippy` | clippy with workspace lints, warnings are errors | 17.3 (2) |
| `test` | `cargo test --workspace` | 17.3 (8) |
| `deps` | `cargo audit`, `cargo deny` (`deny.toml`), `cargo machete` | 17.3 (10), S-03 |
| `build (ubuntu-latest)` | release build | 17.3 (11) |
| `build (windows-latest)` | release build | 17.3 (11) |
| `build (macos-latest)` | release build | 17.3 (11) |
| `architecture` | crate dependency direction (`tools/ci/architecture.toml`) | 4.2, R-04 |
| `duplicates` | `jscpd`, threshold 0, 5 lines / 50 tokens (`.jscpd.json`) | R-01 |
| `literals` | placeholder until the Dylint lint exists | R-06 |
| `codegen` | placeholder until the generators exist | 17.3 (6) |
| `bench` | placeholder until benchmarks exist | 16.1 |
| `e2e` | placeholder until `web/` exists | 17.3 |

The four placeholders pass with a clearly printed "NOT ACTIVE YET" notice.
As soon as `tools/lints`, `tools/codegen`, `tools/bench` or `web/` appear, the matching job fails on purpose until it is wired up, so a placeholder cannot be forgotten.

## Changing the pipeline

Never rename a job without updating the ruleset. Never loosen a check to get a PR green.
When a check is added, tell the product owner so it can be added as required.

## Temporary exceptions

- `[workspace.metadata.cargo-machete]` in `Cargo.toml` ignores the declared but not yet used workspace dependencies of the skeleton crates. Remove entries as crates get code; the list must be empty at milestone M1.
