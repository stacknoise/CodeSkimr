# Secrets adapter (`codeskimr_secrets`)

Implements blueprint section 12 for 1.0. Depends on `application`, `domain` and `codes` only; the MCP crate never uses it.

## What it does

| Part | Purpose |
|---|---|
| `CredentialStore` (port in `application`) | `with_credential` lends a `Credential` to a closure (loan pattern, 12.3), `store` and `delete` manage entries. Not `Clone`, `Debug` or `Serialize`; the texts are `SecretString` and wiped on drop. |
| `KeystoreCredentials<K: Keystore>` | The port on top of a keystore. Reads the entry right before each use, decodes it, lends it, wipes it. Every call reads again. |
| `OsKeystore` | Windows Credential Manager, macOS Keychain, Linux Secret Service through `keyring-core` and the native stores (ADR 0018). Entries are filed under the service name `codeskimr`, named by the `CredentialId`. |
| Stored form | user name, a zero byte, secret. A user name with a zero byte is refused. A damaged entry counts as missing. |

The database stores only the `CredentialId` (`projects.credential`). `SourceFetcher` lends the credential to the Git login (see `source-adapter.md`).

## Installation token of the local server

The token that protects state-changing requests to the local server (`csk_` plus 64 hex digits) is kept in the same credential store, under the name `installation-token`, and created at the first `codeskimr serve`. The server holds only its digest. See `docs/server.md` and ADR 0024.

## Errors

`SEC-201` not found, `SEC-401` rejected, `SEC-402` access denied, `SEC-601` keystore unavailable. Keystore errors are mapped by variant and dropped unread, because some of them carry the bytes of the entry.

## Limits of this package

- Memory hardening (12.4: locking secret memory, no core dumps, `memsec`) and the CI listing of `expose_secret()` call sites are not part of it, see the linked issues in the pull request.
- Libraries copy secrets: `gix` receives the user and the password as plain `String`s for one request (residual risk of blueprint 15, "OS and library copies").
- No command stores a credential yet; `CredentialStore::store` is for the CLI and server packages.
