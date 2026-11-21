#!/usr/bin/env python3
"""List every use of `expose_secret` and `expose_user` in product code and compare with the approved list.

Blueprint 12.4: CI lists every call site, and each one needs reviewer approval. The approval
is an entry in tools/ci/expose-secret-approved.toml (file and number of calls). A new call, a
removed call or a file without an entry fails the check; the reviewer approves by merging a
change of the list. A use is every mention of one of the two names, also with the ending
`_mut`: the method call `x.expose_secret()`, the path call `Credential::expose_secret(&x)` and
the function reference `.map(Credential::expose_secret)`. Only the definition
(`fn expose_secret`) is not a use.

Comments, string literals and character literals are blanked before counting, so a brace or a
name inside them means nothing. Test code is not product code: files below `tests/`, files
named `*_tests.rs` and the one item that follows a `#[cfg(test)]` are skipped. The item ends
with its closing brace, with a semicolon or comma outside all brackets, or before a closing
bracket of the code around it (a marked field, variant, match arm or parameter). Code behind
the item is counted again, also on the same line. Where the end is unclear the check counts
rather too much, and it fails closed: a file whose comment, string or test item is still open
at its end, or that holds a string of a kind the check does not know, is an error.

Exit code 0 = fine, 1 = list and code differ or a file cannot be read safely, 2 = setup problem.
"""
import re
import sys
import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
APPROVED = Path(__file__).with_name("expose-secret-approved.toml")
CALL = re.compile(r"(\bfn\s+)?\bexpose_(?:user|secret)(?:_mut)?\b")
TEST_MODULE = re.compile(r"^\s*#\[cfg\(test\)\]")
# What must not be read as code: line comment, start of a block comment, start of a raw string,
# string literal, character literal (a lifetime has no closing quote and stays).
NO_CODE = re.compile(
    r"""//[^\n]*|/\*|\b[bc]?r(#*)"|[bc]?"(?:\\.|[^"\\])*"|b?'(?:\\.[^'\n]*|[^'\\\n])'""", re.DOTALL
)
BLOCK_EDGE = re.compile(r"/\*|\*/")
OPENING = "{(["
CLOSING = "})]"
ENDING = ";,"
# Close a list of type parameters or of closure parameters; `->` and `=>` are arrows.
LIST_END = re.compile(r"(?<![-=])>|\|")


class Unfinished(Exception):
    """A file ends inside a comment, a raw string or an item marked `#[cfg(test)]`."""


def is_test_file(path):
    return "tests" in path.parts or path.stem.endswith("_tests")


def block_comment_end(text, position):
    """Index behind the block comment that starts at `position`; block comments nest."""
    depth = 0
    for edge in BLOCK_EDGE.finditer(text, position):
        depth += 1 if edge.group() == "/*" else -1
        if depth == 0:
            return edge.end()
    raise Unfinished("a block comment is not closed")


def no_code_end(text, token):
    """Index behind the comment or literal that `token` starts."""
    if token.group() == "/*":
        return block_comment_end(text, token.start())
    if token.group(1) is None:
        return token.end()
    closing = text.find('"' + token.group(1), token.end())
    if closing < 0:
        raise Unfinished("a raw string is not closed")
    return closing + 1 + len(token.group(1))


def blank(text):
    """The text with comments and literals replaced by spaces; line breaks stay where they are."""
    parts = []
    position = 0
    while token := NO_CODE.search(text, position):
        end = no_code_end(text, token)
        parts.append(text[position:token.start()])
        parts.append(re.sub(r"[^\n]", " ", text[token.start():end]))
        position = end
    parts.append(text[position:])
    code = "".join(parts)
    if '"' in code:
        raise Unfinished("a string is not closed or of an unknown kind")
    return code


def list_ends(line, index, state):
    """True at a `|`, or at a `>` without its `<`, before the item opened its own brace.

    Such a sign closes the list of closure or type parameters the item stands in. Angle brackets
    of the item itself (`Vec<u8>`) are paired off first.
    """
    if state["depth"] != 0 or state["braced"]:
        return False
    if line[index] == "<":
        state["angles"] += 1
    elif LIST_END.match(line, index):
        state["angles"] -= 1
        return line[index] == "|" or state["angles"] < 0
    return False


def item_end(line, state):
    """Follows the brackets of the item after `#[cfg(test)]`.

    Returns the index in `line` behind the item, or None while the item goes on. The item ends
    with its closing brace, with a semicolon or comma outside all brackets (a statement, a field,
    a variant, a match arm), or before a closing sign of the code around it: a bracket, or the
    `>` or `|` of a parameter list (see `list_ends`). Only a brace outside all brackets is the
    item's own; one inside `[]` or `()` belongs to a type or an expression.
    """
    for index, sign in enumerate(line):
        if list_ends(line, index, state):
            return index
        if sign in OPENING:
            state["braced"] = state["braced"] or (sign == "{" and state["depth"] == 0)
            state["depth"] += 1
        elif sign in CLOSING:
            state["depth"] -= 1
            if state["depth"] < 0:
                return index
        if state["depth"] == 0 and (sign in ENDING or (sign == "}" and state["braced"])):
            return index + 1
    return None


def count_calls(text):
    """Line numbers of the uses outside comments, literals and items marked `#[cfg(test)]`."""
    found = []
    skipped = None
    for number, line in enumerate(blank(text).splitlines(), start=1):
        marker = TEST_MODULE.match(line)
        if marker:
            skipped = {"depth": 0, "braced": False, "angles": 0}
            line = line[marker.end():]
        if skipped is not None:
            end = item_end(line, skipped)
            if end is None:
                continue
            skipped = None
            line = line[end:]
        found.extend(number for definition in CALL.findall(line) if not definition)
    if skipped is not None:
        raise Unfinished("the item after #[cfg(test)] is not closed")
    return found


def scan(root):
    """Maps every product file with uses to their line numbers, and lists the unreadable files."""
    result = {}
    unreadable = []
    for path in sorted((root / "crates").rglob("*.rs")):
        if is_test_file(path.relative_to(root)):
            continue
        name = path.relative_to(root).as_posix()
        try:
            lines = count_calls(path.read_text(encoding="utf-8"))
        except Unfinished as error:
            unreadable.append(f"{name}: {error}, so the file cannot be checked")
            continue
        if lines:
            result[name] = lines
    return result, unreadable


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
    found, unreadable = scan(ROOT)
    for file, lines in found.items():
        print(f"{file}: {len(lines)} call(s) at line(s) {lines}")
    problems = unreadable + compare(found, approved)
    for problem in problems:
        print(f"ERROR {problem}")
    if problems:
        print("A reviewer must approve new calls in tools/ci/expose-secret-approved.toml.")
    return 1 if problems else 0


if __name__ == "__main__":
    sys.exit(main())
