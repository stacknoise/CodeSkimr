# 0028 – Frontend: framework, editor link, theme build and how the UI gets the token

Status: proposed

## Context

Work package WP-40 collects the decisions that the UI scaffold (WP-42) waits for: the framework (blueprint 4.4 and open point O-01), how "open in editor" works (O-05), how the three themes of ADR 0021 get into the build, and how the browser UI obtains the installation token that ADR 0024 (decision 6) left open. The product owner decided each point in turn.

## Options

1. Framework: SolidJS (kept), React (mainstream, larger bundle, more work for the budgets of 15.5 and 16.1) or Svelte (small, but a smaller ecosystem for virtual lists and no gain over Solid).
2. Editor link: a URL scheme opened by the browser (chosen); a command line started by the server (works with every editor, but starts a process from a web request and needs its own security review); both (more work in 1.0).
3. Themes: CSS variables generated at build time (chosen); Tailwind with a token preset (new dependency, the mocks use plain values); a theme object set by script at run time (flash at start, no build-time check).
4. Token for the UI: a one-time launch code (chosen); the token inside the served `index.html` (any other user of the same machine can fetch `http://127.0.0.1:7421/` and read it); the user copies the token by hand (poor usability, against "zero setup").

## Decision

1. **Framework: SolidJS with TypeScript and Vite**, as in blueprint 4.4. Virtual lists use `@tanstack/solid-virtual`.
2. **Editor link by URL scheme, configurable.** "Open in editor" (`E`, `Alt+Enter`) makes the browser open a URL built from a template, by default `vscode://file/{path}:{line}`. The template is the setting `editor.url_template` (added by WP-42 with the typed configuration). `{path}` and `{line}` are percent-encoded when inserted. The scheme must be on an allow-list in the configuration (`editor.allowed_schemes`, default `vscode`, `vscode-insiders`, `vscodium`, `cursor`, `idea`, `zed`, `subl`); any other scheme, and a template without a scheme, is refused with a registered configuration code. The server never starts a process for this. A command-line launcher is not part of 1.0; it needs its own ADR and a security review.
3. **Themes as CSS variables, generated at build time.** `design/tokens.toml` generates one CSS file with the variables of every theme, scoped by `[data-theme="…"]` (blueprint 15.3). Components use `var(--token)` only; no CSS framework, no run-time colour objects. The theme is set on `<html>` before the first paint by a small blocking script that is its own same-origin file (`theme-init.js`, so no inline script). It reads the choice from a mirror in `localStorage` (not secret; the settings page writes it whenever the stored choice changes). Without a mirror the order of ADR 0021 applies: Dunkel if the operating system is set to dark, else Petrol. The preview cards of screen 13 set `data-theme` on their own container. The `codegen` job keeps checking completeness and contrast.
4. **The UI gets the token through a one-time launch code.**
   - **Code:** 128 random bits. The server keeps only its BLAKE3 digest and compares digests (as for the token in ADR 0024). Valid for 60 seconds, usable once. Only one code is valid at a time; a new one invalidates the old one. Creating and using a code are logged (event only, never the code).
   - **Creating:** the start command and `codeskimr open` ask the running server for a code with the installation token from the credential store, so only the OS user who owns the keystore can ask.
   - **Handing it to the browser:** the code must not appear in a process list. The launcher writes a small redirect page to a file readable by the current user only (mode 0600, in the user's private runtime or temp directory), opens that `file://` path in the browser, and deletes the file after use or after 60 seconds. The page redirects to `http://127.0.0.1:{port}/#code=<code>`. The code is in the URL fragment, so it is never sent to the server and never appears in a `Referer` header or access logs.
   - **Exchange:** the UI's first script removes the fragment with `history.replaceState`, before any other script runs, and sends the code to `POST /api/v1/session/exchange`. The answer carries the installation token. A wrong, used or expired code gives a registered `API-4xx` code and no token, and the UI says that the code was already used or has expired.
   - **Exception to the guard, and its place:** this route is the one place where a state-changing request needs no token, and the only route outside the command registry (ADR 0024, decisions 1 and 2): there is no token yet, and the code takes its role. It has no logic of its own beyond checking the digest and returning the token. In the order of checks of ADR 0024 (decision 3) it follows `Host` and `Origin` and stands in place of the token check; it needs no permission because it grants nothing the keystore owner did not already hold. Blueprint S-04 states this exception.
   - **In the browser:** the token is kept in the tab's `sessionStorage` (survives a reload, is not shared with new tabs; a duplicated tab does copy it) and sent as `Authorization: Bearer …`. It is never put into a URL, a cookie or `localStorage`. If it is gone or rejected, the UI shows "open CodeSkimr again" with the command `codeskimr open`.
   - **Content-Security-Policy:** because a script injected into the page could read `sessionStorage`, the server sends a strict policy with every page: `default-src 'self'`, no `unsafe-inline` and no `unsafe-eval` for scripts and styles, `frame-ancestors 'none'`. Styles come from generated files; components do not use inline `style` attributes for static values.

## Consequences

- WP-42 builds the scaffold on these points, with the CSP header and `theme-init.js`. It also adds the exchange route, the launch-code command and `codeskimr open`, with the new error codes in `errors/error-codes.toml` and the configuration key `editor.url_template`. These are `security-sensitive` and need an updated threat note in `docs/server.md`.
- Blueprint changes with this ADR: 4.4 (reference to this ADR), 10.3 (token for the UI), S-04 (the exchange route as the one exception), 15.3 (build of the themes), 15.4 (`E`), and 20.1 (O-01 and O-05 closed).
- Risks: a local process of the same OS user can still read the keystore (known limit of ADR 0024). While the browser starts, the code exists in a private file and in the browser's address bar; it lives 60 seconds, works once, and a failed exchange in the legitimate browser shows that someone used it first. A browser extension that reads the tab URL sees the fragment; this is the same trust level as the page itself.
- The team server (ADR 0019) replaces the exchange by sign-in and sessions; the UI keeps one place that obtains credentials.
