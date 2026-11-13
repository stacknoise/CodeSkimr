#!/usr/bin/env python3
"""Check the architecture rules from blueprint 4.2 and R-04.

Two checks run in one job (`architecture`):

1. Crate dependency direction: every dependency between workspace crates
   (normal, dev and build) must be allowed by tools/ci/architecture.toml, and
   the rule file itself must match the workspace (no stale or missing crates).
2. Module cycles: inside each crate, the top-level modules (`src/<name>.rs` or
   `src/<name>/`) must not form a cycle through `crate::` / `super::` paths.
   Cargo already rejects cycles between crates.

Exit code 0 = fine, 1 = violation, 2 = setup problem.
"""
import json
import re
import subprocess
import sys
import tomllib
from pathlib import Path

RULES = Path(__file__).with_name("architecture.toml")
ROOT_FILES = {"lib.rs", "main.rs"}
IDENT = re.compile(r"[A-Za-z_][A-Za-z0-9_]*")


def load_packages():
    out = subprocess.run(
        ["cargo", "metadata", "--format-version", "1", "--no-deps", "--locked"],
        check=True, capture_output=True, text=True,
    ).stdout
    return json.loads(out)["packages"]


def workspace_graph(packages):
    names = {p["name"] for p in packages}
    return {
        pkg["name"]: {d["name"] for d in pkg["dependencies"] if d["name"] in names}
        for pkg in packages
    }


def check_rules_file(rules, graph):
    """The rule file must describe exactly the workspace crates."""
    problems = []
    allowed = rules["allowed"]
    wildcard = set(rules["wildcard"]["crates"])
    known = set(graph)
    for crate in sorted((set(allowed) | wildcard) - known):
        problems.append(f"{crate}: listed in tools/ci/architecture.toml but not in the workspace")
    for crate, deps in sorted(allowed.items()):
        for dep in sorted(set(deps) - known):
            problems.append(f"{crate}: rule allows unknown crate {dep}")
    return problems


def check_directions(rules, graph):
    allowed = rules["allowed"]
    wildcard = set(rules["wildcard"]["crates"])
    problems = []
    for crate, deps in sorted(graph.items()):
        if crate in wildcard:
            continue
        if crate not in allowed:
            problems.append(f"{crate}: not listed in tools/ci/architecture.toml")
            continue
        for dep in sorted(deps - set(allowed[crate])):
            problems.append(f"{crate} must not depend on {dep} (blueprint 4.2)")
    return problems


def strip_rust(text):
    """Blank out comments, string and char literals so paths in them are ignored."""
    out, i, n = [], 0, len(text)
    while i < n:
        two = text[i:i + 2]
        if two == "//":
            while i < n and text[i] != "\n":
                i += 1
        elif two == "/*":
            depth = 1
            i += 2
            while i < n and depth:
                if text[i:i + 2] == "/*":
                    depth, i = depth + 1, i + 2
                elif text[i:i + 2] == "*/":
                    depth, i = depth - 1, i + 2
                else:
                    i += 1
        elif text[i] == "r" and re.match(r'r#*"', text[i:i + 8]) and not (out and (out[-1].isalnum() or out[-1] == "_")):
            hashes = re.match(r"r(#*)\"", text[i:]).group(1)
            end = text.find('"' + hashes, i + 2 + len(hashes))
            i = n if end < 0 else end + 1 + len(hashes)
        elif text[i] == '"':
            i += 1
            while i < n and text[i] != '"':
                i += 2 if text[i] == "\\" else 1
            i += 1
        elif text[i] == "'" and re.match(r"'(\\.[^']*|[^\\'])'", text[i:i + 12]):
            i += re.match(r"'(\\.[^']*|[^\\'])'", text[i:i + 12]).end()
        else:
            out.append(text[i])
            i += 1
    return "".join(out)


def group_heads(text, start):
    """Heads of a `{a, b::c, d}` use group starting at text[start] == '{'."""
    heads, depth, i, expect = [], 0, start, True
    while i < len(text):
        ch = text[i]
        if ch == "{":
            depth += 1
            expect = depth == 1 or expect
        elif ch == "}":
            depth -= 1
            if depth == 0:
                break
        elif ch == "," and depth == 1:
            expect = True
        elif depth == 1 and expect and not ch.isspace():
            m = IDENT.match(text, i)
            if m:
                heads.append(m.group())
                i = m.end()
                expect = False
                continue
            expect = False
        i += 1
    return heads


def targets_after(text, pos):
    """Top-level names named by the path continuing at text[pos]."""
    while text[pos:pos + 1].isspace():
        pos += 1
    if text[pos:pos + 1] == "{":
        return group_heads(text, pos)
    m = IDENT.match(text, pos)
    return [m.group()] if m else []


def module_edges(text, mod_path, top_modules):
    """Top-level modules referenced from a file whose module path is `mod_path`."""
    found = set()
    for m in re.finditer(r"(?<![A-Za-z0-9_:])(crate::|(?:super::)+|self::)", text):
        kind = m.group(1)
        if kind == "crate::":
            base = []
        elif kind == "self::":
            base = mod_path
        else:
            base = mod_path[:max(len(mod_path) - kind.count("super::"), 0)]
        if base:
            found.add(base[0])
            continue
        found.update(targets_after(text, m.end()))
    return found & top_modules


def crate_modules(src):
    """Map each top-level module of a crate to the .rs files that belong to it."""
    modules = {}
    for entry in sorted(src.iterdir()):
        if entry.is_file() and entry.suffix == ".rs" and entry.name not in ROOT_FILES:
            modules[entry.stem] = [(entry, [entry.stem])]
        elif entry.is_dir() and entry.name != "bin":
            files = []
            for f in sorted(entry.rglob("*.rs")):
                rel = f.relative_to(src).with_suffix("")
                parts = list(rel.parts)
                if parts[-1] == "mod":
                    parts.pop()
                files.append((f, parts))
            modules[entry.name] = files
    return modules


def module_graph(src):
    modules = crate_modules(src)
    tops = set(modules)
    graph = {}
    for name, files in modules.items():
        deps = set()
        for path, mod_path in files:
            deps |= module_edges(strip_rust(path.read_text(encoding="utf-8")), mod_path, tops)
        graph[name] = deps - {name}
    return graph


def find_cycle(graph):
    """Return one cycle as a list of nodes (first == last), or None."""
    state, stack = {}, []

    def visit(node):
        state[node] = 1
        stack.append(node)
        for nxt in sorted(graph.get(node, ())):
            if state.get(nxt) == 1:
                return stack[stack.index(nxt):] + [nxt]
            if nxt not in state:
                found = visit(nxt)
                if found:
                    return found
        stack.pop()
        state[node] = 2
        return None

    for node in sorted(graph):
        if node not in state:
            found = visit(node)
            if found:
                return found
    return None


def check_cycles(packages):
    problems = []
    for pkg in sorted(packages, key=lambda p: p["name"]):
        src = Path(pkg["manifest_path"]).parent / "src"
        if not src.is_dir():
            continue
        cycle = find_cycle(module_graph(src))
        if cycle:
            problems.append(f"{pkg['name']}: module cycle {' -> '.join(cycle)} (R-04)")
    return problems


def main():
    rules = tomllib.loads(RULES.read_text(encoding="utf-8"))
    packages = load_packages()
    graph = workspace_graph(packages)
    problems = (
        check_rules_file(rules, graph)
        + check_directions(rules, graph)
        + check_cycles(packages)
    )
    for problem in problems:
        print(f"::error::{problem}")
    if problems:
        return 1
    print("Crate dependency direction and module cycles OK.")
    return 0


if __name__ == "__main__":
    sys.exit(main())
