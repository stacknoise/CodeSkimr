#!/usr/bin/env python3
"""Usage accuracy of `find_usages` per language against the tooling of the language (issue #134).

For one public corpus (`corpus/<language>/corpus.toml`) the script scans the pinned checkout with
`codeskimr scan`, picks symbols with a fixed seed, asks a reference tool of the language where
every symbol is used, and compares line by line with the references CodeSkimr stored.

    python3 tools/accuracy/measure.py python --samples 40 --seed 1 --codeskimr target/debug/codeskimr

Reference tools (`truth_<language>.*`): python = Jedi, typescript = the TypeScript language
service. The numbers per symbol: precision = our usages the tool confirms / our usages, recall =
tool usages we found / tool usages. A symbol counts as correct if both are 1 (exact).
"""
import argparse, json, random, shutil, sqlite3, subprocess, sys, tempfile, tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
KINDS = {"class", "interface", "function", "method", "property", "constructor", "field", "struct", "enum", "enum_member"}


def checkout(lang, project):
    path = ROOT / "target/corpus" / project["language"] / project["commit"]
    if not (path / ".git").exists():
        path.parent.mkdir(parents=True, exist_ok=True)
        subprocess.run(["git", "clone", "-q", project["repository"], str(path)], check=True)
    subprocess.run(["git", "-C", str(path), "checkout", "-q", "-f", project["commit"]], check=True)
    return path


def filtered_copy(src, project, dst):
    skip = set(project.get("skip_folders", [])) | {".git"}
    exts = {"." + e for e in project["extensions"]}
    suffixes = tuple(project.get("skip_suffixes", []))
    for file in src.rglob("*"):
        rel = file.relative_to(src)
        if file.is_file() and file.suffix in exts and not set(rel.parts[:-1]) & skip and not file.name.endswith(suffixes):
            (dst / rel).parent.mkdir(parents=True, exist_ok=True)
            shutil.copy2(file, dst / rel)


def scan(codeskimr, languages, source, data):
    subprocess.run([codeskimr, "--data-dir", str(data), "--languages", str(languages), "scan", str(source), "--json"],
                   check=True, capture_output=True)
    dbs = list(data.rglob("*.db")) + list(data.rglob("*.sqlite*"))
    if not dbs:
        sys.exit("no database found under " + str(data))
    return sqlite3.connect(max(dbs, key=lambda p: p.stat().st_size))


def ours(db):
    snapshot = db.execute("select max(id) from snapshots").fetchone()[0]
    files = dict(db.execute("select id, path from files where snapshot_id=?", (snapshot,)))
    symbols = {r[0]: r for r in db.execute(
        "select id, file_id, kind, name, qualified_name, start_line, start_column from symbols where snapshot_id=?", (snapshot,))}
    used = {}
    for to, file, line, kind, conf in db.execute(
            "select to_symbol, file_id, start_line, kind, confidence from symbol_references where snapshot_id=?", (snapshot,)):
        if kind in ("import",):
            continue
        used.setdefault(to, set()).add((files[file].replace("\\", "/"), line + 1))
    return files, symbols, used


def truth(lang, source, targets):
    """Reference tool: `targets` is a list of {file, line, column, name}; returns a list of sets of (file, line)."""
    here = Path(__file__).parent
    if lang == "python":
        cmd = [sys.executable, str(here / "truth_python.py"), str(source)]
    elif lang in ("typescript", "javascript"):
        cmd = ["node", str(here / "truth_typescript.js"), str(source)]
    else:
        sys.exit(f"no reference tool for {lang} yet")
    done = subprocess.run(cmd, input=json.dumps(targets), stdout=subprocess.PIPE, text=True, check=True)
    return [{(f, l) for f, l in result} for result in json.loads(done.stdout)]


def main():
    args = argparse.ArgumentParser()
    args.add_argument("language")
    args.add_argument("--samples", type=int, default=150)
    args.add_argument("--seed", type=int, default=1)
    args.add_argument("--codeskimr", default=str(ROOT / "target/debug/codeskimr"))
    args.add_argument("--languages", default=str(ROOT / "languages"), help="language packs that belong to the binary")
    args.add_argument("--json", action="store_true")
    opt = args.parse_args()
    project = tomllib.loads((ROOT / "corpus" / opt.language / "corpus.toml").read_text())["project"]
    src = checkout(opt.language, project)
    with tempfile.TemporaryDirectory() as tmp:
        tmp = Path(tmp)
        (tmp / "src").mkdir()
        filtered_copy(src, project, tmp / "src")
        db = scan(opt.codeskimr, opt.languages, tmp / "src", tmp / "data")
        files, symbols, used = ours(db)
        # Drawn by qualified name so that two versions of the scanner see the same symbols.
        pool = sorted((s[4], i) for i, s in symbols.items() if s[2] in KINDS and not s[3].startswith("_"))
        rng = random.Random(opt.seed)
        picked = [i for _, i in rng.sample(pool, min(opt.samples, len(pool)))]
        targets = [{"file": files[symbols[i][1]].replace("\\", "/"), "line": symbols[i][5] + 1, "column": symbols[i][6], "name": symbols[i][3]} for i in picked]
        sets = truth(opt.language, tmp / "src", targets)
    rows, exact, tp, fp, fn = [], 0, 0, 0, 0
    for i, t, real in zip(picked, targets, sets):
        mine = used.get(i, set())
        real = {x for x in real if x != (t["file"], t["line"])}
        hit = mine & real
        ok = mine == real
        exact += ok
        tp += len(hit); fp += len(mine - real); fn += len(real - mine)
        rows.append({"symbol": symbols[i][4], "ours": len(mine), "tool": len(real), "both": len(hit), "exact": ok})
    n = len(rows)
    summary = {"language": opt.language, "samples": n, "exact": exact, "exact_rate": round(exact / n, 3) if n else None,
               "precision": round(tp / (tp + fp), 3) if tp + fp else None, "recall": round(tp / (tp + fn), 3) if tp + fn else None}
    if opt.json:
        print(json.dumps({"summary": summary, "rows": rows}, indent=1))
    else:
        for r in rows:
            print(f"{'ok ' if r['exact'] else 'BAD'} ours={r['ours']:3} tool={r['tool']:3} both={r['both']:3}  {r['symbol']}")
        print(summary)


main()
