# 0070 – A reasoned exception from CHK-102 for pure mapping functions

Status: accepted (product owner, issue #361)

## Context

`CHK-102` (rule R-01, no duplication) compares the syntax fingerprints of functions. Two functions that copy the fields of one shape into another (a row into a model, a config section into a view, a request into a command, a port call into the call of the store behind it) look alike in every language, because the work is the same: one expression per field. They are no copies of logic. Merging them would hide the shape behind a generic helper (a macro, a trait with a closure per field) that is longer and harder to read than the two functions, and the fields of both would still have to be named. After the long functions were split (#373), most of the findings that `codeskimr check .` still reports for Rust are of this kind, and the check has no way to say "this is meant". `jscpd` and clippy are not touched by this ADR.

## Decision

1. A function is not reported as a copy (`CHK-102`) if both hold:
   - Its cognitive complexity is 0: no branch and no loop, so it only moves values from one shape into another. The pack must count it; a function of a language whose pack does not count the cognitive complexity (ADR 0048) is never exempt, because the claim cannot be checked.
   - The comment block directly above its declaration (doc comments and attributes included, ended by the first blank line, at most 12 lines) holds the marker `codeskimr: allow CHK-102` followed by a reason of at least 8 letters or digits, for example `// codeskimr: allow CHK-102 maps the storage row into the model`. The comment syntax of the language does not matter; the marker is plain text.
2. The marker removes the finding on the function that carries it (the later one of a pair, as `CHK-102` reports). A later, unmarked copy of a marked function is still reported.
3. The exception covers `CHK-102` only. Length (`CHK-104`), complexity (`CHK-105`) and parameters (`CHK-106`) are not touched, and neither are the thresholds. `duplicate_similarity_percent`, `duplicate_min_lines` and `duplicate_max_owners` stay as they are.
4. The marker is for mapping functions only. A function with logic is not made exempt by a marker: it has branches, so rule 1 refuses it. Parallel test functions are no mapping and are merged with a helper or a table instead. The reviewer checks every new marker for a real mapping and a reason that says what is mapped.
5. The marker is greppable (`rg "codeskimr: allow CHK-102"`), so the exceptions can be listed and counted at any time.

## Consequences

- `ConventionChecker::new` gets the port `SourceText` to read the lines above a declaration. Only the lines of functions that would be reported are read, and only if their cognitive complexity is 0.
- `codeskimr check`, `check_conventions` (MCP) and CI all apply the rule, because it lives in the use case.
- A file that cannot be read gives no exemption; nothing fails.
- The rule can be tightened later (for example to a counted budget of markers) without a change of the marker.
- Documented in `docs/conventions.md`. No change of the data model, storage or error codes.
