# 0078 – Retire error codes that nothing uses

Status: accepted (within the merge approval of Anton of 2026-11-13; it only retires unused codes and follows up in the docs)

## Context

Rule E-02 of the blueprint says CI fails on unused codes, but no check existed. The code generator now reports a registered code that no production code names (`ErrorCode::Variant` in Rust, the quoted id in TypeScript; tests, inline test modules and generated files do not count). Its first run found `UI-901` (now used by the error boundary of the UI, E-03) and four codes nobody raises: `APP-301`, `IDX-301`, `NET-501`, `NET-502`.

The blueprint (11.2) says codes are never reused or renumbered and that a retired code is marked `deprecated = true`. ADR 0060 allows removing a code that is in no published release; none is published yet.

## Options

1. Raise the four codes somewhere. Nothing in the planned scope for 1.0 needs them: sources are local folders and `file://` (issue #388), there is no remote platform client, a second instance is handled without a user-facing error, and a stale read model is rebuilt silently.
2. Remove them (ADR 0060). Smallest catalogue, but the numbers could come back with another meaning in a later PR by accident.
3. Retire them with `deprecated = true`. The numbers stay reserved, messages stay, the check skips them.

## Decision

Option 3. The four codes are marked `deprecated = true`. The catalogue table in `BLUEPRINT.md` says "Retired (ADR 0078)" on their lines, and the user manual no longer lists `APP-301` as something a person can meet. A retired code that is needed later is raised again by removing the flag in the PR that raises it.

## Consequences

- `errors/error-codes.toml`, the generated files, `BLUEPRINT.md` (four table lines), `docs/manual/de/fehler.md` and `docs/manual/en/errors.md` change; no behaviour changes.
- From now on a new code that nothing uses fails the `codegen` job; a code must be used in the PR that registers it, or be registered as retired.
