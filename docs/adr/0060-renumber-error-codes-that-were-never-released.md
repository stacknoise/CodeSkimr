# 0060 – An error code that was never released may be renumbered

Status: accepted

## Context

Blueprint section 11.1 says that error codes are never reused or renumbered; a retired code stays in `errors/error-codes.toml` with `deprecated = true`. The reason is that users, scripts and the manual refer to released codes.

`APP-203` (the embedded language packs cannot be written to the data folder, issue #327) was added after the last release. The first digit of a code decides its class, so `APP-203` was classed "not found", which does not describe the failure; `APP-602` fits (issue #351). The product owner decided on the decision card of #351 to renumber it.

## Decision

1. A code that is in no published release (no tag, no release notes, no published manual) may be renumbered once. It is then removed from the file, not deprecated, because nobody can have seen it.
2. Released codes stay as they are: never reused, never renumbered, retired with `deprecated = true`.
3. Blueprint 11.1 and the header of `errors/error-codes.toml` state the exception.
4. `APP-203` becomes `APP-602` (same name `language_packs_unavailable`); `APP-203` is not used again.

## Consequences

- The catalogue, the Fluent texts, the manual and the generated files change in the same pull request as the renumbering.
- Every renumbering needs this exception to apply: the pull request says that the old code was in no release.
