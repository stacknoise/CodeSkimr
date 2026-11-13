# 0028 – Frontend: framework, editor link, theme build and how the UI gets the token

Status: proposed

## Context

Work package WP-40 collects the decisions that the UI scaffold (WP-42) waits for: the framework (blueprint 4.4 and open point O-01), how "open in editor" works (O-05), how the three themes of ADR 0021 get into the build, and how the browser UI obtains the installation token that ADR 0024 (decision 6) left open. The product owner decided each point in turn.

## Decision

1. **Framework: SolidJS with TypeScript and Vite**, as in blueprint 4.4. Virtual lists use `@tanstack/solid-virtual`. Reasons: fine-grained reactivity and a small bundle serve the budgets of 15.5 and 16.1 (tree and tables with 100k rows). Options rejected: React (larger bundle, more effort for the budgets) and Svelte (smaller ecosystem for virtual lists, no gain over Solid).
2. **Editor link by URL scheme, configurable.** "Open in editor" (`E`, `Alt+Enter`) makes the browser open a URL built from a template, by default `vscode://file/{path}:{line}`. The template is a setting (`editor.url_template` in `config/default.toml`, added by WP-42 with the typed configuration), so other editors work (for example Cursor or JetBrains with their own scheme). The server never starts a process for this. A template whose scheme is `javascript`, `data`, `http` or `https` is refused with a registered configuration code. A command-line launcher is not part of 1.0; it needs its own ADR and a security review.
3. **Themes as CSS variables, generated at build time.** `design/tokens.toml` generates one CSS file with the variables for every theme, scoped by `[data-theme="…"]` (blueprint 15.3). Components use `var(--token)` only. No CSS framework and no runtime colour objects are added. The theme is set on `<html>` before the first paint (inline script from the stored choice or the operating system), so there is no flash. The preview cards of screen 13 set `data-theme` on their own container. The `codegen` job keeps checking completeness and contrast.
4. **The UI gets the token through a one-time launch code.**
   - The running server creates a launch code on request: 128 random bits, kept in memory only, valid for 60 seconds, usable once. The CLI asks for it with `POST /api/v1/commands/…` using the installation token from the credential store (so only the OS user who owns the keystore can ask). A fresh code is created whenever the start command or `codeskimr open` opens the browser.
   - The browser opens `http://127.0.0.1:{port}/#code=<code>`. The code is in the URL fragment, so it is never sent to the server and never appears in a `Referer` header or in access logs.
   - The UI removes the fragment at once and sends the code to `POST /api/v1/session/exchange`. The answer carries the installation token. The usual `Host` and `Origin` checks of ADR 0024 apply; a wrong, used or expired code gives a registered `API-4xx` code and no token.
   - The UI keeps the token in the tab's `sessionStorage` (it survives a reload, not a new tab or a closed browser) and sends it as `Authorization: Bearer …`. It is never put into a URL, a cookie or `localStorage`. If it is gone or rejected, the UI shows "open CodeSkimr again" with the command `codeskimr open`.
   - Options rejected: embedding the token in the served `index.html` (any other user of the same machine can fetch `http://127.0.0.1:7421/` and read it) and letting the user copy the token by hand (poor usability, against "zero setup").

## Consequences

- WP-42 builds the scaffold on these points. It also adds the exchange route, the launch-code command and `codeskimr open`, with the new error codes in `errors/error-codes.toml` and the configuration key `editor.url_template`. These are `security-sensitive` and need an updated threat note in `docs/server.md`.
- Blueprint changes with this ADR: 4.4 (reference to this ADR), 10.3 (token for the UI), S-04, 15.3 (build of the themes), 15.4 (`E`), and 20.1 (O-01 and O-05 closed).
- Risks: a local process of the same OS user can still read the keystore (known limit of ADR 0024). The launch code is only as secret as the URL while the browser opens, so it lives 60 seconds and works once. A browser extension that reads the tab URL sees the fragment; this is the same trust level as the page itself.
- The team server (ADR 0019) replaces the exchange by sign-in and sessions; the UI keeps one place that obtains credentials.
