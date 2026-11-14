# Accuracy of usages per language (issue #134)

`tools/accuracy/measure.py` measures how well `find_usages` agrees with the reference tooling of a language on the pinned public corpora (`corpus/<language>/corpus.toml`). It is a measurement, not a CI gate.

```
python3 tools/accuracy/measure.py python --samples 150 --seed 1 --codeskimr target/debug/codeskimr
python3 tools/accuracy/measure.py typescript --samples 150 --seed 1
python3 tools/accuracy/measure.py rust --samples 150 --seed 1
python3 tools/accuracy/measure.py java --samples 150 --seed 1
```

`--json` adds the numbers per symbol kind, the wrong usages grouped by our reference kind, and up to five wrong and five missed lines per symbol. `--verified-only` counts only references with `verified = 1` (level 2); this is the measure of ADR 0033, decision 14, once the level 2 pass is wired into the scan. Without it, level 1 and level 2 references are counted together.

Steps: the script copies the corpus sources (without the skipped folders), scans them with `codeskimr scan`, draws symbols with a fixed seed from the list sorted by qualified name (so two versions see the same symbols; symbol kinds that can be used, names starting with an underscore are left out, which also leaves out all Python constructors), asks the reference tool where each symbol is used, and compares by file and line.

| Language | Reference tool | Needs |
|---|---|---|
| Python | Jedi `get_references` | `pip install jedi` |
| TypeScript | TypeScript language service `findReferences` | `typescript` (set `TYPESCRIPT_MODULE` and `NODE_PATH` if it is not installed locally) |
| Rust | rust-analyzer over LSP `textDocument/references` (`truth_rust.py`) | `rust-analyzer`, `cargo` and network access for the crates of the corpus project (rust-analyzer loads the full checkout; build scripts and proc macros are off) |
| Java | `javac` itself (`TruthJava.java`, run with `java TruthJava.java`): parse and attribute all sources with `JavacTask`, every identifier, member select, member reference and `new` that the compiler resolves to a declaration is a use | JDK 17 or newer; the pinned compile-time dependencies and the template substitution of `tools/accuracy/reference_tools.toml` (one jar, SHA-256 checked, cached in `target/accuracy-cache`) |

`--scip INDEX` measures level 2 before the scan wires it in (and without the consent of the owner, which is not touched): the index is made by hand with `rust-analyzer scip <checkout> --output index.scip`, `tools/accuracy/import_scip` (a standalone Cargo project, build it with `cargo build --manifest-path tools/accuracy/import_scip/Cargo.toml --target-dir target`) maps it with the public `codeskimr_scip` library onto the scanned database, and the comparison runs on that database; `--scip` needs `--verified-only` (the script refuses otherwise), warns if `rust-analyzer --version` is not the pinned one (`reference_tools.toml`, `[rust_scip]`: `rust-analyzer 1.97.0 (2d8144b 2026-07-07)`) and stops if the index does not cover every scanned file, which is how a foreign index (other corpus commit) shows. The index must come from the checkout of the commit in `corpus/rust/corpus.toml`. The tool calls `convert_columns` like the product (the index of rust-analyzer is UTF-8, so it changes nothing here) and reads the limits from `config/default.toml`. The tool writes only into the throw-away database of the measurement. `--import-scip` names the tool. **What this does not measure:** the `IndexRunner` (call, arguments, limits, process group, environment), the consent and the wiring into the scan are not exercised, because the index is made by hand. The numbers are therefore not an acceptance under ADR 0033 decision 14 (threshold 99 %, here 91.8 %); they measure import and mapping only.

The Java reference (javac) does not read a SCIP index and is independent of the level 2 indexer (ADR 0033, decision 14). For Rust the reference tool and the indexer are the same engine: the reference is rust-analyzer over LSP, the index comes from `rust-analyzer scip`. The reference does not read the index file, which meets the letter of decision 14, but agreement of 100 % precision is partly an engine agreeing with itself, and errors of rust-analyzer do not show. The two runs also differ: LSP runs without build scripts and proc macros, `scip` runs them.

Numbers over all drawn symbols: precision = usages of ours that the tool confirms / usages of ours; recall = usages of the tool that we found / usages of the tool; exact = share of symbols whose set of lines is identical. To compare two versions, pass the binary and the language packs that belong to it (`--codeskimr`, `--languages`).

## Limits of the comparison

- The sample is small: with 60 symbols one symbol is 1.7 percentage points. Use `--samples 150` or more to compare versions.
- A symbol with no usage on either side counts as exact.
- Imports are left out on our side only (they are not usages), which lowers recall a little.
- Precision and recall are summed over all usages, so symbols with many usages weigh more than the exact rate shows.
- The TypeScript run uses a non-strict configuration with bundler module resolution, not the `tsconfig` of the project.
- Comparison is by line, not by position. Two usages in one line count once.
- The reference tools are not complete either: Jedi does not follow attributes of instances in many cases, so some usages of ours that are right count as wrong. Read precision for Python as a lower bound.
- Covered: Python, TypeScript, Rust and Java. C# and Kotlin (compilers not available in CI containers) follow.
- Rust: rust-analyzer runs without build scripts and proc macros, so code that a macro generates has no references; `pub use` re-exports count as uses for rust-analyzer but not for us (we leave imports out). Uses in the skipped folders (tests, benches) are dropped from the reference, because we did not scan them.
- Java: `module-info.java` is not compiled, the language level is 17, and generated sources (here `GsonBuildConfig`) are compiled but never reported. The compiler tells overloads apart; we do not. A method that overrides another has no uses of its own for the compiler, while we also attach calls through the base type to every override (dispatch, `probable`).
- The level 1 numbers below are the baseline. The Rust level 2 numbers come from a hand-made index (see `--scip`), not from the scan; the level 2 pass is not measured through the scan yet.

## Results

First measurement for Rust and Java (150 symbols, seed 1, 2026-11-14, level 1, all references; CodeSkimr built from `main` at 8c83933). Corpora: Rust = CodeSkimr at `bcc970e90cb99c2d052457f79090a78194dfaa8e`, Java = gson at `a961b0fe088f8558d00cc1b56e506a3656907a91`. Tools: rust-analyzer 1.97.0 (2d8144b 2026-07-07), JDK 21.0.12 (`javac` 21.0.12.1, compiled with `--release 17`):

| Language | symbols | usages ours / tool | exact | precision ("correct") | recall |
|---|---|---|---|---|---|
| Rust (CodeSkimr at the pinned corpus commit) | 150 | 735 / 733 | 27.3 % | 59.2 % | 59.3 % |
| Java (gson at the pinned corpus commit) | 150 | 1001 / 590 | 54.0 % | 46.7 % | 79.2 % |

Rust with level 2 (150 symbols, seed 1, 2026-11-14, `--verified-only --scip`; CodeSkimr from `main` at b43a378, index from rust-analyzer 1.97.0 for the same corpus commit, `level2.min_mapped` 0.5): exact 90.0 %, precision 100 % (673 of 673), recall 91.8 % (673 / 733), against 27.3 %, 59.2 % and 59.3 % at level 1. Share mapped of project uses 82.4 % (7008 mapped, 1498 unmapped, 6881 library, 6673 ignored; 147 files replaced, 84 documents of skipped folders without a file). Missing: 56 uses of `Self` (rust-analyzer names them `impl#[Type]`, which the import does not map to the type, issue #240) and 4 definitions of trait methods in `impl Trait for Type` (the `override` link of level 1 is lost, issue #241). rust-analyzer scip: 34 to 41 s, 2.1 GiB peak memory, 5 MB index, 538 MB `target/` in the project folder; it runs build scripts and proc macros of the project (the reference tool does not).

Typical error classes:

- Rust, missing: fields (recall 32 %, uses through `self.x` and struct literals with shorthand), struct and enum uses in paths and patterns (62 % and 44 %), enum variants (60 %).
- Rust, wrong: calls and reads of a field or method name that several types share (`call` 141 and `read` 139 wrong usages of 735), mostly methods (26 % of the usages of method symbols are right) and fields (23 %).
- Java, missing: constructors (recall 0 of 113 uses: `new X(...)` is not linked to the constructor), enum constants (50 %).
- Java, wrong: calls of a method name by name only (`call`: 516 of the 534 wrong usages; methods have 21 % precision, mostly `read`, `write`, `value`, `toString` that many classes define) and `override` links (18).
- Java, fields, classes and interfaces are close to the aim (recall 97 to 100 %, precision 100 %).


After the reversal of the ambiguity rule (ADR 0033, 150 symbols, seed 1, 2026-11-13): level 1 keeps every match and marks it unchecked.

| Language | exact | precision | recall |
|---|---|---|---|
| Python | 32.0 % | 32.3 % | 81.1 % |
| TypeScript | 21.3 % | 27.7 % | 60.5 % |

Recall is back at the level before PR #133 (Python 80.6 %, TypeScript 60.5 %); precision is back there too, which is why results are counted apart as unchecked.


150 symbols, seed 1, before and after PR #133 (2026-11-13). The reading and the follow-up work are in #145.

| Language | exact | precision | recall |
|---|---|---|---|
| Python before | 32.7 % | 32.7 % | 80.6 % |
| Python after | 34.7 % | 33.9 % | 80.6 % |
| TypeScript before | 21.3 % | 27.7 % | 60.5 % |
| TypeScript after | 23.3 % | 58.1 % | 41.6 % |
