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

## Credentials in the server (WP-76)

The browser UI and other REST callers manage logins with three commands (blueprint 12.4; `docs/server.md`):

- `credentials.set` takes `id`, `user` and `secret` and stores them in the keystore of the system. The secret travels in this one request (it needs the installation token), is wrapped in a `SecretString` at once and is wiped after storing (the request body and the JSON value it came from are not wiped: residual risk, blueprint 15). The answer holds the name only. An empty name, user or secret and the name `installation-token` are refused with `API-101`; a body that does not parse is `API-101` as well and never echoes the value (checked over HTTP in the security suite).
- `credentials.delete` takes `id`; a missing credential is not an error.
- `credentials.list` names the credentials that Git projects log in with and whether the keystore holds each (`stored`). It never shows a secret and needs only the read right.
- `projects.add` takes `credential` for a Git source. The project row holds the name only.

The scanner and the commands share one credential store (`wiring::Secrets`). The fetcher of Git sources sees it through `wiring::ForRemotes`, which hides the installation token: a project row that names `installation-token` cannot make a clone send it to a remote (`SEC-201`).

## Errors

`SEC-201` not found, `SEC-202` entry damaged, `SEC-401` rejected, `SEC-402` access denied, `SEC-601` keystore unavailable. Keystore errors are mapped by variant and dropped unread, because some of them carry the bytes of the entry.

## Limits of this package

- Memory hardening (12.4): the packed bytes of a credential lie in `LockedBytes`, a block of its own pages (`memsec::malloc_sized`, locked with `mlock` / `VirtualLock`, wiped and unlocked on drop), so freeing one buffer never unlocks another. A refused lock is `SEC-901`. A test measures `VmLck` of `/proc/self/status` (Linux). **Not covered:** the decoded `SecretString`s (user, secret) and the copy that the OS keystore library returns live on the normal heap; they are wiped on drop but may be swapped out while in use. `gix` needs plain `String`s for a login, so this stays a residual risk (blueprint 15). It is tracked in issue #298.
- Core dumps: at program start `disable_core_dumps` sets the core limit to 0 and `PR_SET_DUMPABLE` to 0 (Linux, macOS: limit only) and returns `Off`. On Windows `SetErrorMode` only switches off the error dialog and report; a machine-wide `LocalDumps` policy can still write a dump, so the result is `Partly` and logged at debug level. All `unsafe` lives in `crates/adapters/secrets/src/memory.rs`.
- CI lists every `expose_secret()` call in product code (`tools/ci/check_expose_secret.py`, job `architecture`); a new call needs an entry in `tools/ci/expose-secret-approved.toml`, which the reviewer approves.
- Libraries copy secrets: `gix` receives the user and the password as plain `String`s for one request (residual risk of blueprint 15, "OS and library copies").
- The server and the UI still have no endpoint or form to create a credential (issue #53, second half). The CLI commands are described in `docs/cli.md`.
- A secret from standard input is limited to 64 KiB. The read buffer of the standard input and the terminal library are copies the program cannot wipe (blueprint 15, library copies).
- The token of the device flow is stored as it is. GitHub tokens of an OAuth app do not expire; GitLab tokens do after two hours, and refreshing them is not built, so a GitLab token has to be created again by `credential login`.

- First start: the installation token is created while the program holds an exclusive lock on `token.lock` in the data folder (`fs4`), so two programs that start together end with one token (issue #269). The wait for the lock ends after 10 seconds with an error instead of blocking the start for ever. A keystore without compare-and-set needs this lock.
