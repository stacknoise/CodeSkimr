# 0079 – The blueprint follows the code in four places

Status: accepted (within Anton's merge approval of 2026-11-13; the decisions behind it are ADR 0024, ADR 0033 and the licence decision)

## Context

The code review of 2026-11-21 found places where the blueprint says something other than the code, although the code follows a decision that was made later than the blueprint text. Rule 9 of `CLAUDE.md` asks for an ADR when the blueprint changes. This ADR changes no decision; it only writes the blueprint down as decided.

## Decision

1. **REST error answer (11.5).** The blueprint now says `{ "code", "description" }` with the matching HTTP status, as ADR 0024, decision 9, decided. The UI looks the localised text up by the code. The earlier text (`message`, `params`, `retryable`) was never built.
2. **`codeskimr_fsguard` (4.1, 4.2).** The crate is in the directory tree and in the table of allowed dependencies: it depends on no workspace crate, and `source` and `scip` may use it (ADR 0033, `tools/ci/architecture.toml`).
3. **Appendix A (error codes).** The table is written again from `errors/error-codes.toml`: the eight `ANN` codes of ADR 0068 were missing and `PRJ-502` did not show its severity. Notes of other changes to the table (for example retired codes) stay.
4. **Open points O-03 and O-04 (20.1)** are closed: Windows, Linux and macOS (CI builds and tests all three), and GitHub with a public repository and the MIT licence.

## Consequences

- No code changes follow from items 1 to 4. The CLI command `codeskimr codes` of blueprint 10.2, which was missing, and the button "Copy details" of 11.5 are built in the same pull request.
- The architecture check and the error-code checks of CI already enforce the new text.
