#!/usr/bin/env python3
"""Generates a changelog section from the Conventional Commit subjects of squash-merged PRs.

Usage: tools/release/changelog.py <version> [--since <tag-or-commit>] [--until <ref>] [--date YYYY-MM-DD]
Prints one Markdown section to standard output (docs/release.md, step "Changelog").
"""
import argparse
import datetime
import re
import subprocess
import sys

REPO_URL = "https://github.com/stacknoise/CodeSkimr"
SUBJECT = re.compile(r"^(?P<type>[a-z]+)(\((?P<scope>[^)]+)\))?(?P<breaking>!)?: (?P<text>.+)$")
PR_SUFFIX = re.compile(r"\s*\(#(?P<pr>\d+)\)$")
# Order of the groups in the output; a type that is not listed here is reported as an error.
GROUPS = [
    ("feat", "Added"),
    ("fix", "Fixed"),
    ("refactor", "Changed"),
    ("docs", "Documentation"),
    ("test", "Tests"),
    ("chore", "Maintenance"),
]
SEPARATOR = "\x1f"


def parse_subject(subject):
    """Returns (type, text, pr number or None, breaking) or raises ValueError."""
    match = SUBJECT.match(subject)
    if not match:
        raise ValueError(f"not a Conventional Commit subject: {subject}")
    text = match.group("text")
    pr = PR_SUFFIX.search(text)
    if pr:
        text = text[: pr.start()]
    return match.group("type"), text.strip(), pr.group("pr") if pr else None, bool(match.group("breaking"))


def read_subjects(since, until):
    """The subjects of the commits on the first-parent line, newest first."""
    rev = f"{since}..{until}" if since else until
    out = subprocess.run(
        ["git", "log", "--first-parent", f"--format=%s{SEPARATOR}%h", rev],
        check=True, capture_output=True, text=True,
    ).stdout
    return [line.split(SEPARATOR) for line in out.splitlines() if line]


def render(version, date, subjects):
    """Builds the Markdown section; raises ValueError for subjects that do not parse."""
    known = {kind for kind, _ in GROUPS}
    grouped = {kind: [] for kind in known}
    breaking = []
    for subject, short in subjects:
        kind, text, pr, is_breaking = parse_subject(subject)
        if kind not in known:
            raise ValueError(f"unknown type '{kind}': {subject}")
        ref = f" ([#{pr}]({REPO_URL}/pull/{pr}))" if pr else f" (`{short}`)"
        grouped[kind].append(f"- {text}{ref}")
        if is_breaking:
            breaking.append(f"- {text}{ref}")
    lines = [f"## [{version}] - {date}", ""]
    if breaking:
        lines += ["### Breaking changes", "", *breaking, ""]
    for kind, title in GROUPS:
        if grouped[kind]:
            lines += [f"### {title}", "", *grouped[kind], ""]
    return "\n".join(lines)


def main(argv):
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("version")
    parser.add_argument("--since")
    parser.add_argument("--until", default="HEAD")
    parser.add_argument("--date", default=datetime.date.today().isoformat())
    args = parser.parse_args(argv)
    try:
        sys.stdout.write(render(args.version, args.date, read_subjects(args.since, args.until)))
    except ValueError as error:
        sys.stderr.write(f"error: {error}\n")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
