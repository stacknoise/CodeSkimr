#!/usr/bin/env python3
"""Check the crate dependency direction from blueprint 4.2.

Reads `cargo metadata` and compares every dependency between workspace crates
with tools/ci/architecture.toml. Cargo itself already rejects crate cycles.
Exit code 0 = fine, 1 = violation, 2 = setup problem.
"""
import json
import subprocess
import sys
import tomllib
from pathlib import Path

RULES = Path(__file__).with_name("architecture.toml")


def workspace_graph():
    out = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--no-deps", "--locked"],
        check=True, capture_output=True, text=True,
    ).stdout
    packages = json.loads(out)["packages"]
    names = {p["name"] for p in packages}
    graph = {}
    for pkg in packages:
        deps = {d["name"] for d in pkg["dependencies"] if d["name"] in names}
        graph[pkg["name"]] = deps
    return graph


def main():
    rules = tomllib.loads(RULES.read_text(encoding="utf-8"))
    allowed = rules["allowed"]
    wildcard = set(rules["wildcard"]["crates"])
    problems = []
    for crate, deps in sorted(workspace_graph().items()):
        if crate in wildcard:
            continue
        if crate not in allowed:
            problems.append(f"{crate}: not listed in tools/ci/architecture.toml")
            continue
        for dep in sorted(deps - set(allowed[crate])):
            problems.append(f"{crate} must not depend on {dep} (blueprint 4.2)")
    for problem in problems:
        print(f"::error::{problem}")
    if problems:
        return 1
    print("Crate dependency direction OK.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
