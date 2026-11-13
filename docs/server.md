# Local server (WP-41)

`codeskimr serve` starts the local server: REST generated from the command registry, a WebSocket with the domain events, and the protection that blueprint S-04 asks for. The decisions are in ADR 0024. WP-42 added the browser UI delivery, the launch code and the CSP (ADR 0028, ADR 0030). The OpenAPI document is not part of these packages.

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
| `GET /api/v1/ui-config` | public, no secret: editor template, allowed schemes, default theme |
| `POST /api/v1/session/launch-code` | Bearer token; returns a new one-time launch code |
| `GET /api/v1/session/launch-code` | Bearer token; `pending` is true while a code waits to be used |
| `POST /api/v1/session/exchange` | body `{ "code": … }`; returns the token once (`API-403` if wrong, used or expired) |
| `/mcp` | MCP over streamable HTTP (WP-48); every request needs the Bearer token, see below |
| everything else | files of the UI folder; paths without extension give `index.html` |

Every error answer is `{ "code": "API-401", "description": "…" }` with the HTTP status of the code's class. Unknown paths give `API-201`, a wrong method `API-101` (405).

## Checks, in this order

1. `Host` is the bound address or `localhost` with the same port; `Origin`, if present, is `http://` plus one of these. Otherwise `API-402` (403). This is the protection against DNS rebinding and cross-site requests.
2. The command exists (`API-201`).
3. A command with `read_only = false`, and a command with the permission `read_source` (it returns source text, ADR 0032), needs `Authorization: Bearer <installation token>` (`API-401`).
4. The principal holds the permission of the command (`AUTH-401`, 403). This comes before the body is read: a denied caller always gets the same answer.
5. The body is at most 1 MiB (`API-501`, 413) and, if there is one, `application/json` (`API-101`).

CORS is closed: no `Access-Control-*` header is ever sent and there is no `OPTIONS` route. Every answer has `Cache-Control: no-store`, `X-Content-Type-Options: nosniff`, `Referrer-Policy: no-referrer`, `Cross-Origin-Opener-Policy: same-origin` and a strict `Content-Security-Policy` (no inline script or style, no framing, connections only to the own host).

## MCP at `/mcp` (WP-48)

The running app offers the MCP server of `docs/mcp.md` on the same listener, as blueprint 10.4 asks: `codeskimr serve` builds the tools once (`crates/app/src/mcp.rs`, `http_service`) and hands the `tower` service of `codeskimr_mcp::http_service` to `Server::with_mcp`. The server crate does not know `rmcp`; it only mounts a service at `/mcp`.

- **Checks:** the `Host` and `Origin` checks of step 1 come first, as for every route. Then the token: every request to `/mcp`, whatever its method, needs `Authorization: Bearer <installation token>` (`API-401`, 401), also a pure read. Tool answers carry the structure of the code, and without the token any program on this machine could read it through the open loopback port; the token proves that the caller can read the keystore, like in ADR 0032. A client such as Claude Code or Cursor is configured with the URL `http://127.0.0.1:7421/mcp` and the header `Authorization: Bearer <token>`. How the token reaches such a client is the task of the installation (the token is never shown by a command, see below).
- **Who calls:** every request runs as the principal of the composition root, the local owner in single-user mode. After 1.0 the team edition resolves a principal per personal access token and builds one handler per caller (ADR 0019); the tools check their permission as before (`rescan` needs `scan`).
- **Stateless:** every request is answered on its own with a JSON body, no session survives a request, and there is no `GET` stream. A restart of the app loses nothing; a client that sends `Mcp-Session-Id` is not tracked. The body is at most 1 MiB.
- **Shared instance:** the service uses the database file of the app and the event bus of the server. A `rescan` of an agent publishes the same events as a scan from the UI, so the UI shows its progress. The store is opened a second time next to that of the commands (one connection each, same file); sharing the connection waits until the commands take a store from the composition root.
- **Projects:** the app has no default project. A call that names none is `MCP-101` with `params.detail = project`; the ids come from `projects.list` over REST. An MCP tool that lists the projects is not part of WP-48.
- **Not yet:** the resources (`codeskimr://…`) belong to the MCP server itself, not to this transport.

## The installation token

`csk_` and 64 hex digits (256 random bits), created at the first `serve` and kept in the credential store (OS keystore). The server keeps the BLAKE3 digest for checking requests. The token text is read in `installation_token::digest_of` and, while a launch code is waiting, held as a secret string (see below); it is never logged and not part of any `Debug` output. The prefix is masked by the log redaction (S-01). There is no command that shows the token. The browser UI gets it through a launch code:

1. `codeskimr open` reads the token from the keystore and asks the server for a code (128 random bits, valid `ui.launch_code_seconds`, default 60, usable once, one code at a time).
2. It writes a redirect page to a launch file (mode 0600, created exclusively, in a private folder with mode 0700) and starts the browser with that `file://` path. The code is never on a command line. `--no-browser` only prints the path.
3. The page redirects to `http://127.0.0.1:PORT/#code=…`. The fragment is not sent to the server. The UI removes it with `history.replaceState`, then posts the code to the exchange route and keeps the token in `sessionStorage` only.
4. `open` polls until the code is taken or has expired and then deletes the file.

Threat note: `codeskimr open` sends the token to the port of `server.port` over plain HTTP on the loopback address. If no server runs there, another local user could listen on that port and read it. This is an accepted risk of the single-user mode (ADR 0030). The server holds the token text for at most the life of the code. Another OS user cannot read the launch file (0600 in a 0700 folder). A process of the same user could read it, but that user can read the keystore anyway. Within the 60 seconds a local process that guesses the code would need 2^128 tries, and a wrong code does not burn the valid one. A used or expired code is gone for good.

## Events

Messages have a `type` and ids and counts only: `scan_progressed` (`project`, `stage`, `done`, `total`), `snapshot_created` (`project`, `snapshot`), `files_changed` (`project`, `files`, `rescan`), `scan_stopped` (`project`, `reason` = `cancelled` or `failed`, `code`). The stream needs the `read` permission; messages from the client are ignored (limit 1 KiB). A thread per connection carries the events from the bus (which is synchronous) to the runtime and ends when the socket is gone.

## Configuration

`server.port` (default 7421, must not be 0). `editor.url_template` and `editor.allowed_schemes` (the link of "open in editor"), `ui.default_theme`, `ui.dir` (the folder of the built UI, default `ui`) and `ui.launch_code_seconds` (default 60). A taken port fails with `API-301`. The address is not configurable.

## Known limits

- The commands of the UI are registered in `crates/app/src/ui_commands/` (`command_api`): `projects.list` (read), `projects.add` (register_project, scan), `projects.rescan` (scan), `explorer.outline` and `explorer.symbol` (both read), `search.query` (read), `status.info` (read), `explorer.source` (read_source), `graph.layers` (read). Feature packages add theirs there. `explorer.outline` takes a project id and returns the layers, namespaces and types of its newest snapshot (the layers of the project; types that belong to none come last with an empty layer name); `explorer.symbol` takes a project id and a symbol id and returns the symbol with breadcrumb, metrics (with a rating against the configured thresholds) and member table. An unknown symbol is `MCP-202`. A project without a scan gives `explorer.outline` an empty outline with `snapshot: null`, and `explorer.symbol` the error `MCP-203`. `projects.add` takes an absolute folder path or a Git address without credentials, `projects.rescan` a project id; both run the scan on the thread of the call, publish progress on the event bus of the WebSocket and refuse a second scan of the same project with `PRJ-302`. `search.query` takes a project id and a text and returns at most 50 hits of the newest scan, best first, each with kind, qualified name, file and the enclosing symbol; a project without a scan or an empty text gives an empty list. `explorer.source` takes a project id and a symbol id (type or member) and returns the lines of its declaration as they are in the project folder now (`path`, `first_line`, `last_line`, `lines`, `truncated` after 400 lines, `available`). It needs `read_source` and the token (ADR 0032); a Git project answers `available: false` without lines; a file that is gone, outside the project folder or not a file is `SRC-201`, a file over 4 MiB `SRC-501`. `graph.layers` takes a project id and returns the dependency map of its newest snapshot as a picture: `boxes` (one per layer with `name`, `x`, `y`, `width`, `height`, `rank`, `order`), `links` (`from`, `to`, `count`, `violates_layering`, `in_cycle`) and the size of the picture (`width`, `height`). The layers keep the order of the project's `layers` setting, outer layers first, one row each (ADR 0031); a layer that only the scan's edges name is added after them. A project without a scan or without layers gives an empty picture (`snapshot: null` before the first scan). The positions come from the layered layout in Rust (`docs/graph-layout.md`); spacing and box size are the `[graph]` settings. `status.info` takes nothing and returns `startup_ms` (program start until the server answers, `null` if not measured) and `memory_mb` (physical memory of the process, `null` if the system does not tell). A relative folder path is refused with `PRJ-101`, so the working folder of the server never decides what is scanned.
- No OpenAPI document and no generated TypeScript client yet (needs `utoipa` and real command types, see ADR 0024).
- The UI is served from the folder `ui.dir`; it is not embedded in the binary yet. The bare `codeskimr` does not start the app; use `codeskimr serve` and `codeskimr open`.
- Launch file and browser start were run on Linux only.
- No TLS and no non-loopback binding: that is the team server's server mode (ADR 0019).
- `serve` needs a usable keystore. On a headless Linux without a Secret Service it stops with `SEC-601`.
- Events are not filtered per principal; in single-user mode there is only the local owner.
