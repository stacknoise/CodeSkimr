#!/usr/bin/env python3
"""Reference answers for Python from Jedi: reads targets as JSON on stdin, prints per target the
list of [file, line] where Jedi finds a use inside the project."""
import json, sys
from pathlib import Path
import jedi

root = Path(sys.argv[1]).resolve()
project = jedi.Project(path=str(root), added_sys_path=[str(root / "src")])
out = []
for t in json.load(sys.stdin):
    path = root / t["file"]
    uses = set()
    try:
        script = jedi.Script(path=str(path), project=project)
        # Jedi columns are 0-based; the name sits at the start of the declaration line search below.
        text = path.read_text(encoding="utf-8").splitlines()
        line = text[t["line"] - 1] if t["line"] - 1 < len(text) else ""
        column = line.find(t["name"], t["column"]) if t["column"] <= len(line) else -1
        if column < 0:
            column = line.find(t["name"])
        if column >= 0:
            for ref in script.get_references(t["line"], column, scope="project"):
                if ref.module_path and Path(ref.module_path).resolve().is_relative_to(root) and not ref.is_definition():
                    uses.add((str(Path(ref.module_path).resolve().relative_to(root)).replace("\\", "/"), ref.line))
    except Exception as error:  # a symbol Jedi cannot handle counts as no answer
        print(f"jedi: {t['name']}: {error}", file=sys.stderr)
    out.append(sorted(uses))
print(json.dumps(out))
