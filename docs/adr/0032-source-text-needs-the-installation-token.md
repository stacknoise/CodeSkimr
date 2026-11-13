# 0032 – Commands that return source text need the installation token

Status: proposed

## Context

ADR 0024 lets read-only commands run without the installation token: the server listens on the loopback address only, CORS is closed, and the numbers and names that a read-only command returns are low risk. The explorer's source column (WP-45b) returns source text of a project folder. Blueprint 5.2 says source text needs `read_source`. On a machine with several programs or several users, any local process can reach the loopback port; without a token it could read every file that a scanned project contains.

## Options

1. Treat `read_source` like `read`: no token. Simple, but the open port becomes a way to read source code.
2. Require the token for every command that has the permission `read_source`, whether it changes something or not.
3. Mark such commands as not read-only. That would also let them change things as far as the registry is concerned and falsely suggest a mutation.

## Decision

Option 2. The route checks the token for a command that is not `read_only` and for a command whose permission is `read_source`. The rule is one function in `crates/server/src/routes.rs` (`needs_token`) and is tested with a command of the test server. The order of checks stays: the command must exist, then the token, then the permission, then the body is read. A caller without the right gets `AUTH-401` whatever it sends.

## Consequences

- `explorer.source` (and any later command with `read_source`) works in the browser UI, which holds the token, and not for a bare `curl` without it.
- `GET /api/v1/commands` still lists the command with `read_only: true`; the field means "does not change anything".
- Only folder projects show source in 1.0. A Git project's clone lies in the cache folder and is not read from the UI yet.
- The reader (`codeskimr_source::FolderText`) resolves the path and the root and refuses anything that lies outside the root, also through a link; files over 4 MiB are refused with `SRC-501`; a declaration is cut at 400 lines.
