# 0056 – A language pack can take query files over from another pack

Status: accepted

## Context

A pack has exactly one grammar and its own query files. The packs `typescript`, `tsx` and `javascript` share most of `symbols.scm`, `references.scm` and `metrics.scm`, so the files were kept as copies, and a test held the copies equal (issue #80, from the review of PR #79). A change to a rule of TypeScript had to be made three times, and a missed copy showed only as a failing equality test or as a silent difference between the languages.

## Decision

1. `language.toml` gets an optional table `[queries]` with `from` (the folder name of another pack) and `files` (a list of `symbols.scm`, `references.scm` and `metrics.scm`).
2. For every listed file the text of the file in the pack `from` comes first, the file of the pack itself follows (if there is one), and the two texts are compiled as one query against the grammar of the pack. The own file therefore only adds patterns; it cannot remove or change one of the base.
3. Rules: `from` is a plain folder name (no path), it is not the pack itself, `files` names only the three query files, and the pack `from` takes no files from a pack itself (no chains, so no cycles). A violation is `PRS-301`, as any other broken pack. A pack with `files = ["symbols.scm"]` needs no `symbols.scm` of its own.
4. The base is read from the folder beside the pack, so it works for a folder of packs and for the packs of the program alike (the program writes all packs to one folder, see `embedded_packs.rs`).
5. `typescript` is the base. `tsx` takes all three files and keeps only what is its own: the constants and fields whose value is JSX (`symbols.scm`) and the JSX references (`references.scm`). `javascript` takes `metrics.scm`, which is identical for it; its `symbols.scm` and `references.scm` stay its own, because the base names node kinds that the JavaScript grammar does not have (`interface_declaration`, `type_alias_declaration`, ...) and would not compile.

## Consequences

- The copies in `tsx` (`symbols.scm`, `references.scm`, `metrics.scm`) and `javascript` (`metrics.scm`) are gone; the golden files of all three packs stay as they are, which shows that the result is the same.
- The base must compile against the grammar of every pack that takes it. A base query with a node kind that is missing in a grammar makes the taking pack invalid at load time.
- Sharing the `symbols.scm` of `javascript` would need something finer than appending (a way to drop patterns). It is not part of this decision.
- No change of the data model, the storage or the wire formats.
