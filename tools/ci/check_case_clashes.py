#!/usr/bin/env python3
"""Fail when two tracked paths, or two path prefixes (folders), differ only in case.

Windows and macOS treat `Preview.tsx` and `preview.ts`-style names, and also the folders
`Foo/` and `foo/`, as the same entry, so a repository containing such a pair cannot be
checked out there. Every prefix of every tracked path is compared, not only whole paths.

Exit code 0 = fine, 1 = clashes found.
"""
import collections
import subprocess
import sys


def prefixes(path):
    """Every path prefix of `path`: `a/b/c.rs` gives `a`, `a/b` and `a/b/c.rs`."""
    parts = path.split("/")
    return ["/".join(parts[: count + 1]) for count in range(len(parts))]


def find_clashes(paths):
    """Groups of different spellings that fold to the same name."""
    spellings = collections.defaultdict(set)
    for path in paths:
        for prefix in prefixes(path):
            spellings[prefix.casefold()].add(prefix)
    return [sorted(names) for names in spellings.values() if len(names) > 1]


def tracked_paths():
    out = subprocess.run(
        ["git", "-c", "core.quotePath=false", "ls-files"],
        check=True, capture_output=True, text=True,
    )
    return out.stdout.splitlines()


def main():
    clashes = find_clashes(tracked_paths())
    for names in clashes:
        print("Paths that differ only in case:", *names, file=sys.stderr)
    return 1 if clashes else 0


if __name__ == "__main__":
    sys.exit(main())
