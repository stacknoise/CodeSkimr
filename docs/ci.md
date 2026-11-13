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
| `architecture` | crate dependency direction (`tools/ci/architecture.toml`, rule file must match the workspace) and module cycles between the top-level modules of each crate (`tools/ci/check_architecture.py`, own unit tests) | 4.2, R-04, 17.3 (5) |
| `duplicates` | `jscpd`, threshold 0, 5 lines / 50 tokens (`.jscpd.json`) | R-01 |
| `literals` | placeholder until the Dylint lint exists | R-06 |
| `codegen` | regenerates the error code artifacts and fails on any diff; i18n keys, tokens and OpenAPI client follow | 17.3 (6) |
| `bench` | criterion benchmarks of `tools/bench` for the base commit and the change on the same runner; fails on a slowdown above the tolerance or a budget breach (`docs/benchmarks.md`) | 16.1 |
| `e2e` | placeholder until the frontend project (`web/package.json`) exists | 17.3 |

The placeholders (`literals`, `e2e`) pass with a clearly printed "NOT ACTIVE YET" notice.
As soon as `tools/lints` or `web/package.json` appear, the matching job fails on purpose until it is wired up, so a placeholder cannot be forgotten.

## Changing the pipeline

Never rename a job without updating the ruleset. Never loosen a check to get a PR green.
When a check is added, tell the product owner so it can be added as required.

## Temporary exceptions

- `[workspace.metadata.cargo-machete]` in `Cargo.toml` ignores the declared but not yet used workspace dependencies of the skeleton crates. Remove entries as crates get code; the list must be empty at milestone M1.

## Architecture check: scope and limits

- Crate rules: direct dependencies only (normal, dev and build). Cargo rejects crate cycles itself.
- Module cycles: nodes are the top-level modules of a crate (`src/<name>.rs` or `src/<name>/`); edges come from `crate::`, `super::` and `self::` paths (comments and string literals are ignored). `lib.rs` and `main.rs` are the root and not a node. Cycles hidden behind re-exports or `use` of a glob from the root are not seen; the check can only report too little, never a false cycle from text in comments.
