# 0024 – Local server: REST surface of the command registry and the request guard

Status: proposed

## Context

Blueprint 10.1 says the command registry generates the REST endpoints, 10.3 describes `/api/v1`, a WebSocket for events and "every route resolves a `Principal`", and S-04 binds the single-user server to `127.0.0.1`, requires the per-installation token for state-changing requests, closes CORS and validates `Host` and `Origin`. None of this says how the token is created and kept, how a typed command becomes a route, what "state-changing" means for a command, or in which order the checks run. WP-41 has to fix that, and the choices decide what the UI and the later team server (ADR 0019, stage T1) can build on.

## Decisions

1. **One route per command, `POST /api/v1/commands/{id}`.** The body is the JSON input (no body is `null`), the answer is the JSON output. `GET /api/v1/commands` lists the commands the caller may run (id, title and description keys, shortcut, `read_only`). The composition root exposes a registered command with `CommandApi::expose::<C>()`; a registered command that is not exposed is not reachable. No route has logic of its own (R-01). Resource-style URLs (`/projects/{id}`) are not generated; they would be a second description of the same action.
2. **The descriptor decides the token.** A command with `read_only = false` needs the token; read-only commands do not. This is the registry's own flag (it must agree with the permission, blueprint 10.1), so no route can forget it.
3. **Order of checks, tested:** `Host` and `Origin` (all requests) → the command exists (`API-201`) → the token for non-read-only commands (`API-401`) → the permission (`AUTH-401`) → the body (`API-501`, `API-101`) → the command. A caller without the permission sees the same answer whatever it sends, before any input is read (blueprint 10.1).
4. **Bodies are JSON or nothing.** Any other content type is refused with `API-101`. A form or a plain-text request from another site needs no CORS preflight; a JSON request does, and CORS is closed. Bodies over 1 MiB are refused with `API-501`.
5. **`Host` and `Origin`.** `Host` must be the bound address or `localhost` with the bound port (case-insensitive); anything else, or no `Host`, gives `API-402`. `Origin`, if present, must be `http://` plus one of those names; `null` and every other origin give `API-402`. No `Access-Control-*` header is ever sent. Every answer carries `Cache-Control: no-store` and `X-Content-Type-Options: nosniff`.
6. **The installation token** is 256 random bits as `csk_` plus 64 hex digits (the prefix of ADR 0019, so log redaction matches it). It is created at the first start of `codeskimr serve` and kept in the credential store (blueprint 12.2, the OS keystore); only a missing token is created. The server holds its BLAKE3 digest and compares digests, never the text. It is sent as `Authorization: Bearer …`, never in a URL. How the browser UI obtains it is decided with the UI scaffold (WP-42); until then there is no command that shows it.
7. **Bind address.** `codeskimr serve` listens on `127.0.0.1` and `server.port` (default 7421). There is no flag for the address. `Server::bind` refuses any address that is not a loopback address with `API-102` before it opens a socket; a taken port is `API-301`.
8. **Event stream.** `GET /api/v1/events` upgrades to a WebSocket and sends the domain events as JSON text messages (`scan_progressed`, `snapshot_created`, `files_changed`, `scan_stopped`). They carry ids, counts, stage labels and registered codes only: no names, paths or source text. The stream needs the `read` permission. `Origin` is checked on the upgrade; no token is read from the URL. What a client sends is ignored.
9. **Error answers** are `{ "code", "description" }` with the English description of the code, the HTTP status follows the class of the code (`AUTH-401` and `API-402` give 403, `API-501` gives 413). Neither the missing permission nor a cause is named.
10. **OpenAPI and the TypeScript client are not part of this package.** Blueprint 10.3 asks for `utoipa`. Typed schemas need the input and output types of real commands, which do not exist yet, and a client generator in CI. A follow-up package adds `utoipa` (a new dependency) when the first commands are exposed.

## Consequences

- New dependency: `axum` (features `http1`, `json`, `tokio`, `ws`) and its tree, in `codeskimr_server`; `getrandom` in the composition root. Both are widely used and already in the lock file in part (`tokio`, `hyper`, `tower`).
- New codes `API-102`, `API-201`, `API-301`, `API-402`, `API-501`; new setting `server.port`; new command `codeskimr serve`.
- The log redaction masks the `csk_` prefix (S-01).
- A command must have JSON-serialisable input and output to be exposed over REST (`DeserializeOwned`, `Serialize`).
- Risks: (a) the token sits in the keystore, so on a headless Linux without Secret Service `serve` stops with `SEC-601` instead of running unprotected; a file-based fallback needs its own decision. (b) A local process of the same user can read the keystore and use the token: S-04 protects against web pages and the network, not against code running as the user.
- Known limits, tracked in #88: no cap on WebSocket connections and unbounded event queues, no HTTP timeouts, and a command id is no secret (unknown id gives `API-201`, a known one without right gives `API-401` or `AUTH-401`); for the team stage this has to be compared with S-06.
- The team server (ADR 0019, T1) replaces the single token by named tokens and sessions behind the same route shape; the guard order stays.
