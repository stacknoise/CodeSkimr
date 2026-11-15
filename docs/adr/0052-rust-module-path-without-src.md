# 0052 – Rust module paths do not contain `src` or the file name of a module root

Status: accepted (Anton, 15 November 2026, decision card, option "Annehmen"; implementation is a separate work package, issue #336)

Amends ADR 0036 for Rust: the file stays the first segment of a name, but `src` and the root file names are not part of it.

## Context

ADR 0036 leaves the file as the first segment of every qualified name for Rust, Python, TypeScript and JavaScript, because for those languages the path of the file is the module path. For Rust that holds only in part. A Cargo project has the folders `crates/domain/src/`, and the Rust compiler never names `src`; and `lib.rs`, `main.rs` and `mod.rs` are the module of their folder, not a module called `lib`. So a type in `crates/domain/src/reverse_index.rs` is named `crates::domain::src::reverse_index::ReverseIndex`, where a Rust developer says `domain::reverse_index::ReverseIndex`, and the items of `crates/domain/src/lib.rs` live under `crates::domain::src::lib` (issue #336). The explorer tree, the namespace level of the map and the quick search show these names, so they are long and carry two words that mean nothing.

The layers of a project (ADR 0031) name prefixes of these names. This repository's `.codeskimr.toml` uses `crates::domain`, `crates::adapters` and so on. Those prefixes stay valid if `src` and the root file name are removed, because they stop before `src`.

## Options

1. Hide `src` and `lib` only in the display. Layers, search, MCP and SCIP would still see the long form; ADR 0036 rejected this for the same reason.
2. Let the pipeline drop `src` for all languages. `src` means nothing in Python or TypeScript, but it can be a real package name there.
3. Change the names in the Rust pack with two pack keys (chosen below), as ADR 0036 did with `file_in_names`.

## Decision

1. **Two pack keys in `language.toml`** for languages whose names contain the file path: `skip_path_segments` (list of folder names that are left out, default empty) and `root_files` (list of file names that stand for their folder, default empty).
2. **Rust sets `skip_path_segments = ["src"]` and `root_files = ["lib", "main", "mod"]`.** The file `crates/domain/src/reverse_index.rs` gives the module `crates::domain::reverse_index`; `crates/domain/src/lib.rs` gives `crates::domain`; `crates/app/src/main.rs` gives `crates::app`; `crates/x/src/a/mod.rs` gives `crates::x::a`.
3. **`lib.rs` and `main.rs` of one package** both stand for the package folder, so their declarations share the module name `crates::app`. That is accepted: names are not keys (ADR 0036, decision 4), the module symbol of each file keeps its own path, and ids tell two items of the same name apart. `src/bin/tool.rs`, `tests/` and `examples/` are not special: `crates::x::bin::tool`, `crates::x::tests::case` (only `src` and the three root file names are removed).
4. **The module symbol keeps the full path of the file as before** (ADR 0036, decision 3), so it stays unique per file. Only the names of declarations and of the module path above them change.
5. **Layers need no change** for projects whose prefixes stop before `src`, as this repository's do. A layer that names `src` (`crates::domain::src`) owns nothing after the change; the release notes and `docs/language-packs.md` say so.
6. **Existing snapshots** follow the rule of ADR 0036, decision 5: the naming rules of all packs are recorded with the snapshot (`snapshots.naming_rules`) and an incremental scan whose rules differ is a full scan. The two new keys are part of that record.
7. **Golden tests** of the Rust pack and the layer examples in tests and docs are regenerated.

## Consequences

- Names of Rust symbols match what the compiler, rust-analyzer and the SCIP index (ADR 0033) call them, so the match of level-2 hits becomes simpler.
- Two files can now have the same module name (`a.rs` and `a/mod.rs` do not occur together in a valid crate; two crates with `src/lib.rs` have different crate folders). Names stay no keys; ids decide (ADR 0036, consequences).
- Checklist for the implementation PR, as in ADR 0036: `Layer::owns`, the namespace graph and its depth merge, `type_neighbours`, the resolver lookup, the SCIP name match, the explorer outline, `get_symbol`, the golden files of the Rust pack, and the accuracy corpus (`docs/accuracy.md`), which must not get worse.
- Python and TypeScript keep their names; they can use the keys later without a new ADR.
