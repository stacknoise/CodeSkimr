# 0030 – UI delivery, launch code routes and the settings of WP-42

Status: proposed

## Context

ADR 0028 decided the framework, the editor link, the theme build and the launch code. WP-42 builds them. Building showed a few points that the ADR leaves open or only implies. This ADR records them so that the code and the documents agree. It extends ADR 0028 and changes none of its decisions.

## Decision

1. **Two routes create and watch the code, outside the command registry** (like the exchange route of ADR 0028). `POST /api/v1/session/launch-code` needs the installation token as `Authorization: Bearer` and returns a new code. `GET /api/v1/session/launch-code` needs the token too and answers whether a code is still waiting (`pending`). `codeskimr open` uses the second one to delete the launch file as soon as the browser has taken the code. `POST /api/v1/session/exchange` needs no token; a wrong code is `API-403` and does not burn the valid one.
2. **The server holds the token text for as long as the code lives.** Besides the BLAKE3 digest of the code it keeps the token (as a secret string, zeroed on drop) for at most the code's lifetime (`ui.launch_code_seconds`, default 60), so that the exchange can hand it over. It is dropped when the code is used or expires.
3. **`GET /api/v1/ui-config` is public** and carries no secret: editor template, allowed schemes and default theme. Host and Origin checks still apply.
4. **The UI is served from a folder** (`ui.dir`, default `ui`, found next to the data folder or the program). Only plain files below the folder are served (strict path allow-list, no `..`, no hidden files); paths without a file extension fall back to `index.html` for the router. Without the folder the server answers the API only. Embedding the built UI into the binary is a follow-up (done in #103: `build.rs` embeds `web/dist`; an existing `ui.dir` wins).
5. **Headers on every answer:** besides `no-store` and `nosniff`: `Content-Security-Policy: default-src 'self'; frame-ancestors 'none'; base-uri 'none'; form-action 'none'; object-src 'none'; connect-src 'self' ws://<own hosts>`, `Referrer-Policy: no-referrer` and `Cross-Origin-Opener-Policy: same-origin`.
6. **The launch file** is created exclusively (`create_new`) with mode 0600 in a private folder (0700): `$XDG_RUNTIME_DIR/codeskimr/launch` if that is an absolute path, else `<data folder>/launch`. A stale file of an earlier run is removed first. This covers issue 97.
7. **Editor links:** the path is encoded per path part, backslashes become slashes and a drive colon is kept. `editor.url_template` must contain `{path}` and a scheme from `editor.allowed_schemes`; otherwise the configuration fails with its registered code. `editor.*` and `ui.*` are read from the user file and the command line only, not from project files, because a project file must not choose the scheme the browser opens (see issue 54).
8. **Theme and language** live in the browser's `localStorage` until a settings store exists on the server.
9. **Not part of WP-42:** the bare `codeskimr` start command, embedding the UI in the binary, and tests of the launch file and browser start on Windows and macOS (the code has paths for them; they were run on Linux only).

10. **Exception to blueprint 10.1:** the launch code and exchange routes are not commands of the registry and take no `Principal`; the installation token (or the code) takes that role. They are the only such routes besides the UI files.
11. **Accepted risk:** `codeskimr open` sends the installation token over plain loopback HTTP to the configured port. A local user who listens on that port while no server runs could read it. This is accepted for the single-user mode; the team server (ADR 0019) uses its own login. `open` also refuses a code that is not 32 hexadecimal digits before it writes the launch page.
12. **Known gap:** decision 7 (project files cannot set `editor.*` and `ui.*`) holds because `serve` and `open` load the configuration without a project file. The configuration layer does not reject these keys yet; see the follow-up issue.

## Consequences

- The threat note in `docs/server.md` describes what a local attacker can and cannot do with the launch file and the 60-second window.
- The `e2e` job now runs for real: Playwright drives the real server and checks the CSP.
- New dependencies: the frontend packages of ADR 0028 (SolidJS, Vite, Fluent, Vitest, Playwright and their tools), and in the server `getrandom`, `secrecy` and `zeroize` for the code and the token loan.
