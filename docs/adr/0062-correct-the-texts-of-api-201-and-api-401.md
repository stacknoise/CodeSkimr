# 0062 – Correct the texts of `API-201` and `API-401`

Status: accepted (Anton, 2026-11-15, decision card on issue #400)

## Context

The QA walk before 1.0 (`docs/acceptance/qa-1-0.md`) found two descriptions in the error catalogue that no longer say what the code does:

- `API-201` reads "The REST request names a command or route that does not exist." Since ADR 0044 it also answers an MCP tool the caller does not know or may not run. An MCP client that asks for a tool that does not exist is told about a "REST request".
- `API-401` reads "A state-changing request lacks a valid installation token." Since WP-48 every request to `/mcp` needs the token, also a pure read, and so does a command that hands out source text (ADR 0035). The text names only one of the cases.

The descriptions are part of the catalogue in blueprint 21 and of the answer body (`{ "code", "description" }`), so a change needs an ADR (CLAUDE.md rule 9).

## Options

1. Leave the texts and document the wider meaning in `docs/server.md`. Costs nothing, but the answer itself stays misleading for the client that reads it.
2. Change the descriptions, the messages in `locales/` and the blueprint table. The codes, names, classes and status codes stay as they are.
3. Add new codes for the MCP cases. More precise, but a second code for the same condition, and ADR 0044 wants one answer for "unknown" and "not allowed".

## Decision

Option 2.

| Code | New description (en) | German message |
|---|---|---|
| `API-201` | The request names a command, tool or route that does not exist. | Diesen Befehl, dieses Werkzeug oder diese Adresse gibt es nicht. |
| `API-401` | The request lacks a valid installation token. | Für diese Anfrage fehlt ein gültiges Installations-Token. |

The English message of `API-201` becomes "This command, tool or route does not exist." and that of `API-401` "This request requires a valid installation token." The text of `API-201` still tells nothing about whether the thing exists or is only not allowed (ADR 0044).

## Consequences

- `errors/error-codes.toml`, the table in `BLUEPRINT.md`, `locales/de/errors.ftl`, `locales/en/errors.ftl` and the generated files (`docs/error-codes.md`, the generated TypeScript catalogue) change; no code, name, severity, class or status code changes.
- No test compares the old description texts; the tests that check codes stay as they are.
- Clients that match on the description text, not on the code, would see the change; none is known, and the code is the stable part of the answer.
