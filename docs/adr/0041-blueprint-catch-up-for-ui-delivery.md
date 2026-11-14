# 0041 – Blueprint catch-up for the UI delivery and its hardening

Status: accepted

## Context

ADR 0028 and ADR 0030 decided how the browser UI is delivered and how it gets the installation token. The code follows them (WP-42, #102), but the blueprint still says otherwise in four places (issue #115), and the hardening of issue #106 adds rules that belong in the security section.

## Decision

1. **Path of the design tokens** (blueprint 3.2): the generated file is `web/src/shared/theme/tokens.ts`, as `tools/codegen` writes it.
2. **Exception to blueprint 10.1/10.3:** `POST` and `GET /api/v1/session/launch-code`, `POST /api/v1/session/exchange`, `GET /api/v1/ui-config` and the files of the UI are not commands of the registry and take no `Principal`. The installation token (launch-code route) or the one-time code (exchange) takes that role. There are no other such routes.
3. **S-04** names what ADR 0028 and 0030 added and what the hardening of #106 requires:
   - strict Content-Security-Policy without inline script and style, no framing, and a `Permissions-Policy` that switches off camera, microphone, location, payment and USB;
   - the launch file is created exclusively in a private folder (0700 on Unix; on Windows the rules the folder inherits are dropped and only the account of the user is granted access); the folder and everything above it belong to the user or the administrator and cannot be replaced by others (a folder that everybody can write to is accepted only with the sticky bit);
   - copies of the token and of the launch code in memory (request text, answer text, address) are wiped when they are dropped;
   - the UI comes from the embedded files or from `ui.dir`; an existing folder wins, links below it are not followed, and a folder outside the data folder is written to the log;
   - the project file of a scanned repository cannot set `editor.*`, `ui.*` or `level2.*` (they are dropped before merging, like `level2` since ADR 0033);
   - `editor.allowed_schemes` can never contain `javascript`, `data`, `file` or `vbscript` in any layer; the configuration is then refused with `CFG-101`.
4. **15.4** carries the last rule at the shortcut `E`.

## Consequences

- No behaviour is added that ADR 0028 and 0030 did not already decide; this ADR makes the blueprint say what the product does and fixes the rules for the hardening.
- `server.port` in the project file is still open and tracked in issue #289 together with the other settings of the team stage.
- ADR 0030, decision 12 (known gap) is closed by the dropped tables.
