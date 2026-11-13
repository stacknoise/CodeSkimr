# Secrets adapter (`codeskimr_secrets`)

Implements blueprint section 12 for 1.0. Depends on `application`, `domain` and `codes` only; the MCP crate never uses it.

## What it does

| Part | Purpose |
|---|---|
| `CredentialStore` (port in `application`) | `with_credential` lends a `Credential` to a closure (loan pattern, 12.3), `store` and `delete` manage entries. Not `Clone`, `Debug` or `Serialize`; the texts are `SecretString` and wiped on drop. |
| `KeystoreCredentials<K: Keystore>` | The port on top of a keystore. Reads the entry right before each use, decodes it, lends it, wipes it. Every call reads again. |
| `OsKeystore` | Windows Credential Manager, macOS Keychain, Linux Secret Service through `keyring-core` and the native stores (ADR 0018). Entries are filed under the service name `codeskimr`, named by the `CredentialId`. |
| Stored form | user name, a zero byte, secret. A user name with a zero byte is refused. A damaged entry is `SEC-202`, distinct from a missing one (`SEC-201`). |

The database stores only the `CredentialId` (`projects.credential`). `SourceFetcher` lends the credential to the Git login (see `source-adapter.md`).

## Installation token of the local server

The token that protects state-changing requests to the local server (`csk_` plus 64 hex digits) is kept in the same credential store, under the name `installation-token`, and created at the first `codeskimr serve`. The server holds only its digest. See `docs/server.md` and ADR 0024.

## OAuth device flow (WP-73)

`DeviceFlow` (RFC 8628) signs in to GitHub or GitLab without a password: it asks the platform for a code, hands the page and the short code to the caller, polls the token endpoint (honouring `interval` and `slow_down`) and returns the token as a `Credential` for the store. `Provider` knows the endpoints and the minimal scope (GitHub `repo`, GitLab `read_repository`); `--host` replaces the public host for a self-managed installation. Rules: only HTTPS (plain HTTP only for `127.0.0.1`, for the tests), redirects are not followed (the device code would follow), the answers are limited to 64 KiB, nothing of the flow is logged and every failure is `NET-601` without a reason. The HTTP client is `reqwest` with the same TLS backend that `gix` already uses; the answers and the request body are wiped on our side, `reqwest` keeps its own copy for the duration of the request (residual risk, blueprint 15).

The use of the system's Git credential helper (point 1 of the preference order) is the default of a source without a credential; the only thing that changed is that a terminal prompt is switched off.

## Errors

`SEC-201` not found, `SEC-202` entry damaged, `SEC-401` rejected, `SEC-402` access denied, `SEC-601` keystore unavailable. Keystore errors are mapped by variant and dropped unread, because some of them carry the bytes of the entry.

## Limits of this package

- Memory hardening (12.4: locking secret memory, no core dumps, `memsec`) and the CI listing of `expose_secret()` call sites are not part of it, see the linked issues in the pull request.
- Libraries copy secrets: `gix` receives the user and the password as plain `String`s for one request (residual risk of blueprint 15, "OS and library copies").
- The server and the UI still have no endpoint or form to create a credential (issue #53, second half). The CLI commands are described in `docs/cli.md`.
- A secret from standard input is limited to 64 KiB. The read buffer of the standard input and the terminal library are copies the program cannot wipe (blueprint 15, library copies).
- The token of the device flow is stored as it is. GitHub tokens of an OAuth app do not expire; GitLab tokens do after two hours, and refreshing them is not built, so a GitLab token has to be created again by `credential login`.
