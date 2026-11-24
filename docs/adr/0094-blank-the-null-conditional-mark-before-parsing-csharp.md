# 0094 – Blank the null-conditional mark of an assignment before parsing C#

Status: accepted (decision of Anton of 2026-11-24, "Das passt so", for option A of issue #705)

## Context

C# 14 allows an assignment to a null-conditional access: `a?.B = x`, `a?.B += x`, `a?[i] = x`. The grammar of the C# pack (`tree-sitter-c-sharp` 0.23.5, the newest version) does not know them. It reads the method that holds such a line as a field and the rest of the file as statements outside every class. A call after the line has no caller and is not listed: `find_usages` missed `if (!await _field.Method())` in a real project because an earlier method of the same file used `a?.B = false`. Nothing in the scan said so.

## Options

1. Wait for a grammar that knows the syntax. None is released and none is expected before 1.0.
2. Fork or patch the grammar. A new dependency source and a build of generated code for one line of syntax.
3. Prepare the source before parsing: the question mark in front of an assigned member becomes a space of the same width.

## Decision

Option 3, for the C# pack only, switched on by `blank_null_conditional_assignment = true` in `language.toml` (default `false`).

- **What is changed.** A `?` followed by `.name` (or `.name.name…`) or by `[…]`, and then `=` (not `==`, not `=>`), `??=` or a compound sign (`+=`, `-=`, `*=`, `/=`, `%=`, `&=`, `|=`, `^=`). `a?.B = x` is parsed as `a .B = x`, which is a plain assignment with the same target name.
- **Width.** The replacement is one space for one `?`, so no byte offset, line or column moves. Spans, snippets and the highlighting of the file read from disk stay right. Everything the analysis reads from the source comes from the prepared text: symbols, signatures, doc comments, content hashes and fingerprints (also the one of a snippet). Literals are fingerprinted by kind only, so their text does not matter. The text of a signature cannot hold an assignment. A doc comment or string that itself contains something like `a?.B = x` shows it with a space for the `?`; that is the one place where prepared text reaches a stored value. A symbol that holds such a line has a different content hash than before.
- **Where it is not applied.** Reads (`a?.B`), comparisons and lambdas are untouched. A match inside a string or a comment is possible and harmless: the space only changes the text of that literal for the parser.

## Consequences

- Files that use the new syntax keep their symbols and references. A project scanned before this change keeps its stored result until a file changes or a full scan runs; a full scan is needed once to see the calls that were lost.
- `a?.B <<= x` and `a?.B >>= x` are not covered (the grammar fails on them as before); the pack lists no such line in its tests.
- The scan still does not say when a file has syntax errors. That is a separate decision (a warning with the number and the files, issue to follow).
- A newer grammar makes the key unnecessary; the pack then sets it to `false` or drops it.
