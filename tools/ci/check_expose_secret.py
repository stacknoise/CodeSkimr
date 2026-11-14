#!/usr/bin/env python3
"""List every `expose_secret()` call in product code and compare with the approved list.

Blueprint 12.4: CI lists every call site, and each one needs reviewer approval. The approval
is an entry in tools/ci/expose-secret-approved.toml (file and number of calls). A new call, a
removed call or a file without an entry fails the check; the reviewer approves by merging a
change of the list. Test code is not product code: files below `tests/`, files named
`*_tests.rs` and everything after the first `#[cfg(test)]` of a file are skipped, and so are
comments.

Exit code 0 = fine, 1 = list and code differ, 2 = setup problem.
"""
import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
APPROVED = Path(__file__).with_name("expose-secret-approved.toml")
CALL = re.compile(r"\.expose_secret\(\)")
TEST_MODULE = re.compile(r"^\s*#\[cfg\(test\)\]")


def is_test_file(path):
    return "tests" in path.parts or path.stem.endswith("_tests")


def count_calls(text):
    """Number of calls outside comments and test modules."""
    found = []
    for number, line in enumerate(text.splitlines(), start=1):
        if TEST_MODULE.match(line):
            break
        if line.lstrip().startswith("//"):
            continue
        found.extend([number] * len(CALL.findall(line)))
    return found


def scan(root):
    """Maps every product file with calls to the line numbers of its calls."""
    result = {}
    for path in sorted((root / "crates").rglob("*.rs")):
        if is_test_file(path.relative_to(root)):
            continue
        lines = count_calls(path.read_text(encoding="utf-8"))
        if lines:
            result[path.relative_to(root).as_posix()] = lines
    return result


def compare(found, approved):
    problems = []
    for file, lines in found.items():
        entry = approved.get(file)
        if entry is None:
            problems.append(f"{file}: {len(lines)} call(s) at line(s) {lines} without approval")
        elif entry["calls"] != len(lines):
            problems.append(
                f"{file}: {len(lines)} call(s) at line(s) {lines}, approved are {entry['calls']}"
            )
    for file in sorted(set(approved) - set(found)):
        problems.append(f"{file}: approved but no call left, remove the entry")
    return problems


def main():
    try:
        approved = {e["file"]: e for e in tomllib.loads(APPROVED.read_text("utf-8"))["approved"]}
    except (OSError, KeyError, tomllib.TOMLDecodeError) as error:
        print(f"setup problem: {error}")
        return 2
    found = scan(ROOT)
    for file, lines in found.items():
        print(f"{file}: {len(lines)} call(s) at line(s) {lines}")
    problems = compare(found, approved)
    for problem in problems:
        print(f"ERROR {problem}")
    if problems:
        print("A reviewer must approve new calls in tools/ci/expose-secret-approved.toml.")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
