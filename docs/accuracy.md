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

`--scip` also takes TypeScript and Python indexes (spike of 2026-11-14, same limits as above; `--indexer PROGRAM` names the program for the version check, pins in `reference_tools.toml`, `[typescript_scip]` and `[python_scip]`). By hand, in a throw-away copy of the corpus checkout (never in the repository):

```
# TypeScript: scip-typescript 0.6.2, the command of languages/typescript/language.toml
npm ci --ignore-scripts && scip-typescript index --output index.scip --infer-tsconfig
python3 tools/accuracy/measure.py typescript --verified-only --scip index.scip --allow-uncovered 2
# Python: scip-python 0.6.8, in a clean virtual environment (see below)
scip-python index --project-version 0 --output index.scip
python3 tools/accuracy/measure.py python --verified-only --scip index.scip
```

`--allow-uncovered N` (not negative) lets the index leave out up to N scanned files and names them in a warning; an index that covers more files than were scanned is always refused: with a `tsconfig.json` scip-typescript indexes only what it includes (here `src` and `test`), so `vite.config.ts` and `playwright.config.ts` of the corpus have no document and keep level 1 (their references are not counted in `--verified-only`). Without a `tsconfig.json` and with `--infer-tsconfig` all 64 files are covered and the numbers are the same. The Python result depends on the environment of the indexer: the corpus is click, and this container has another click (8.5.0) installed, so `from click.shell_completion import ...` inside the sources resolved to the installed copy and 7 uses of `CompletionItem` became library uses; in a clean virtual environment (`python3 -m venv`, then activate before `scip-python`) the recall is 94.6 % instead of 92.6 %. The numbers below are for the clean environment. Reference and indexer share an engine for TypeScript as well (scip-typescript uses the TypeScript compiler API, as the language service does); for Python they are independent (Pyright against Jedi).

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
- The level 1 numbers below are the baseline. The level 2 numbers (Rust, TypeScript, Python) come from a hand-made index (see `--scip`), not from the scan; the level 2 pass is not measured through the scan yet.

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


TypeScript and Python with level 2 (150 symbols, seed 1, 2026-11-14, `--verified-only --scip`; CodeSkimr at b43a378, `level2.min_mapped` 0.5; indexers scip-typescript 0.6.2 and scip-python 0.6.8, Node 22; references TypeScript language service 5.6.3 and Jedi 0.20.1). Corpora as pinned in `corpus/<language>/corpus.toml`.

| Language | level | exact | precision | recall | usages ours / tool |
|---|---|---|---|---|---|
| TypeScript | 1 | 21.3 % | 27.7 % | 60.5 % | 1513 / 693 |
| TypeScript | 2 | 82.7 % | 99.3 % (576 of 580) | 83.1 % (576 of 693) | 580 / 693 |
| Python | 1 | 32.0 % | 32.3 % | 81.1 % | 880 / 350 |
| Python | 2 | 66.7 % | 61.3 % (331 of 540) | 94.6 % (331 of 350) | 540 / 350 |

Share mapped of project uses: TypeScript 87.6 % (8679 mapped, 1228 unmapped, 6678 library, 11695 ignored; 62 files replaced, 51 documents of skipped folders, 2 scanned files without a document), Python 86.0 % (2578 mapped, 418 unmapped, 3770 library, 7325 ignored; 17 files replaced, 74 documents of skipped folders). Time and memory of the indexer (one cold run, whole checkout including the skipped tests): scip-typescript 3.7 to 5.7 s, 404 MiB peak, 5.4 MB index (with `npm ci` 2.3 s first); scip-python 13 s in a clean environment (22 s with the full site-packages), 528 to 589 MiB, 4.6 MB index; scip-python 0.6.0 (the minimum of the pack) needs 46 s.

- TypeScript, missing (117 = 693 - 576): all 114 missing uses of 24 symbols of the form `const f = (...) => ...` (110 uses in code and 4 import lines of these symbols), whose span starts after the name (issue #247; level 1 found 39 of their 114 uses, level 2 finds none), and 3 uses of a getter (`<get>size`, issue #248). The 4 import lines are what the language service counts and we do not (issue #250). If #247 and #248 are fixed, 689 of 693 uses are found, a recall of about 99.4 % (arithmetic on the measured numbers, not a new measurement).
- TypeScript, wrong: 4 uses of `RoomNode.id` through a variable of the union type `FloorNode | RoomNode`, which scip-typescript counts and the language service does not; the index is right there.
- Python, missing: 14 uses of functions with `@overload` (scip-python has one symbol for all overloads, the definition is the first one; issue #249), 3 calls resolved by Pyright to the subclass or the declared type of the receiver (`obj.shell_complete`, `super().shell_complete`) where Jedi names the base method.
- Python, wrong (209 lines): 165 on symbols for which Jedi finds no use at all (instance attributes `self.x` and annotations; 14 of 14 lines checked by eye are true uses), 27 on import lines (issue #250), 11 heads of overloads (issue #249), 6 other. Without the lines of symbols for which Jedi has nothing and without import lines the precision is 95.1 %; read the 61.3 % as a lower bound, as for level 1.
- UTF-16 columns: scip-typescript marks every document UTF-16, scip-python leaves the encoding unspecified but writes UTF-16 columns (the import reads `Unspecified` as UTF-16, which is right). 231 occurrences of the TypeScript corpus stand behind a non-ASCII character; a one-line test with a surrogate pair (`"é𝒳"; greet()`) maps to byte column 44 from UTF-16 column 41, as it should.
- TypeScript monorepo: a root `tsconfig.json` with `"files": []` and `references` (solution style) indexes all referenced projects, paths relative to the root, with or without `--yarn-workspaces`. Without `node_modules` the project symbols map the same (library uses drop from 6678 to 5730).
- Indexer command of the packs: TypeScript `index --output {output} --infer-tsconfig` is right. scip-typescript below 0.3.11 bundles TypeScript 4 and stops on `"moduleResolution": "Bundler"` (`error TS6046`), so the `min_version` 0.3.0 does not hold for current projects (issue #209). Python `index {project} --output {output}`: scip-python has no positional argument, `{project}` is ignored and the folder indexed is the working folder (the runner sets it to the project root, so it works by luck); outside a git repository (or without a commit) scip-python writes an empty index and exits with 1 unless `--project-version` is given.

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
