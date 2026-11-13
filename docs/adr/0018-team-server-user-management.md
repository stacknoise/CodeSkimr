# 0018 – Team server with user management (after 1.0)

Status: proposed

## Context

Blueprint section 1 lists "Team server with multiple users and authentication" as out of scope for 1.0 but "already designed so it can be added without rework". Section 12.5 designs the team vault, section 4.3 and S-04 bind everything to `127.0.0.1`, and the roadmap has a "later" row for the team edition. The product owner confirmed: a team server with login is wanted, planned **after 1.0**.

This ADR is the concept for that step. **It does not change the scope, milestones or budgets of 1.0.** It only fixes the seams that 1.0 must keep open, and describes the team edition so that it can be planned and split into work packages later. Nothing here is implemented before M7.

Readings considered: (a) local profiles on one machine, (b) team server with login, roles and project permissions, (c) token authentication only for MCP over HTTP. Decision: (b); (c) becomes its first stage, (a) is covered by (b) and not built separately.

## Options

1. **Local profiles only.** Cheap, protects nothing, no remote access.
2. **MCP tokens only.** Small, but no users, roles or UI.
3. **Team server with users, roles and project permissions, delivered in stages.** Matches the existing design; largest scope.

## Decision

Option 3, in three stages, each with its own work packages and its own milestone after 1.0.

### Scope of the team edition

- Same executable, started in **server mode** (`codeskimr serve --team`). Single-user mode stays the default and behaves exactly as in 1.0 (one implicit local owner, no login, `127.0.0.1`).
- **Users and login.** Local accounts with password (Argon2id) in the first version; OIDC (single sign-on) as an optional second adapter. Sessions via short-lived, server-side session tokens in a `Secure`, `HttpOnly`, `SameSite=Strict` cookie. Personal access tokens for CLI, REST and MCP.
- **Roles** (fixed in the first version): `admin` (users, settings, all projects), `maintainer` (registers projects, triggers scans, manages members of own projects), `member` (reads and searches projects granted to them), `viewer` (read-only, no export of source excerpts). Custom roles are not in scope.
- **Permissions per project** (`ProjectGrant`: user or group, project, role). No grant means the project does not exist for that user: lists, search, MCP tools and resources never reveal it (no "forbidden" that leaks existence).
- **Audit log:** who did what and when (login, failed login, token creation, grants, scans, credential use). Never values, never secrets.
- **Remote access** becomes possible only in server mode, and only over TLS (own certificate or reverse proxy with `X-Forwarded-*` trust list).

### Stages

1. **Stage A – tokens for MCP over HTTP (can ship first).** Several named personal access tokens instead of the single installation token, each with project scope and read-only flag, revocable, stored hashed. No UI beyond a token page and CLI commands.
2. **Stage B – users, login, roles, grants** on SQLite, plus the admin UI. Authorisation enforced in the application layer for REST, WebSocket, CLI-against-server and MCP alike.
3. **Stage C – scale and integration:** PostgreSQL adapter (already named in blueprint 4.4), OIDC, team vault (12.5) wired to the user and project model, audit export.

### Domain and ports

New in `codeskimr_domain`: `User`, `Group`, `Role`, `ProjectGrant`, `AccessToken`, `Principal` (the authenticated caller; in single-user mode the fixed `LocalOwner`), `Permission` (read, scan, manage, admin). Pure types and rules (`Role` → permissions, "may principal P do X on project Q"), no I/O.

New ports in `codeskimr_application`:

| Port | Responsibility | Adapter |
|---|---|---|
| `UserStore` | users, groups, grants, password hashes | `codeskimr_storage` (SQLite, later PostgreSQL) |
| `Authenticator` | credentials / token / OIDC code → `Principal` | new crate `codeskimr_auth` |
| `AuditLog` | append-only audit entries | `codeskimr_storage` |
| `Authorizer` | `check(principal, permission, project)` | implemented in the application layer on top of the domain rules |

Rules:

- **Every command and query takes a `Principal`.** This is the one change that 1.0 must anticipate: in 1.0 all use cases already receive a `Principal` that is always `LocalOwner`, and the command registry (blueprint 10.1) declares the required `Permission` per command. Authorisation then lives once in the registry, not in each adapter (single source, R-01). Whether this is added in 1.0 or in Stage B is decided when Stage B is planned; this ADR only records that adding it later must not change the signatures of the ports for analysis, parsing or source.
- `codeskimr_auth` is a driven adapter: it depends on `application` only, like the other adapters. Password hashing and OIDC libraries stay inside it. The dependency direction in blueprint 4.2 does not change.
- `codeskimr_secrets` stays free of user data. The team vault binds ciphertext to project id and credential id (12.5); Stage C adds the owning grant as access check, not as part of the key.

### Storage

- A separate **identity database** (own SQLite file, later own PostgreSQL schema), never in the analysis database, so that analysis data can be rebuilt or exported without touching accounts. Migrations in their own versioned directory.
- Tables: `users`, `groups`, `group_members`, `project_grants`, `access_tokens` (hash only, prefix for display, expiry, scope), `sessions`, `audit_log`.
- Analysis data stays per workspace/project as in 6.2. Project visibility is decided by grants, not by file layout.
- Per-user settings (language, view state) move into `user_settings` keyed by user; blueprint 13 already says the language choice is "stored per user".

### Security

This work is `security-sensitive`; every work package of it needs the security review and is merged by the product owner (CLAUDE.md rule 7).

- Passwords: Argon2id with parameters from `config/default.toml`; rate limiting and lockout back-off on login; constant-time comparison; no user enumeration (same response and timing for unknown user and wrong password).
- Tokens: random 256 bit, shown once, stored as hash; scoped, expiring, revocable; never in URLs, logs or errors (E-05, S-01).
- Sessions: rotation on login and privilege change; CSRF protection for state-changing requests; CORS stays closed; the installation token of S-04 is replaced by the principal check in server mode and remains for single-user mode.
- S-04 is extended, not weakened: **non-loopback binding is allowed only in server mode and only with TLS configured**; otherwise start fails with a registered error code.
- S-05 (path canonicalisation against registered project roots) additionally checks the grant.
- The canary-secret suite (S-02) gets canaries for passwords, session and access tokens, and OIDC secrets. Add authorisation tests: every REST route, WebSocket message, CLI command and MCP tool is tested with each role and with "no grant".
- A bootstrap step creates the first admin through a one-time code printed on the server console, never through a default password.
- New error codes in the `API`/`APP` groups (class `4` authentication / permission, blueprint 11.2) are registered before use; messages in German and English.

### Impact on interfaces

- **REST/WebSocket:** `/api/auth/*` (login, logout, tokens), `/api/admin/*` (users, grants), OpenAPI client regenerated. All other routes unchanged apart from the `Authorization` / session requirement.
- **CLI:** `codeskimr login --server <url>`, `codeskimr user …`, `codeskimr token …`; local commands keep working without login.
- **MCP:** stdio stays single-user and local. Streamable HTTP `/mcp` authenticates with a personal access token; tools and resources only see granted projects.
- **UI:** login page, user menu, admin screens (users, groups, grants, tokens, audit). A mock update is required first (blueprint 15.1): these screens do not exist in the mocks yet.

### Milestones (all after M7)

| Milestone | Content | Acceptance |
|---|---|---|
| **T1** | Stage A: personal access tokens for MCP/REST with project scope, auth crate skeleton, `Principal` in the command registry | an agent reads only the granted project through `/mcp`; revoked token fails; canary suite extended |
| **T2** | Stage B: users, login, roles, grants, admin UI, audit log, TLS rule | role × route/tool matrix test green; no existence leak for foreign projects; security review passed |
| **T3** | Stage C: PostgreSQL adapter, OIDC, team vault integration | same test matrix on PostgreSQL; vault access follows grants |

Each milestone is split into work packages when it is released; each needs the labels `security-sensitive` and, where a library is added, `new-dependency`.

## Consequences

- **1.0 is unchanged:** scope, budgets, milestones M1 to M7 and the single-user behaviour stay as they are. After this ADR is accepted, blueprint section 1, 4.3, 10.4, 12.5, 12.6 and 19 get a short pointer to this ADR in a separate `blueprint-change` PR; until then nothing in the blueprint is edited by this PR.
- Open seam for 1.0 work (recommendation, not a rule yet): use cases take a caller identity parameter and the command registry carries the required permission. Cheap to do now, expensive to retrofit later. The product owner decides whether this goes into an M-package.
- New crate `codeskimr_auth` needs an entry in the dependency diagram when accepted.
- New dependencies (password hashing, OIDC, TLS, PostgreSQL driver) are decided per work package with the `new-dependency` label.
- Risks: a team server enlarges the attack surface (network exposure, accounts). Mitigation: off by default, TLS-only, audit log, security review for every package.

## Open questions for the product owner

1. Is OIDC (company single sign-on) needed for the first team release, or are local accounts enough?
2. Are the four fixed roles enough, or are groups and custom roles expected early?
3. Should "viewer" be able to see source excerpts, or only metrics and structure?
4. Is the 1.0 seam (identity parameter and permission in the command registry) wanted now?
