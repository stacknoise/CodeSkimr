# 0019 – Team server with user management (after 1.0)

Status: accepted

## Context

Blueprint section 1 lists "Team server with multiple users and authentication" as out of scope for 1.0 but "already designed so it can be added without rework". Section 12.5 designs the team vault, section 4.3 and S-04 bind everything to `127.0.0.1`, and the roadmap has a "later" row for the team edition. The product owner confirmed: a team server with login is wanted, planned **after 1.0**.

This ADR is the concept for that step. **The team edition itself is not part of 1.0.** The one exception is a small seam, the caller identity in the use cases and the command registry (see "Seam in 1.0"), which the product owner decided to include in 1.0 because it is cheap now and expensive to retrofit. Everything else is implemented after M7.

Readings considered: (a) local profiles on one machine, (b) team server with login, roles and project permissions, (c) token authentication only for MCP over HTTP. Decision: (b); (c) becomes its first stage, (a) is covered by (b) and not built separately.

## Options

1. **Local profiles only.** Cheap, protects nothing, no remote access.
2. **MCP tokens only.** Small, but no users, roles or UI.
3. **Team server with users, roles and project permissions, delivered in stages.** Matches the existing design; largest scope.

## Decision

Option 3, in three stages, each with its own work packages and its own milestone after 1.0.

### Scope of the team edition

- Same executable, started in **server mode** (`codeskimr serve --team`). Single-user mode stays the default and behaves exactly as in 1.0 (one implicit local owner, no login, `127.0.0.1`).
- **Users and login.** Single sign-on through OIDC from the first login stage (decided by the product owner), next to local accounts with password (Argon2id), which remain for the bootstrap admin and small setups. Sessions via short-lived, server-side session tokens in a `Secure`, `HttpOnly`, `SameSite=Strict` cookie. Personal access tokens for CLI, REST and MCP.
- **Roles** (fixed in the first version). *Global role* of a user: `admin` (users, settings, every project; every admin access to a project without a grant is audited) or `user`. *Per-project role* in a grant: `maintainer` (triggers scans, manages members of that project), `member` (reads and searches), `viewer` (structure and metrics only, no source excerpts). Registering a project needs the global permission `register_project` (held by `admin` and grantable to users); the creator receives an automatic `maintainer` grant. Grants of a user and of their groups combine as a union; there are no deny rules. Custom roles are not in scope.
- **Permissions per project** (`ProjectGrant`: user or group, project, role). No grant means the project does not exist for that user: lists, search, counts, MCP tools and resources, and WebSocket events never reveal it. "Not found" and "no grant" give the same status, body and timing and never name the project. The grant filter runs inside the repository queries, not as a post-filter, and every object id (snapshot, file, symbol) is first resolved to its project (no IDOR). Cross-project features (search, `find_similar_code`, aggregates, shared caches) only see granted projects.
- **Audit log:** who did what and when (login, failed login, token creation, grants, scans, source reads, admin access without grant, credential use). Never values, never secrets; failed-login entries do not store the submitted user name verbatim (sanitised, truncated). The port has no update or delete, entries are hash-chained for tamper evidence, retention comes from config, and if an audit write fails the audited action fails.
- **Remote access** becomes possible only in server mode (TLS and proxy rules under Security). Server mode is chosen at start and cannot be switched at runtime. Shell access to the server host equals admin; standalone `codeskimr scan` and the CLI act as `LocalOwner` on local files, so standalone mode refuses to open a database that holds team data.

### Stages

1. **Stage A – tokens for MCP over HTTP (can ship first).** Several named personal access tokens instead of the single installation token, each with project scope, read-only flag and a mandatory maximum expiry, revocable, stored hashed. Without users a token has no owner: in Stage A a token is its own principal (`TokenPrincipal`, permissions = its scope), and the server still binds only to `127.0.0.1`; non-loopback binding waits for Stage B. No UI beyond a token page and CLI commands.
2. **Stage B – users, login (OIDC and local accounts), roles, grants** on SQLite, plus the admin UI. OIDC group claims can be mapped to groups and roles. Authorisation enforced in the application layer for REST, WebSocket, CLI-against-server and MCP alike.
3. **Stage C – scale and integration:** PostgreSQL adapter (already named in blueprint 4.4), team vault (12.5) wired to the user and project model, audit export.

### Domain and ports

New in `codeskimr_domain`: `User`, `Group`, `Role`, `ProjectGrant`, `AccessToken`, `Principal` (the authenticated caller; in single-user mode the fixed `LocalOwner`), `Permission` (read, read_source, scan, manage, register_project, admin; `viewer` lacks `read_source`, and every route, tool and export that returns source text requires it). Pure types and rules (`Role` → permissions, "may principal P do X on project Q"), no I/O.

New ports in `codeskimr_application`:

| Port | Responsibility | Adapter |
|---|---|---|
| `UserStore` | users, groups, grants, password hashes | `codeskimr_storage` (SQLite, later PostgreSQL) |
| `Authenticator` | credentials / token / OIDC code → `Principal` | new crate `codeskimr_auth` |
| `AuditLog` | append-only audit entries | `codeskimr_storage` |
| `Authorizer` | `check(principal, permission, project)` | implemented in the application layer on top of the domain rules |

Rules:

- **Every command and query takes a `Principal`.** This is the one change that 1.0 must anticipate: in 1.0 all use cases already receive a `Principal` that is always `LocalOwner`, and the command registry (blueprint 10.1) declares the required `Permission` per command. Authorisation then lives once in the registry, not in each adapter (single source, R-01). This is part of 1.0 (see "Seam in 1.0").
- `codeskimr_auth` is a driven adapter: it depends on `application` only, like the other adapters. Password hashing and OIDC libraries stay inside it. The dependency direction in blueprint 4.2 does not change.
- `codeskimr_secrets` stays free of user data. The team vault binds ciphertext to project id and credential id (12.5); Stage C adds the owning grant as access check, not as part of the key.

### Seam in 1.0 (decided by the product owner)

Work package "caller identity" ([issue #59](https://github.com/stacknoise/CodeSkimr/issues/59)), released before M3 so that M1/M2 code is retrofitted once and the REST/UI surface is built with it; no behaviour change for users:

- `Principal` and `Permission` exist in `codeskimr_domain`; the only principal in 1.0 is `LocalOwner`, which holds every permission.
- Every use case of the application layer takes a `Principal`, passed inside a request/context struct so that the parameter limit of R-02 holds; driving adapters (CLI, REST, MCP) pass `LocalOwner`.
- The command registry (10.1) declares the required `Permission` per command; one central check runs before execution.
- `Principal` is resolved in one place (composition root or one request extractor), never hard-coded in adapters. `LocalOwner` can only be constructed in the single-user composition root, and team-mode principals only by the `Authenticator`.
- No login, no user storage, no new dependencies, no UI. One new error code "permission denied" (class `4`); its group and id are fixed by the work package through the registry (`API-401` is the installation token and `APP` is start-up, so neither fits as is). Denial and "not found" must not become an oracle (see Storage/permissions rule).
- Tests: the check is covered by unit tests with a restricted test principal.

### Storage

- A separate **identity database** (own SQLite file, later own PostgreSQL schema), never in the analysis database, so that analysis data can be rebuilt or exported without touching accounts. Migrations in their own versioned directory.
- Tables: `users`, `groups`, `group_members`, `project_grants`, `access_tokens` (hash only, prefix for display, expiry, scope), `sessions`, `audit_log`.
- Analysis data stays per workspace/project as in 6.2. Project visibility is decided by grants, not by file layout.
- Per-user settings (language, view state) move into `user_settings` keyed by user; blueprint 13 already says the language choice is "stored per user".

### Security

This work is `security-sensitive`; every work package of it needs the security review and is merged by the product owner (CLAUDE.md rule 7).

- Passwords: Argon2id with parameters from `config/default.toml` and hard minimum floors in code; rehash on login when parameters change; constant-time comparison; no user enumeration (dummy hash verification for unknown users, same response and timing); back-off per account and per client IP, no hard lockout of the bootstrap admin; an admin password reset revokes all sessions.
- Access tokens: random 256 bit with a fixed prefix (`csk_`) so redaction and secret scanners can match them; shown once; stored as SHA-256 hash (acceptable only because of the entropy); scoped, expiring (maximum lifetime enforced), revocable, `last_used` tracked, lookups rate limited. Effective permission is the intersection of token scope and the owner's current permissions, checked at use time. A token cannot create tokens or change grants. The CLI keeps its token in the keystore (12.2), never in a config file. Tokens are never in URLs, logs or errors (E-05, S-01).
- Sessions: server-side, session id stored hashed, `__Host-` cookie with `Secure`, `HttpOnly`, `SameSite=Strict`; idle and absolute timeout; rotation on login and privilege change; revocation on logout, password change, user disable and grant change, plus "log out everywhere". CSRF protection by a synchronised token plus `Origin` check on every state-changing request (SameSite alone is not a CSRF defence). CORS stays closed. Host and `Origin` are validated against DNS rebinding.
- WebSocket (`/api/v1/events`): authenticated by the session cookie with a strict `Origin` check (no token in the URL); events are filtered per principal; open sockets close on session revoke, user disable or grant loss.
- OIDC: authorisation code with PKCE, `state` and `nonce`; exact-match redirect URI; validate `iss`, `aud`, `exp`, signature with an algorithm allow-list, JWKS cache with rotation. The pre-login state lives server-side (the `SameSite=Strict` cookie is not sent on the cross-site callback). Accounts link by (`iss`, `sub`), never by e-mail. Group-claim mapping is explicit, grants no admin by default and is re-evaluated at every login. Deprovisioning: sessions end by their maximum lifetime or a periodic re-check of the IdP. The client secret and the session-signing key live in the keystore, an environment secret or a secret file (12.2), never in config or SQLite. Local password login can be disabled once OIDC works, with one break-glass admin.
- S-01 is extended: redaction also covers `Cookie`/`Set-Cookie`, the OIDC `code`, `state`, `id_token` and the `csk_` token format. OIDC codes appear in callback URLs and access logs, so "never in URLs" applies to tokens, not to the one-time OIDC code, which is single-use and short-lived.
- Transport: S-04 is extended, not weakened. Non-loopback binding is allowed only in server mode and only with TLS configured **or** behind a reverse proxy that is configured as a trusted peer (then the server binds loopback or a unix socket; `X-Forwarded-*` is accepted only from trusted peers, the client address is the rightmost untrusted hop, and rate limits use it). Otherwise start fails with a registered error code. The follow-up blueprint PR changes the text of S-04 and of the 10.4 line "bound to 127.0.0.1, HTTP requires a per-installation token".
- Project registration is a path primitive: registrable roots and Git clone URLs are restricted by a server-side allow-list (path traversal, SSRF). S-05 additionally checks the grant. Who may use which stored credential for a scan is part of the grant model; a maintainer can only use credentials explicitly assigned to that project.
- The canary-secret suite (S-02) gets canaries for passwords, session and access tokens, OIDC secrets and the bootstrap code, and scans the identity database too: a password canary may appear there only as its Argon2 hash. Authorisation tests: every REST route, WebSocket message, CLI command and MCP tool is tested with each role and with "no grant", and "not found" and "no grant" are compared for identical responses.
- Bootstrap: the first admin is created with a one-time code (high entropy, single use, short expiry, stored hashed, attempts rate limited, endpoint disabled once an admin exists, only over loopback or TLS). The code is written to the console only and is excluded from file, journald and container logs. A lost-admin recovery is a local CLI command that needs file access to the identity database. No default password.
- New error codes (class `4` authentication / permission, blueprint 11.2; group chosen in the work package, possibly a new group) are registered before use; messages in German and English. The 12.5 vault binding must survive project deletion or id reuse (no re-binding of ciphertext to a reused id).

### Impact on interfaces

- **REST/WebSocket:** `/api/auth/*` (login, logout, tokens), `/api/admin/*` (users, grants), OpenAPI client regenerated. All other routes unchanged apart from the `Authorization` / session requirement.
- **CLI:** `codeskimr login --server <url>`, `codeskimr user …`, `codeskimr token …`; local commands keep working without login.
- **MCP:** stdio stays single-user and local. Streamable HTTP `/mcp` authenticates with a personal access token; tools and resources only see granted projects.
- **UI:** login page, user menu, admin screens (users, groups, grants, tokens, audit). A mock update is required first (blueprint 15.1): these screens do not exist in the mocks yet.

### Milestones (all after M7)

| Milestone | Content | Acceptance |
|---|---|---|
| **T1** | Stage A: personal access tokens for MCP/REST with project scope, auth crate skeleton, building on the 1.0 caller-identity seam | an agent reads only the granted project through `/mcp`; revoked token fails; canary suite extended |
| **T2** | Stage B: users, login with OIDC and local accounts, roles, grants, admin UI, audit log, TLS rule | role × route/tool matrix test green; no existence leak for foreign projects; security review passed |
| **T3** | Stage C: PostgreSQL adapter, team vault integration | same test matrix on PostgreSQL; vault access follows grants |

Each milestone is split into work packages when it is released; each needs the labels `security-sensitive` and, where a library is added, `new-dependency`.

## Consequences

- **1.0 stays single-user** with unchanged budgets and milestones; it gains only the caller-identity seam above, as one extra work package. After this ADR is accepted, blueprint sections 1, 4.1, 4.2, 4.3, 10.1 to 10.4, 11.2/11.3, 12.5, 12.6, 18.4 and 19 and `tools/ci/architecture.toml` get the matching text (S-04 and the 10.4 token line change, they do not only get a pointer) in a separate `blueprint-change` PR; until then nothing in the blueprint is edited by this PR.
- The caller-identity seam goes into 1.0 (decided by the product owner), tracked in #59. Issue #62 tracks the `blueprint-change` PR with the blueprint text changes.
- New crate `codeskimr_auth` needs an entry in the dependency diagram when accepted.
- New dependencies (password hashing, OIDC, TLS, PostgreSQL driver) are decided per work package with the `new-dependency` label.
- Risks: a team server enlarges the attack surface (network exposure, accounts). Mitigation: off by default, TLS or trusted proxy only, audit log, security review for every package.

## Open questions for the product owner (answered: the 1.0 seam is wanted; the rest is open)

1. ~~OIDC in the first release?~~ Answered: yes, OIDC is part of Stage B.
2. Are the fixed roles (global admin/user plus project roles maintainer, member, viewer) enough, or are custom roles expected early?
3. The `viewer` role is provisionally without source excerpts (permission `read_source`). Is that right?
