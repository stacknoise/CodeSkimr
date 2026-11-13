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
| `literals` | Dylint lint `no_literals` on the product crates (`tools/lints/no_literals`, own test cases), ESLint rules from `web/eslint.literals.config.js` once `web/package.json` exists | R-06, 3.2 |
| `codegen` | regenerates the error code artifacts and fails on any diff; i18n keys, tokens and OpenAPI client follow | 17.3 (6) |
| `bench` | criterion benchmarks of `tools/bench` for the base commit and the change on the same runner; fails on a slowdown above the tolerance or a budget breach (`docs/benchmarks.md`) | 16.1 |
| `e2e` | placeholder until the frontend project (`web/package.json`) exists | 17.3 |

The placeholder `e2e` passes with a clearly printed "NOT ACTIVE YET" notice.
As soon as `web/package.json` appears, the job fails on purpose until it is wired up, so a placeholder cannot be forgotten.

## The `no_literals` lint

Rule R-06 is enforced by a Dylint lint in `tools/lints/no_literals`. It is a separate Cargo workspace with its own pinned nightly toolchain (`rust-toolchain.toml`), because lints link against compiler internals.

- Run it from the repository root: the check for `tests/` and `benches/` looks for `Cargo.toml` by relative path. Elsewhere, test files would be reported (safe direction).
- Run it locally: `cargo install --locked cargo-dylint@6.4.0 dylint-link@6.4.0`, then `cargo dylint --path tools/lints/no_literals --workspace` in the repository root.
- Its test cases are UI tests in `tools/lints/no_literals/ui/` (`flagged.rs`, `exempt.rs`, `consts.rs`, `build.rs` with their `.stderr`). Run them with `cargo test` inside the lint folder. When the lint output changes on purpose, copy the new actual output (printed as `/tmp/<name>.stage-id.stderr`) over the `.stderr` file.
- What is reported: numbers other than `0` and `1` (also `0.0`), strings, characters, byte strings, floats, enum discriminants, array lengths and `concat!` calls and format strings with fixed text (`format!("{a}/{b}")`; `println!`/`eprintln!` always, because of the line break, `write!(f, "{x}")` only if it has fixed text). Error code `CHK-101`.
- What is not reported (blueprint 3.2): `0`, `1`, booleans; test code (`#[cfg(test)]`, `#[test]`, and files in `tests/` or `benches/` directly below a crate root; a module named `tests` inside `src/` is still checked); the value of a `const` or `static` in a file named `consts.rs`; build scripts (`build.rs`); attributes and doc comments; text produced by foreign macros (derives, `std`).
- Interpretation: developer tooling under `tools/` (code generator, CI scripts) and generated code in `OUT_DIR` are not product code and are not checked. A literal written inside a local `macro_rules!` is reported in the crate that defines it. A macro from another workspace crate is not followed (issue #26). Code behind `cfg(feature)` or another platform is not compiled by the Linux job and not checked. These interpretations still need an ADR (issue #24).
- Moving the toolchain: change the date in `rust-toolchain.toml` together with `dylint_linting` and `dylint_testing` (`cargo dylint new` shows the matching date), fix the lint if the compiler API changed, and run the lint tests.
- TypeScript: `web/eslint.literals.config.js` sets `no-magic-numbers` (only 0 and 1) and `i18next/no-literal-string`, with the same exceptions (`*.test.ts`, `*.spec.ts`, `tests/`, `consts.ts`, generated files). The job runs it as soon as `web/package.json` exists, so that WP for the frontend must add `eslint`, `typescript-eslint` and `eslint-plugin-i18next` as dev dependencies.

## Changing the pipeline

Never rename a job without updating the ruleset. Never loosen a check to get a PR green.
When a check is added, tell the product owner so it can be added as required.

## Architecture check: scope and limits

- Crate rules: direct dependencies only (normal, dev and build). Cargo rejects crate cycles itself.
- Module cycles: nodes are the top-level modules of a crate (`src/<name>.rs` or `src/<name>/`); edges come from `crate::`, `super::` and `self::` paths (comments and string literals are ignored). `lib.rs` and `main.rs` are the root and not a node. Cycles hidden behind re-exports or `use` of a glob from the root are not seen; the check can only report too little, never a false cycle from text in comments.
