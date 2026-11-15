# 0061 – Level 2 maps uses of Rust module paths onto the module symbols that level 1 has

Status: proposed

Follow-up to ADR 0055 (issue #436). ADR 0055 aimed at fewer than 1 % of the Rust uses without a level 1 symbol and reached 7.5 %. This ADR explains the remainder and proposes how to close it. It supplements ADR 0033, decision 2 (mapping by file and range of the definition) and the mapping rules of `docs/level2-import.md`; it does not change ADR 0055.

## Context

### What was measured

Corpus: CodeSkimr at `bcc970e` (`corpus/rust/corpus.toml`), `rust-analyzer 1.97.0 (2d8144b 2026-07-07)`, `codeskimr` and `codeskimr_import_scip` built from `main` at 5c2c6f0 (ADR 0055 implemented), the hand-made path of `tools/accuracy/measure.py rust --scip` (index made by `rust-analyzer scip` in a clone of the pinned checkout, 5.0 MB, 231 documents of which 147 are scanned files). The run reproduces the figures of ADR 0055: mapped 8146, unmapped 659, library 6881, ignored 6374, 72 symbols created, `without_level1_symbol` 659 of 8805 (7.5 %).

The 659 were split by decoding the index (a small script that is not part of the repository) and by looking up each occurrence in the database after the import (is there a level 2 reference at that place that points to a symbol?). The two views agree to the occurrence.

| Cause | Uses | Share of 8805 |
|---|---|---|
| A. Use of a crate root (`crate/` of rust-analyzer): `use crate::x`, `codeskimr_domain::Y`, `pub(crate)`. 12 crate roots, defined in `lib.rs`, `main.rs` or `build.rs` | 448 | 5.1 % |
| B. Use of a module that lives in a `mod.rs` file (`sql/`, `ports/`, `graph_layout/`, `resolve/`, `pipeline/`, `events/`, `registry/`; 7 modules) | 131 | 1.5 % |
| C. Use of a module or crate root whose file the scan skips (24) | 24 | 0.3 % |
| D. Use of a non-module item (types, functions, constants) defined in a file the scan skips (56) | 56 | 0.6 % |
| Total | 659 | 7.5 % |

A and B together are the "603 uses of module paths" of issue #436 (579 plus the 24 of C). The issue counts C with the module paths because every one of them is a namespace descriptor; by cause it belongs with D.

Where the 603 module uses stand: 477 in `use` items, 82 in qualified paths of expressions and types (`codeskimr_codegen::load(root)`), 35 in other places (mostly the keyword in `pub(crate)` and `pub(super)`), 9 in `mod` lines.

### The premise of the issue is only half true: modules are symbols

Issue #436 says that modules are not created as symbols. Level 1 has them already (Rust pack, `languages/rust/language.toml`; blueprint 6.1): one `module` symbol per file and one `namespace` symbol per `mod` item (corpus: 147 and 150). Of the 1110 uses of module paths in the scanned files, **507 map today** (492 to a `module`, 15 to a `namespace` symbol, kind `read`, `verified`). The 603 fail for one reason: the import matches a definition by name (`docs/level2-import.md`, rule 1: "whose name is the name of the last descriptor"), and for these modules the two names differ.

- **A.** rust-analyzer calls the root of every crate `crate/`. The level 1 module of `lib.rs` is called `lib`, that of `main.rs` `main`, that of `build.rs` `build`. No name matches.
- **B.** rust-analyzer names the module of `ports/mod.rs` `ports/`. The level 1 module of that file is called `mod`.

In both cases the definition is the whole file (range from line 0, column 0; all 147 file-root definitions of the index have that shape, one per scanned file; the other 15 namespace definitions are single-line `mod` items), a symbol of kind `module` of that file spans it, and only the name check fails. The 507 uses that map today point to modules that live in a file of their own name (`error.rs` for `error/`) or are `mod` items inside a file.

ADR 0052 (proposed, not implemented) would rename the *qualified names* of module declarations (`crates::domain` for `lib.rs`), but its decision 4 keeps the module symbol per file with its file path and file name. It would not fix A (the descriptor is `crate`, never the folder name), and for B it would only change the qualified name, not the `name` that the import compares. This ADR is therefore independent of 0052 and does not wait for it.

### C and D are an artifact of the corpus configuration

`skip_folders` of the corpus matches a folder name at any depth. `crates/application/src/config/` and `crates/mcp/src/tools/` are product code, skipped only because a folder is called `config` or `tools` (35 and 12 of the 56 uses of D; the remaining 9 are `tools/codegen`). The index defines those symbols, the snapshot does not hold them, so nothing in the snapshot can be their target. That is correct behaviour (like a library), but the figure `without_level1_symbol` counts it as a gap of the mapping. It should be reported apart.

## Options

1. **Create module symbols with `origin = index`** (the mechanism of ADR 0055). It would add a second symbol for every module that level 1 already has (147 duplicates of the file modules), with two usages pages for one module and two units in the coupling. Rejected: the symbols exist, only the match fails.
2. **Rename the level 1 modules** (`lib` to the crate name, `mod` to the folder name). It would cure B and A only if the crate name is known at level 1 (it is in `Cargo.toml`, not in the file), changes names and qualified names of the 147 file modules (ADR 0052 territory, golden files, layers, snapshots), and leaves C and D. Level 1 would have to read a manifest for a naming rule that the import can settle from the index. Rejected for this problem; ADR 0052 stays its own decision.
3. **Map by position for a file root (chosen).** A definition of a namespace descriptor that spans a whole file maps onto the level 1 `module` symbol of that file, whatever its name. Nothing is guessed: the index states that this file is the module, and the symbol of that file is the module. No new symbols, no storage change.
4. **Map the uses onto the target item instead of the module** (`use crate::a::B` is a use of `B`). Level 1 and the import already do that for the item; the module in the path is a use of the module as well (rust-analyzer lists it as a reference of the module, so the usages of `sql` cannot be complete without it). Rejected as a replacement.
5. **Do not map module paths; report them apart.** Leaves 579 uses (6.6 %) without a target, against the aim of nearly compiler-exact usages (product owner, ADR 0033). Rejected for A and B. It is the right treatment for C and D (decision 4 below).

## Decision

Option 3, and the separate report for C and D.

1. **File root rule.** In `mapping`, a definition occurrence of a namespace descriptor whose range starts at line 0, column 0 of its document and ends at the end of the file, in a document that is a scanned file, maps onto the `module` symbol of that file (the one whose span starts at the first line and covers the file), whatever its name. Without such a symbol (the file has none, the file is not scanned) nothing is mapped. The name rule is unchanged for every other definition, and the rule applies to namespace descriptors only, so no function, type or constant is ever mapped by position alone. Every other occurrence of the symbol follows the existing machinery (`frozen` targets, kind of the level 1 reference at the place, else `read`, `verified`).
2. **Several roots for one symbol.** rust-analyzer gives the library and the build script of a package the same symbol `crate/` (corpus: `crates/codes` with `src/lib.rs` and `build.rs`, `crates/adapters/i18n` likewise), so the symbol has two file roots. Mapping a use to both would give the build script module uses that belong to the library. For a symbol with several roots, a use maps to (a) the root in the same file, else (b) the root whose folder is the deepest folder above the file of the use, else (c) the root that the language pack names as library root (`lib`; the pack key `root_files` of ADR 0052 is the model, a new key `library_root` is not needed if the file name `lib.rs` is read from the pack), else (d) nothing, counted as `unmapped`. Tests pin the corpus cases (a use in `build.rs`, a use in `src/x.rs` of the same package, a use from another package).
3. **No new symbol, origin or migration.** The targets are level 1 symbols of origin `syntax`. Rescan behaves as ADR 0055 decision 4 and ADR 0033 decision 13 say for every reference to a level 1 symbol (see Consequences).
4. **Count by cause.** `ImportStats` gets `outside_scan`: occurrences of a symbol that the index defines only in documents that match no file of the snapshot (skipped folders, generated code that is not scanned). They leave `unmapped` and, like `library`, stay out of the denominator of `level2.min_mapped`, because the user told the scan not to look there. `unmapped` keeps its meaning for the rest. The summary of `measure.py --scip` reports `without_level1_symbol` as `uses` (in scope), `outside_scan` and `project_uses`, and `by_cause` for the in-scope part (`module_path` while the rule above is not implemented, `other`). The change needs the new counters of the import tool (`tools/accuracy/import_scip`), so it is part of the implementation and not of this pull request. `docs/accuracy.md` and `docs/level2-import.md` name the split.
5. **Corpus.** Whether to make `skip_folders` of the Rust corpus match only top-level folders (so that `crates/application/src/config` is scanned) is not decided here (question 3).

## Measurement and acceptance

- `without_level1_symbol` for Rust on the corpus: in scope **0 of 8725** expected, **under 1 %** in any case; with `outside_scan` counted as before 80 of 8805 (0.9 %), which is under 1 % as well. The acceptance states both numbers: the in-scope share must be under 1 % and the rule of decision 4 must give the same total of 659 minus what decision 1 maps (579).
- Module symbols are checked completely, not by sample: all 147 file modules and the 150 `mod` items (297 symbols) against the references of rust-analyzer over LSP (`truth_rust.py`; `module` and `namespace` are not in `KINDS` of `measure.py` today and are added for this check). Precision and recall at least 99 %. Independence is as limited as for ADR 0055 (same engine for index and reference); the cases of decision 2 are checked by hand against the source.
- Tests: golden import tests with a crate root, a `mod.rs` module, a file with a `mod` line, a package with `lib.rs` and `build.rs`, a file root whose file is not scanned, a definition range that is not the whole file (inline `mod x { ... }`: unchanged name rule); a rescan test (changed `lib.rs` drops the references to its module and the next level 2 pass restores them); a test that level 1 results do not change.
- Budget: no new rows for symbols; references grow by 579 on 8193 verified references (7 %). Import time and memory are not expected to change measurably (one lookup per file root); the implementation reports the numbers.

## Consequences

- **UI and MCP.** The usages page of a module (`lib`, `ports`) lists the uses in paths; the 507 that exist today are `read` references. The crate root module is named `lib`, `main` or `build` in the explorer and in `find_usages` (level 1 naming, ADR 0052 is the place for a better name). A page "usages of `lib`" with 122 uses is correct but may look odd until 0052 lands. No new field, no marker, no mock needed.
- **Coupling and layer graph (ADR 0037).** Coupling counts distinct pairs of units; a reference to a module adds an edge to that unit. For crate roots this is new: about 112 files hold a use of a crate root, in about 236 pairs of file and crate root, a part of them inside one crate (a file naming its own `crate::`). The implementation measures the coupling of the corpus before and after (Ce, Ca, instability of the 12 crate roots) and reports it in the pull request. If the figures are distorted, the way out is a reference kind for path qualifiers that find_usages lists and coupling skips, which needs its own decision (blueprint 6.1; question 2).
- **Rescan.** The references to the module of `lib.rs` or `mod.rs` are level 2 references to a level 1 symbol of a file. If that file changes in a scan of MCP or UI (level 1 only), its symbol ids are replaced and the references to it from unchanged files are dropped until the next command line scan (ADR 0033, decision 13). `lib.rs` and `mod.rs` change often (a new `mod` line), so the usages page of a crate root is incomplete between a UI scan and the next command line scan more often than that of a type. This is the existing behaviour and is documented, not new.
- **Other languages.** The rule is written for namespace descriptors and a file-root definition; whether TypeScript (`index.ts` modules), Python (`__init__.py`) or C# have the same gap was not examined. Only the Rust pack uses it first; the other languages need their own measurement (as ADR 0055, risk "Other languages").
- **Docs.** `docs/level2-import.md` (rule 1 and the new counters), `docs/accuracy.md` (the split of `without_level1_symbol`), `docs/acceptance/level2.md` after the next run. No change to `BLUEPRINT.md` unless question 2 leads to a new reference kind (then 6.1, 9.4).

## Risks

- **The file-root shape is rust-analyzer's.** Another indexer may write a module definition differently (a range of the name only). The rule fires only on a definition that spans a whole file and falls back to the name rule otherwise, so the worst case is today's behaviour. The golden test pins the shape of rust-analyzer 1.97.0; the pin in `tools/accuracy/reference_tools.toml` already names that version.
- **Wrong target for shared symbols** (decision 2). A package with `lib.rs` and several `src/bin/*.rs` shares one `crate/` symbol among targets; the use of `crate::` in a binary maps by rule (a) or (b) to the right root only if the use is in the same file or below its folder. A use from a sibling folder falls to rule (c) and counts for the library root. The corpus has no case beyond `build.rs`; the risk is an over-count for the library root, never a use that disappears.
- **Skipped folders.** Counting `outside_scan` outside the denominator of `level2.min_mapped` lets an index pass the threshold that would have failed before if most of a project is skipped. The index still has to cover at least one file and map something (ADR 0033 and `docs/level2-import.md`, rule 7), so a project that skips everything still fails.

## Questions to the product owner

1. **Accept option 3** (map file-root definitions by position, no new symbols)? Recommended: yes. It closes 579 of the 603 uses with no new row and no migration; the alternative of renaming modules (option 2) changes 147 symbols and does not reach crate roots.
2. **References to a module as a path qualifier.** Today they are `read` and count in coupling, the usages page and the layer graph (507 already). Keep it for now and measure coupling in the implementation (recommended), or introduce a reference kind for path qualifiers first (a blueprint change, 6.1 and 9.4; listed in `find_usages` like `test`, left out of coupling like `test`)?
3. **The skipped folders of the Rust corpus.** `config` and `tools` match product folders at any depth (47 of the 56 uses of D). Narrow `skip_folders` to the top level (more files, new expected counts in `corpus/rust/corpus.toml`, a deliberate change as that file says) or leave the corpus and rely on the separate report of decision 4 (recommended: leave it, report apart; the corpus stays comparable with earlier figures)?
4. **`outside_scan` outside the denominator of `level2.min_mapped`** (decision 4), like `library`? Recommended: yes, because the user excluded those folders.
5. **Crate root name.** The root of a crate appears as `lib`, `main` or `build` until ADR 0052 is decided. Should 0052 be accepted and implemented before this one reaches users, or is the interim acceptable? Recommended: interim is acceptable.
