# Accuracy of usages per language (issue #134)

`tools/accuracy/measure.py` measures how well `find_usages` agrees with the reference tooling of a language on the pinned public corpora (`corpus/<language>/corpus.toml`). It is a measurement, not a CI gate.

```
python3 tools/accuracy/measure.py python --samples 150 --seed 1 --codeskimr target/debug/codeskimr
python3 tools/accuracy/measure.py typescript --samples 150 --seed 1
python3 tools/accuracy/measure.py rust --samples 150 --seed 1
python3 tools/accuracy/measure.py java --samples 150 --seed 1
python3 tools/accuracy/measure.py csharp --samples 150 --seed 1 --workspace DIR
```

`--json` adds the numbers per symbol kind, the wrong usages grouped by our reference kind, and up to five wrong and five missed lines per symbol. `--verified-only` counts only references with `verified = 1` (level 2); this is the measure of ADR 0033, decision 14, once the level 2 pass is wired into the scan. Without it, level 1 and level 2 references are counted together.

Steps: the script copies the corpus sources (without the skipped folders), scans them with `codeskimr scan`, draws symbols with a fixed seed from the list sorted by qualified name (so two versions see the same symbols; symbol kinds that can be used, names starting with an underscore are left out, which also leaves out all Python constructors), asks the reference tool where each symbol is used, and compares by file and line.

| Language | Reference tool | Needs |
|---|---|---|
| Python | Jedi `get_references` | `pip install jedi` |
| TypeScript | TypeScript language service `findReferences` | `typescript` (set `TYPESCRIPT_MODULE` and `NODE_PATH` if it is not installed locally) |
| Rust | rust-analyzer over LSP `textDocument/references` (`truth_rust.py`) | `rust-analyzer`, `cargo` and network access for the crates of the corpus project (rust-analyzer loads the full checkout; build scripts and proc macros are off) |
| C# | Roslyn (`tools/accuracy/truth_csharp`, run with `dotnet run`): opens the restored solution with `MSBuildWorkspace`, every simple name and every `new` that the compiler resolves to a declaration is a use | .NET SDK 8 or newer, network access to NuGet, and `--workspace DIR`: a copy of the corpus checkout in which both projects are retargeted to the SDK of the container (see below) |
| Java | `javac` itself (`TruthJava.java`, run with `java TruthJava.java`): parse and attribute all sources with `JavacTask`, every identifier, member select, member reference and `new` that the compiler resolves to a declaration is a use | JDK 17 or newer; the pinned compile-time dependencies and the template substitution of `tools/accuracy/reference_tools.toml` (one jar, SHA-256 checked, cached in `target/accuracy-cache`) |

`--scip INDEX` measures level 2 before the scan wires it in (and without the consent of the owner, which is not touched): the index is made by hand with `rust-analyzer scip <checkout> --output index.scip`, `tools/accuracy/import_scip` (a standalone Cargo project, build it with `cargo build --manifest-path tools/accuracy/import_scip/Cargo.toml --target-dir target`) maps it with the public `codeskimr_scip` library onto the scanned database, and the comparison runs on that database; `--scip` needs `--verified-only` (the script refuses otherwise), warns if `rust-analyzer --version` is not the pinned one (`reference_tools.toml`, `[rust_scip]`: `rust-analyzer 1.97.0 (2d8144b 2026-07-07)`) and stops if the index does not cover every scanned file, which is how a foreign index (other corpus commit) shows. The index must come from the checkout of the commit in `corpus/rust/corpus.toml`. The tool calls `convert_columns` like the product (the index of rust-analyzer is UTF-8, so it changes nothing here) and reads the limits from `config/default.toml`. The tool writes only into the throw-away database of the measurement. `--import-scip` names the tool. **What this does not measure:** the `IndexRunner` (call, arguments, limits, process group, environment), the consent and the wiring into the scan are not exercised, because the index is made by hand. The numbers are therefore not an acceptance under ADR 0033 decision 14 (threshold 99 %, here 91.8 %); they measure import and mapping only.

`--scip` also takes TypeScript and Python indexes (spike of 2026-11-14, same limits as above; `--indexer PROGRAM` names the program for the version check; `scip-typescript --version` and `scip-python --version` print the bare number, which is what the pins hold; pins in `reference_tools.toml`, `[typescript_scip]` and `[python_scip]`, also for the reference tools TypeScript 5.6.3, Node 22 and Jedi 0.20.1). By hand, in a throw-away copy of the corpus checkout (never in the repository):

```
# TypeScript: scip-typescript 0.6.2, the command of languages/typescript/language.toml
npm ci --ignore-scripts && scip-typescript index --output index.scip --infer-tsconfig
python3 tools/accuracy/measure.py typescript --verified-only --scip index.scip --allow-uncovered 2
# Python: scip-python 0.6.8, in a clean virtual environment (see below)
scip-python index --project-version 0 --output index.scip
python3 tools/accuracy/measure.py python --verified-only --scip index.scip
```

`--allow-uncovered N` (not negative) lets the index leave out up to N scanned files and names them in a warning; an index that covers more files than were scanned is always refused: with a `tsconfig.json` scip-typescript indexes only what it includes (here `src` and `test`), so `vite.config.ts` and `playwright.config.ts` of the corpus have no document and keep level 1 (their references are not counted in `--verified-only`). Without a `tsconfig.json` and with `--infer-tsconfig` all 64 files are covered and the numbers are the same. The Python result depends on the environment of the indexer: the corpus is click, and this container has another click (8.5.0) installed, so `from click.shell_completion import ...` inside the sources resolved to the installed copy and 7 uses of `CompletionItem` became library uses; in a clean virtual environment (`python3 -m venv`, then activate before `scip-python`) the recall on `main` was 94.6 % instead of 92.6 %. The numbers below are for the clean environment. Reference and indexer share an engine for TypeScript as well (scip-typescript uses the TypeScript compiler API, as the language service does); for Python they are independent (Pyright against Jedi).

`--scip` also takes Java and C# indexes (2026-11-14, same limits as above; pins in `reference_tools.toml`, `[java_scip]` and `[csharp_scip]`; `--indexer PROGRAM` names the program for the version check; `--workspace DIR` names the checkout prepared for the reference tool, which for C# is the restored copy the index was made in). By hand, in a throw-away copy of the corpus checkout (never in the repository):

```
# COPY is the throw-away copy of the corpus checkout, REPO the checkout of CodeSkimr; the measuring command runs in REPO.
# Java: scip-java 0.12.3 (JDK 21, Maven). The JDK warns that `release 8` is obsolete and the pom asks for failOnWarning:
# set `<failOnWarning>` of the root pom.xml in COPY to false, then (extra arguments replace the goals of the build)
(cd COPY && scip-java index --output index.scip -- -pl gson clean compile)
python3 REPO/tools/accuracy/measure.py java --verified-only --scip COPY/index.scip --indexer scip-java --allow-uncovered 1
# C#: scip-dotnet 0.2.16 (`dotnet tool install scip-dotnet`), .NET SDK 8. The projects ask for net10.0 and net11.0, which the
# SDK of the container cannot build: set `<TargetFrameworks>` of both projects in COPY to net8.0 (sources unchanged)
(cd COPY && scip-dotnet index --working-directory . --output index.scip)
python3 REPO/tools/accuracy/measure.py csharp --verified-only --scip COPY/index.scip --workspace COPY
```

**These tools run foreign code.** `scip-java` runs the Maven or Gradle build of the project, `scip-dotnet` and the C# reference tool (`MSBuildWorkspace`) evaluate its MSBuild files (props, targets, analyzers, source generators), and the build tools fetch and run plugins and packages. That is the code of the corpus, which is pinned and public, but nothing here may be pointed at a project one does not trust. Run the measurement in a container or a virtual machine without credentials, tokens, a mounted home folder or access to private networks. The reference tools run in the throw-away folder of the measurement and never in the checkout (the `dotnet` host reads a foreign `global.json` of its working folder), and `truth_csharp` restores its NuGet packages in locked mode from `packages.lock.json` (content hashes, the transitive `System.Text.Json` and `System.Drawing.Common` are raised to fixed versions); the SHA-256 pins of `reference_tools.toml` cover the Java downloads only.

`--allow-uncovered 1` for Java is `module-info.java`, which scip-java does not index (it stays at level 1; its references are not counted). For C# the index covers all 55 scanned files and the option is not needed; 4 documents of the index belong to no scanned file (generated sources). Both reference tools key a declaration by file, name and category (type, method, constructor, member) within eight lines of the line of the target, and a declaration counts as found whether or not anything uses it: before 2026-11-14 `TruthJava` keyed by file, line and name only, which let a method take the uses of its own parameter of the same name (`value(long value)`), a constructor those of its class and an unused overload those of its neighbour. That had inflated the tool uses of the Java level 1 baseline: they fall from 590 to 556 (see the table below).

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
- Covered: Python, TypeScript, Rust, Java and C#. Kotlin follows.
- Rust: rust-analyzer runs without build scripts and proc macros, so code that a macro generates has no references; `pub use` re-exports count as uses for rust-analyzer but not for us (we leave imports out). Uses in the skipped folders (tests, benches) are dropped from the reference, because we did not scan them.
- Java: `module-info.java` is not compiled, the language level is 17, and generated sources (here `GsonBuildConfig`) are compiled but never reported. The compiler tells overloads apart; we do not. A method that overrides another has no uses of its own for the compiler, while we also attach calls through the base type to every override (dispatch, `probable`).
- C#: the sources are compiled for `net8.0` (the container has only that SDK; the projects ask for `net10.0` and `net11.0`), and the reference and the indexer see the same retargeted copy. A use through a conditional-compilation branch that `net8.0` does not take is in neither. `var` is not counted as a use of the type it stands for. Roslyn reports the explicit implementation `IEnumerable.GetEnumerator` under that name; the tool keys it by the part behind the last dot.
- The level 1 numbers below are the baseline. The level 2 numbers (Rust, TypeScript, Python) come from a hand-made index (see `--scip`), not from the scan; the level 2 pass is not measured through the scan yet.

## Results

First measurement for Rust and Java (150 symbols, seed 1, 2026-11-14, level 1, all references; CodeSkimr built from `main` at 8c83933). Corpora: Rust = CodeSkimr at `bcc970e90cb99c2d052457f79090a78194dfaa8e`, Java = gson at `a961b0fe088f8558d00cc1b56e506a3656907a91`. Tools: rust-analyzer 1.97.0 (2d8144b 2026-07-07), JDK 21.0.12 (`javac` 21.0.12.1, compiled with `--release 17`):

| Language | symbols | usages ours / tool | exact | precision ("correct") | recall |
|---|---|---|---|---|---|
| Rust (CodeSkimr at the pinned corpus commit) | 150 | 735 / 733 | 27.3 % | 59.2 % | 59.3 % |
| Java (gson at the pinned corpus commit) | 150 | 1001 / 556 | 55.3 % | 46.7 % | 84.0 % |

The Java row was measured first with 590 tool usages (exact 54.0 %, recall 79.2 %); the reference script has been corrected since (see `--scip` above: it took the uses of a parameter, of a class and of a neighbouring overload for those of a method, a constructor and an unused overload), which removes 34 usages and leaves our 467 correct ones unchanged.

Rust with level 2 (150 symbols, seed 1, 2026-11-14, `--verified-only --scip`; CodeSkimr from `main` at b43a378, index from rust-analyzer 1.97.0 for the same corpus commit, `level2.min_mapped` 0.5): exact 90.0 %, precision 100 % (673 of 673), recall 91.8 % (673 / 733), against 27.3 %, 59.2 % and 59.3 % at level 1. Share mapped of project uses 82.4 % (7008 mapped, 1498 unmapped, 6881 library, 6673 ignored; 147 files replaced, 84 documents of skipped folders without a file). Missing: 56 uses of `Self` (rust-analyzer names them `impl#[Type]`, which the import does not map to the type, issue #240) and 4 definitions of trait methods in `impl Trait for Type` (the `override` link of level 1 is lost, issue #241). rust-analyzer scip: 34 to 41 s, 2.1 GiB peak memory, 5 MB index, 538 MB `target/` in the project folder; it runs build scripts and proc macros of the project (the reference tool does not).


After the fixes for `Self` and for the methods of trait implementations (issues #240 and #241, `docs/level2-import.md` points 8 and 9; same corpus, tool and command, 2026-11-14): seed 1, 150 symbols: exact 150 of 150, precision 100 % (733 of 733), recall 100 % (733 / 733). Other draws as a check: seed 2 with 400 symbols precision 99.6 %, recall 100 %; seed 3 with 300 symbols precision 99.7 %, recall 99.9 %. The remaining wrong uses are in `codes` (a generated table) and a few enum members, which rust-analyzer's own references do not list. Two limits of the figure: the reference (`rust-analyzer` references) and the index (`rust-analyzer scip`) come from the same engine, so a mistake of rust-analyzer does not show; and the corpus (CodeSkimr itself) holds few cases of a library type that shares its name with a project type, which the import deliberately leaves unmapped.

Typical error classes:

- Rust, missing: fields (recall 32 %, uses through `self.x` and struct literals with shorthand), struct and enum uses in paths and patterns (62 % and 44 %), enum variants (60 %).
- Rust, wrong: calls and reads of a field or method name that several types share (`call` 141 and `read` 139 wrong usages of 735), mostly methods (26 % of the usages of method symbols are right) and fields (23 %).
- Java, missing: constructors (recall 0 of 83 uses: `new X(...)` is not linked to the constructor), enum constants (50 %).
- Java, wrong: calls of a method name by name only (`call`: 516 of the 534 wrong usages; methods have 21 % precision, mostly `read`, `write`, `value`, `toString` that many classes define) and `override` links (18).
- Java, fields, classes and interfaces are close to the aim (recall 97 to 100 %, precision 100 %).


TypeScript and Python with level 2 (150 symbols, seed 1, 2026-11-14, `--verified-only --scip`; CodeSkimr at b43a378, `level2.min_mapped` 0.5; indexers scip-typescript 0.6.2 and scip-python 0.6.8, Node 22; references TypeScript language service 5.6.3 and Jedi 0.20.1). Corpora as pinned in `corpus/<language>/corpus.toml`.

| Language | level | exact | precision | recall | usages ours / tool |
|---|---|---|---|---|---|
| TypeScript | 1 | 21.3 % | 27.7 % | 62.7 % | 1513 / 668 |
| TypeScript | 2 | 99.3 % | 99.4 % (668 of 672) | 100 % (668 of 668) | 672 / 668 |
| Python | 1 | 32.0 % | 32.3 % | 81.1 % | 880 / 350 |
| Python | 2 | 69.3 % | 63.3 % (347 of 548) | 99.1 % (347 of 350) | 548 / 350 |

Share mapped of project uses: TypeScript 93.1 % (9565 mapped, 705 unmapped, 6678 library, 11332 ignored; 62 files replaced, 51 documents of skipped folders, 2 scanned files without a document), Python 85.8 % (2533 mapped, 418 unmapped, 3770 library, 7370 ignored; 17 files replaced, 74 documents of skipped folders). Time and memory of the indexer (one cold run, whole checkout including the skipped tests): scip-typescript 3.7 to 5.7 s, 404 MiB peak, 5.4 MB index (with `npm ci` 2.3 s first); scip-python 13 s in a clean environment (22 s with the full site-packages), 528 to 589 MiB, 4.6 MB index; scip-python 0.6.0 (the minimum of the pack) needs 46 s.

- TypeScript, missing: nothing in the sample. The earlier measurement (576 of 693, issues #247 and #248) lost 24 functions of the form `const f = (...) => ...` (14 of them local to another function, with a `local N` symbol) and the getter `size`; they are mapped now (`docs/level2-import.md`, points 10 to 12). A local function is mapped only if the text of its definition reads exactly like the function's name (the import reads that text in `convert_columns`, in bytes); the first version compared only the length of the name, which could have mapped uses of `key` to a function `has` on the same line, and the measurement is the same with the exact comparison (668 of 668). The reference script itself had to be corrected: it looked for the name behind the start of the symbol, so for these functions it asked the language service about another identifier on the line (`hasAction` for `has`) and counted its uses (693 uses before, 668 now; the 4 import lines of the earlier figure belong to that count, issue #250 stays open and untouched). The script now finds the name in the syntax tree (also for `const h: (a: number) => void = ...`) and stops with an error where it cannot, instead of falling back silently.
- TypeScript, wrong: 4 uses of `RoomNode.id` through a variable of the union type `FloorNode | RoomNode`, which scip-typescript counts and the language service does not; the index is right there.
- Python, missing (3 = 350 - 347): 2 calls resolved by Pyright to the subclass or the declared type of the receiver (`obj.shell_complete`, `super().shell_complete`) and 1 of `invoke`, where Jedi names the base method. The 14 uses of functions with `@overload` of the earlier measurement are found now (issue #249). Same index, `main` at 79619e7 against this branch: recall 94.6 % (331) before, 99.1 % (347) now, so 16 more; no symbol lost a use. An earlier figure of 340 (97.1 %) came from an index made in an environment with another click installed: 7 uses of `CompletionItem` pointed into the installed library (see above), main and branch alike. The measurement above is from a clean virtual environment.
- Python, wrong (201 = 548 - 347 lines), accounted for by symbol: 161 on 27 fields for which Jedi finds no use at all (instance attributes `self.x`, annotations; lines checked by eye in the earlier measurement were true uses), 21 on 4 classes and 15 on 11 functions where Jedi finds some (among the 132 lines `--json` lists, 25 are import lines, issue #250, and the rest were not examined one by one), 2 on one class without any Jedi use and 2 on one method. The heads of overloads (11 before) are no uses any more. Read the 63.3 % as a lower bound, as for level 1.
- UTF-16 columns: scip-typescript marks every document UTF-16, scip-python leaves the encoding unspecified but writes UTF-16 columns (the import reads `Unspecified` as UTF-16, which is right). 231 occurrences of the TypeScript corpus stand behind a non-ASCII character; a one-line test with a surrogate pair (`"é𝒳"; greet()`) maps to byte column 44 from UTF-16 column 41, as it should.
- TypeScript monorepo: a root `tsconfig.json` with `"files": []` and `references` (solution style) indexes all referenced projects, paths relative to the root, with or without `--yarn-workspaces`. Without `node_modules` the project symbols map the same (library uses drop from 6678 to 5730).
- Indexer command of the packs: TypeScript `index --output {output} --infer-tsconfig` is right. scip-typescript below 0.3.11 bundles TypeScript 4 and stops on `"moduleResolution": "Bundler"` (`error TS6046`), so the `min_version` 0.3.0 does not hold for current projects (issue #209). Python `index {project} --output {output}`: scip-python has no positional argument, `{project}` is ignored and the folder indexed is the working folder (the runner sets it to the project root, so it works by luck); outside a git repository (or without a commit) scip-python writes an empty index and exits with 1 unless `--project-version` is given.

Java and C# with level 2 (150 symbols, seed 1, 2026-11-14, `--verified-only --scip`; CodeSkimr from this branch (on `main` at e150798), `level2.min_mapped` 0.5; indexers scip-java 0.12.3 (JDK 21.0.12, Maven) and scip-dotnet 0.2.16 (.NET SDK 8.0.134); references javac 21.0.12 (`--release 17`) and Roslyn 4.8.0). Corpora as pinned in `corpus/<language>/corpus.toml`.

| Language | level | exact | precision | recall | usages ours / tool |
|---|---|---|---|---|---|
| Java | 1 | 55.3 % | 46.7 % (467 of 1001) | 84.0 % (467 of 556) | 1001 / 556 |
| Java | 2 | 93.3 % | 90.9 % (489 of 538) | 87.9 % (489 of 556) | 538 / 556 |
| C# | 1 | 88.0 % | 28.8 % | 98.0 % | 677 / 199 |
| C# | 2 | 94.0 % | 60.2 % (68 of 113) | 34.2 % (68 of 199) | 113 / 199 |

Share mapped of project uses: Java 99.3 % (5806 mapped, 41 unmapped, 5230 library, 7580 ignored; 86 files replaced, 1 document of the index without a scanned file: the generated `GsonBuildConfig`; the 41 unmapped were not examined), C# 100 % (2142 mapped, 0 unmapped, 4616 library, 2771 ignored; 55 files replaced, 4 documents without a scanned file). Indexer, one cold run each, the Maven dependencies and the NuGet packages already in the cache: scip-java 7.3 s, 270 MiB peak, 2.6 MB index (the index is not byte-identical between two runs, the numbers are the same); scip-dotnet 12.3 s (23 s the first time, with `dotnet restore`), 298 MiB, 1.4 MB index.

- Java, wrong (49 of 538): every one is an import line (`import com.google.gson.Gson;`, issue #250, untouched). Without them the precision is 100 % (489 of 489).
- Java, missing (67 = 556 - 489): 67 uses of constructors and none of anything else. 58 are the implicit `super()` call in the default constructor of a subclass (javac counts it in the line of the class) and 9 are anonymous subclasses (`new IntegerFieldsTypeAdapter<Duration>(...) { ... }`); the index has only the type symbol there (issue #259). Not fixable in the import. Classes: recall 100 % (101 of 101), interfaces 100 %, fields 100 %, methods 100 % (140 of 140), enum constants 100 %.
- Java, fixed in this branch: `new Gson(...)` stands in the index as a use of the constructor `Gson#<init>().` and not of the class; the class had 0 of 3 uses of `WildcardTypeImpl` and 87 of 101 over all classes, and has 101 of 101 now. The import adds the use of the type where level 1 reads the type name at the same place (`docs/level2-import.md` point 13). `this(...)` and `super(...)` stand on a keyword, level 1 has no type use there, and no use is added.
- C#, wrong (45 of 113) and missing (131 = 199 - 68): 45 and 130 of them come from 8 sampled symbols, overloaded extension methods that are called as `Guard.Against.Negative(0L)`. scip-dotnet 0.2.16 writes the symbol of the **first** overload for every such call (the static form `Guard.Negative(g, 0L)` is right), so the first overload gets the uses of all and the others none (issue #258; the reproduction is there). On the other 142 symbols 141 are exactly equal (59 of 59 uses ours, 59 of 60 of the tool): the one miss is `new Guard()` of a private constructor, which scip-dotnet does not write as a use of the constructor (issue #259).
- C#, fixed in this branch: the rule of point 12 (overload heads) took `GetEnumerator()` in `IEnumerator IEnumerable.GetEnumerator() => GetEnumerator();` for the head of the explicit implementation on the same line, so 8 symbols lost a use (`docs/level2-import.md` point 12). A function that is already the target of a symbol is no head. The measurement before the fix: precision 57.1 %, recall 30.2 %, exact 88.7 %.
- Engine of reference and index: for Java both are javac (the reference reads the trees with `com.sun.source`, the index comes from the semanticdb-javac plugin in the compiler; release 17 against release 8 and the classpath of the Maven build against the pinned jar), for C# both are Roslyn (the reference through `MSBuildWorkspace`, the index through scip-dotnet's own workspace and symbol naming). A mistake of the compiler itself therefore does not show; mistakes of the indexer's naming and writing do, as the overloads of C# show. The reference and the index do not read each other.
- C# level 1 has the better recall here (98.0 % against 34.2 %) because it matches by name; level 2 is more exact than level 1 (precision 60.2 % against 28.8 %) but far below the threshold, and its 45 wrong uses all come from the indexer fault of #258. The exact rates (Java 93.3 %, C# 94.0 %) are carried by symbols without uses: on the 142 C# symbols that the fault does not touch there are only 60 uses in all. With the overload fault of scip-dotnet the product would lose uses of every overloaded extension method; that is a fact of this indexer version, see issue #258.
- **Not an acceptance under ADR 0033, decision 14 (threshold 99 % for precision and recall)**, as for Rust: the index is made by hand, so the `IndexRunner` (call, arguments, limits, environment), the consent and the wiring into the scan are not measured, and neither language reaches the threshold (Java 90.9 % and 87.9 %, C# 60.2 % and 34.2 %); they measure the import and the mapping and the indexers only.
- The C# reference tool uses Roslyn 4.8.0, which understands C# up to version 12; the corpus targets net10.0 and net11.0, so a construct of a newer language version could be missing from the reference (and from nothing else). Whether the corpus holds one was not checked.

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
