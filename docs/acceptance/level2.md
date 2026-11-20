# Acceptance report level 2 (ADR 0033, decision 14)

Measured on 2026-11-15 through the real way: `codeskimr level2 confirm` (the project name typed at the prompt of a pseudo terminal) and then `codeskimr scan` with `level2.enabled=true`, so the `IndexRunner` (call, arguments, limits, process group, environment), the consent and the wiring into the scan are exercised. No index is made by hand and nothing reads one: `tools/accuracy/measure.py LANGUAGE --level2-run` (helper `tools/accuracy/level2_run.py`) copies the pinned corpus checkout into a throw-away folder, prepares it, scans it with the skipped folders of the corpus as `scan.excluded_folders` (the scan sees the same files as the filtered copy that the reference tools read, the indexer sees the whole project), and compares the references with `verified = 1` with the reference tool. The reference tools are those of `docs/accuracy.md` and do not read the index (javac, TypeScript language service, Jedi, rust-analyzer LSP, Roslyn).

**Binary:** `main` at 6a5ffe1 plus PR #396 (without it the memory limit stops rust-analyzer, scip-dotnet and the version probe of scip-java, issue #392). **Machine:** Linux container, 4 vCPU, 16 GB. **Indexers and tools:** rust-analyzer 1.97.0, scip-typescript 0.6.2 (Node 22, TypeScript 5.6.3), scip-python 0.6.8 (clean virtual environment outside the project, Jedi 0.20.1), scip-java 0.12.3 (JDK 21.0.12, Maven), scip-dotnet 0.2.16 (.NET SDK 8.0.134, Roslyn 4.8.0). Default limits (`level2.timeout_seconds` 900, `max_memory_mb` 4096). Corpora as pinned in `corpus/<language>/corpus.toml`.

Legend: met = precision and recall of the sample at least 99 % (threshold of decision 14); not met = below, with the cause.

## Result per language

Seed 1, 150 symbols (the draw of the earlier hand-made measurements) and seed 2, 300 symbols (a second draw as a check).

| Language | Seed / symbols | exact | precision | recall | usages ours / tool | Level 2 scan | Verdict |
|---|---|---|---|---|---|---|---|
| Rust | 1 / 150 | 149 | 100 % | 99.9 % | 747 / 748 | 22 s | met |
| Rust | 2 / 300 | 295 | 99.1 % | 100 % | 967 / 958 | 18 s | met |
| TypeScript | 1 / 150 | 118 | 99.4 % | 91.8 % | 617 / 668 | 12 s | precision met; recall below by import lines (see below) |
| TypeScript | 2 / 300 | 230 | 99.4 % | 91.5 % | 1391 / 1511 | 12 s | precision met; recall below by import lines (see below) |
| Python | 1 / 150 | 117 | 67.2 % | 99.1 % | 516 / 350 | 11 s | not met (precision; see below) |
| Python | 2 / 300 | 219 | 58.2 % | 99.6 % | 971 / 567 | 11 s | not met (precision) |
| Java | 1 / 150 | 150 | 100 % | 100 % | 556 / 556 | 37 s | met |
| Java | 2 / 300 | 299 | 100 % | 99.8 % | 1096 / 1098 | 32 s | met |
| C# | 1 / 150 | 141 | 60.2 % | 34.2 % | 113 / 199 | 15 s | not met (indexer, #258) |
| C# | 2 / 300 | 287 | 93.3 % | 84.3 % | 927 / 1026 | 15 s | not met (indexer, #258) |
| Kotlin | | | | | | | **not measured** |

**TypeScript and Python were measured again on 2026-11-20** (same way, `main` at 7402cc3 plus #594, scip-typescript 0.6.2, scip-python 0.6.8, same corpus commits) after issue #250: import names are no uses any more, so we have 55 and 32 usages fewer than on 2026-11-15 (TypeScript 617 instead of 672, Python 516 instead of 548 on seed 1). Python precision rises from 63.3 % to 67.2 % (seed 2: 55.6 % to 58.2 %), recall stays at 99.1 % and 99.6 %. TypeScript precision stays at 99.4 %, and recall falls to 91.8 % and 91.5 % because the language service counts the import line of a symbol as a reference and we do not: all 55 missing usages of seed 1 (668 minus 613) are import lines, and 121 of the 128 of seed 2 (1511 minus 1383; the other 7 are #394); each lies on its own line, checked against the source with the 5-per-symbol limit of `--json` lifted. See `docs/accuracy.md`. The verdict for TypeScript is therefore not "met" by the column; whether import lines count on the side of the tool is open.

The seed 1 figures of the rows Rust, Java and C# equal, usage for usage, the hand-made measurements of `docs/accuracy.md` (that was also true for TypeScript and Python on 2026-11-15, before #250): Java (556 of 556) and C# (68 of 113 there), so for these the runner, the consent and the wiring add nothing and lose nothing compared with the import alone. **Rust differs:** `docs/accuracy.md` has 733 of 733 usages (hand-made, 2026-11-14, `main` at b43a378 plus the fixes #240 and #241), this run has 747 ours and 748 of the tool. The two are not the same sample: the symbols are drawn from the symbols of level 1, and the Rust pack changed in between (for example #377), so the same seed draws other symbols and the usage counts differ. This is the likely cause, **not verified** (the hand-made run was not repeated with the old binary). Both runs are at or above 99 %; the hand-made Rust figure is not wrong for its binary, it is not comparable one to one.

## Deviations, classified

| Deviation | Cause | Handling |
|---|---|---|
| **Memory limit stops rust-analyzer and scip-dotnet, version probe of scip-java fails** (found by this run, every real run on Linux with default limits) | Own error: `TreeMemory` counted every thread of a process as a process with the memory of the whole process (rust-analyzer 14.7 GiB measured against 1.3 GiB real) | Issue #392, fix and test in PR #396. The numbers above are with the fix. |
| TypeScript: `powerReader` and `energyReader` 0 of 7 and 0 of 4 (seed 2) | Own error: the import maps a function that starts behind its name only when it starts on the same line; a curried arrow function on the next line (`export const f =` newline `(x) =>`) is not mapped | Issue #394 |
| TypeScript: 4 uses of `RoomNode.id` (and `color`, `name`, `watts` through the union `FloorNode \| RoomNode`) | The index is right: scip-typescript counts the union, the language service does not (known, see below) | No action |
| Python precision 67.2 % / 58.2 % (2026-11-20; 63.3 % / 55.6 % before #250): 135 `read`, 25 `write`, 6 `type_use`, 3 `call` wrong on seed 1 | Reference gap: Jedi is silent for 28 (seed 2: 67) symbols, all instance attributes (`self.x`), on which we have 163 (371) uses; where Jedi answers the precision is 90.1 % (87.6 %). Lines checked by eye earlier were true uses. The remaining wrong ones: annotations and import lines (issue #250) | Read the figure as a lower bound. The 99 % threshold cannot be shown for Python with Jedi; an independent second reference (Pyright itself is the indexer) is missing. Recall is 99.1 % and 99.6 %. |
| C# overloaded extension methods (`Guard.Against.Negative(0L)`): first overload gets all uses | Indexer: scip-dotnet 0.2.16 | Issue #258, upstream draft in `docs/accuracy.md` |
| C# `FooGuard.Foo` 0 of 3, local function `IsNullableType` 0 of 4 | Not traced to the cause | Issue #395 (probably #258 and a local symbol) |
| Rust: `Entry::id`, `retryable`, `message_key`, `ErrorCode::message_key` wrong (the generated table `codes`) | Reference gap: rust-analyzer's references do not list uses in generated code | Known, see `docs/accuracy.md` |
| Rust: `MetricRole` 12 against 7 | Reference gap: uses through `use ... MetricRole as Role` are true uses that the LSP does not list | No action |
| Rust: `Flow::run` 4 of 5 (seed 1) | Not traced | Issue #395 |
| Java: `ToNumberPolicy.LONG_OR_DOUBLE.parseAsDouble` 0 of 2 (seed 2) | Probably #366 (members of enum constant bodies are local symbols) | Issue #395 |

## Gaps

- **Kotlin not measured.** The Kotlin pack, the corpus (KotlinPoet) and scip-java exist, but there is no reference tool for Kotlin (no `kotlinc` in the container, no compiler API script like `TruthJava`), so there is nothing independent to compare with. Not estimated.
- **Share of uses that could not be mapped** (`unmapped / (mapped + unmapped)`) is not reported by the scan, so the real way does not give it; the hand-made figures of `docs/accuracy.md` stand (Rust 82.4 % mapped before the fixes, TypeScript 93.1 %, Python 85.8 %, Java 99.3 %, C# 100 %). Uses the tool finds and we do not are the recall column.
- **Rust: reference and indexer are the same engine** (rust-analyzer), see `docs/accuracy.md`; errors of rust-analyzer do not show. Reference and indexer of TypeScript share the compiler, those of Java and C# are both javac and Roslyn respectively.
- **Corpus preparation** (what a user does before level 2): Java: `failOnWarning` off and only the `gson` module in the root `pom.xml` (the module `test-shrinker` needs R8 from a repository that the container cannot reach); C#: both projects retargeted to `net8.0`; TypeScript: `npm ci --ignore-scripts`; Python: clean virtual environment. The runner itself has no way to pass arguments to an indexer, so a multi-module Maven project that cannot be built completely cannot be indexed through it.
- One machine, one cold run per language. Memory peaks of the indexers were not recorded in this run (the runner does not report them); the hand-made figures stand.
- Sample size: 150 and 300 of about 1700 (Java) to 2400 (Rust) symbols; one symbol is 0.7 to 0.3 points.

## Verdict

Level 2 through the real way meets the threshold of decision 14 on the samples for **Rust and Java**. **TypeScript** has precision 99.4 % but, since #250, recall 91.8 % against the language service, which counts import lines (measured 2026-11-20); that is a question of what counts as a use on the side of the tool, open for the product owner. It is **not met for Python** (precision cannot be shown with Jedi, which does not see attributes of instances; recall 99 %) and **not for C#** (fault of scip-dotnet, issue #258, and two untraced cases). **Kotlin is not measured.** The run found one defect of its own (#392, fixed in #396) that stopped three of five indexers with the default limits, and one import gap (#394).

## Repeat

```
python3 tools/accuracy/measure.py LANGUAGE --samples 150 --seed 1 --level2-run --tools-bin DIR --codeskimr target/debug/codeskimr
```

`DIR` holds the indexers (`scip-typescript`, `scip-python`, `scip-java`, `scip-dotnet`); `rust-analyzer` comes from the `PATH`. `--set KEY=VALUE` passes settings to the level 2 scan. Run it in a container without credentials: the indexers run the build tools of the corpus (see `docs/accuracy.md`).
