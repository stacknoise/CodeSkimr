# Browser UI (WP-42)

SolidJS, TypeScript and Vite in `web/`. Decisions: ADR 0028 and ADR 0030.

## Develop

```
cd web && npm ci
npm run dev                   # vite build --watch, writes web/dist
codeskimr --set ui.dir=web/dist serve
codeskimr open
```

## Checks

`npm run typecheck`, `npm run lint`, `npm run format`, `npm test` (Vitest) and `npm run e2e` (Playwright against the real server, needs Chromium; `CODESKIMR_CHROMIUM` can point to a binary).

## Structure

- `src/shared`: API client, session (launch code exchange, token in `sessionStorage`), theme store, texts (Fluent files from `locales/`), editor link, small UI parts.
- `src/features`: settings (theme cards with preview, language, editor), connection screen, projects (empty state for now).
- Themes Hell, Petrol (default) and Dunkel are CSS variables generated from `design/tokens.toml` by `cargo run -p codeskimr_codegen`; never edit the generated files.
- No literals in the code: values live in `consts.ts` files, texts in `locales/`.
