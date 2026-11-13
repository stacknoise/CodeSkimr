# Local server (WP-41)

`codeskimr serve` starts the local server: REST generated from the command registry, a WebSocket with the domain events, and the protection that blueprint S-04 asks for. The decisions are in ADR 0024. The embedded UI and the OpenAPI document are not part of this package.

## Structure

- `codeskimr_server` is a driving adapter. `CommandApi` holds the commands that are exposed over REST, `Server` builds the router, `BoundServer` serves it. It depends on `application`, `domain` and `codes` only.
- The composition root (`crates/app`, `serve.rs`) wires it: it creates the `Registry` and `CommandApi`, resolves the `Principal` once (`principal.rs`), loads or creates the installation token (`installation_token.rs`) and binds `127.0.0.1:{server.port}`.
- `commands.rs` turns a typed `Command` into a route (`expose::<C>()`, needs `DeserializeOwned` input and `Serialize` output). The call goes through `Registry::execute`, so the permission is checked there as for every other surface. Commands run on the blocking pool, never on the async runtime.

## Routes

| Route | Meaning |
|---|---|
| `GET /api/v1/commands` | the commands the caller may run: `id`, `title_key`, `description_key`, `shortcut`, `read_only` |
| `POST /api/v1/commands/{id}` | runs a command; body = JSON input (none = `null`), answer = JSON output |
| `GET /api/v1/events` | WebSocket, one JSON text message per event |

Every error answer is `{ "code": "API-401", "description": "…" }` with the HTTP status of the code's class. Unknown paths give `API-201`, a wrong method `API-101` (405).

## Checks, in this order

1. `Host` is the bound address or `localhost` with the same port; `Origin`, if present, is `http://` plus one of these. Otherwise `API-402` (403). This is the protection against DNS rebinding and cross-site requests.
2. The command exists (`API-201`).
3. A command with `read_only = false` needs `Authorization: Bearer <installation token>` (`API-401`).
4. The principal holds the permission of the command (`AUTH-401`, 403). This comes before the body is read: a denied caller always gets the same answer.
5. The body is at most 1 MiB (`API-501`, 413) and, if there is one, `application/json` (`API-101`).

CORS is closed: no `Access-Control-*` header is ever sent and there is no `OPTIONS` route. Every answer has `Cache-Control: no-store` and `X-Content-Type-Options: nosniff`.

## The installation token

`csk_` and 64 hex digits (256 random bits), created at the first `serve` and kept in the credential store (OS keystore). The server keeps only the BLAKE3 digest. The token text is read in one place (`installation_token::digest_of`, one `expose_secret()` call) and never leaves the loan. The prefix is masked by the log redaction (S-01). There is no command that shows the token yet; how the browser UI gets it is decided with the UI scaffold (WP-42).

## Events

Messages have a `type` and ids and counts only: `scan_progressed` (`project`, `stage`, `done`, `total`), `snapshot_created` (`project`, `snapshot`), `files_changed` (`project`, `files`, `rescan`), `scan_stopped` (`project`, `reason` = `cancelled` or `failed`, `code`). The stream needs the `read` permission; messages from the client are ignored (limit 1 KiB). A thread per connection carries the events from the bus (which is synchronous) to the runtime and ends when the socket is gone.

## Configuration

`server.port` (default 7421, must not be 0). A taken port fails with `API-301`. The address is not configurable.

## Known limits

- No command is exposed yet: `GET /api/v1/commands` answers `[]`. The feature packages register their commands in `serve.rs` (`command_api`).
- No OpenAPI document and no generated TypeScript client yet (needs `utoipa` and real command types, see ADR 0024).
- No embedded UI and no browser start (`codeskimr` without arguments).
- No TLS and no non-loopback binding: that is the team server's server mode (ADR 0019).
- `serve` needs a usable keystore. On a headless Linux without a Secret Service it stops with `SEC-601`.
- Events are not filtered per principal; in single-user mode there is only the local owner.
