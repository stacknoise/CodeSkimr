# Accuracy of usages per language (issue #134)

`tools/accuracy/measure.py` measures how well `find_usages` agrees with the reference tooling of a language on the pinned public corpora (`corpus/<language>/corpus.toml`). It is a measurement, not a CI gate.

```
python3 tools/accuracy/measure.py python --samples 60 --seed 1 --codeskimr target/debug/codeskimr
python3 tools/accuracy/measure.py typescript --samples 60 --seed 1
```

Steps: the script copies the corpus sources (without the skipped folders), scans them with `codeskimr scan`, draws symbols with a fixed seed (symbol kinds that can be used, names not starting with an underscore), asks the reference tool where each symbol is used, and compares by file and line.

| Language | Reference tool | Needs |
|---|---|---|
| Python | Jedi `get_references` | `pip install jedi` |
| TypeScript | TypeScript language service `findReferences` | `typescript` (set `TYPESCRIPT_MODULE` and `NODE_PATH` if it is not installed locally) |

Numbers over all drawn symbols: precision = usages of ours that the tool confirms / usages of ours; recall = usages of the tool that we found / usages of the tool; exact = share of symbols whose set of lines is identical. To compare two versions, pass the binary and the language packs that belong to it (`--codeskimr`, `--languages`).

## Limits of the comparison

- Comparison is by line, not by position. Two usages in one line count once.
- The reference tools are not complete either: Jedi does not follow attributes of instances in many cases, so some usages of ours that are right count as wrong. Read precision for Python as a lower bound.
- Only languages with a reference tool that runs without a build of the project are covered. Rust (rust-analyzer), Java (javac) and C#/Kotlin (compilers not available in CI containers) follow.
