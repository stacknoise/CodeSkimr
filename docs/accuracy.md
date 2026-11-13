# Accuracy of usages per language (issue #134)

`tools/accuracy/measure.py` measures how well `find_usages` agrees with the reference tooling of a language on the pinned public corpora (`corpus/<language>/corpus.toml`). It is a measurement, not a CI gate.

```
python3 tools/accuracy/measure.py python --samples 150 --seed 1 --codeskimr target/debug/codeskimr
python3 tools/accuracy/measure.py typescript --samples 150 --seed 1
```

Steps: the script copies the corpus sources (without the skipped folders), scans them with `codeskimr scan`, draws symbols with a fixed seed from the list sorted by qualified name (so two versions see the same symbols; symbol kinds that can be used, names starting with an underscore are left out, which also leaves out all Python constructors), asks the reference tool where each symbol is used, and compares by file and line.

| Language | Reference tool | Needs |
|---|---|---|
| Python | Jedi `get_references` | `pip install jedi` |
| TypeScript | TypeScript language service `findReferences` | `typescript` (set `TYPESCRIPT_MODULE` and `NODE_PATH` if it is not installed locally) |

Numbers over all drawn symbols: precision = usages of ours that the tool confirms / usages of ours; recall = usages of the tool that we found / usages of the tool; exact = share of symbols whose set of lines is identical. To compare two versions, pass the binary and the language packs that belong to it (`--codeskimr`, `--languages`).

## Limits of the comparison

- The sample is small: with 60 symbols one symbol is 1.7 percentage points. Use `--samples 150` or more to compare versions.
- A symbol with no usage on either side counts as exact.
- Imports are left out on our side only (they are not usages), which lowers recall a little.
- Precision and recall are summed over all usages, so symbols with many usages weigh more than the exact rate shows.
- The TypeScript run uses a non-strict configuration with bundler module resolution, not the `tsconfig` of the project.
- Comparison is by line, not by position. Two usages in one line count once.
- The reference tools are not complete either: Jedi does not follow attributes of instances in many cases, so some usages of ours that are right count as wrong. Read precision for Python as a lower bound.
- Only languages with a reference tool that runs without a build of the project are covered. Rust (rust-analyzer; see #134), Java (javac) and C#/Kotlin (compilers not available in CI containers) follow.

## Results

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
