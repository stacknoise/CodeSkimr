# 0077 – Credentials live in one locked buffer

Status: accepted (Anton, 2026-11-20; the rebuild was decided on 2026-11-15, issue #298)

## Context

Blueprint 12.4 says that secret buffers are locked against swapping and zeroed on drop. Until now only the packed bytes of a keystore entry lay in `LockedBytes`. `decode` created two `SecretString`s on the ordinary heap, `encode` read from them, and so the user name and the secret could be written to the swap file during their use (issue #298, a rest from the security review of PR #296).

The application layer must not contain `unsafe` (R-08) and must not depend on an adapter (blueprint 4.2), so it cannot hold a `LockedBytes` itself.

## Decision

1. `Credential` (port `CredentialStore`) holds **one** locked buffer in the stored form `user`, a zero byte, `secret`, behind the small trait `LockedSecret` (`bytes()`), which `codeskimr_secrets::LockedBytes` implements. The application layer only borrows the bytes.
2. `Credential::new(SecretString, SecretString)` is gone. A credential is made by `codeskimr_secrets::credential(user, secret)` (refuses a user name with the zero byte, locks the memory, wipes the temporary packing buffer) or, when it is read from the keystore, by `decode`, which takes ownership of the locked copy of the entry. Nothing is copied into ordinary memory on the way.
3. The user name and the secret are read through `expose_user()` and `expose_secret()`. Both return `&str` into the locked buffer. Both parts are checked to be UTF-8 once, when the credential is built (`SEC-202` if not). CI lists the calls of both methods (`tools/ci/check_expose_secret.py`, approvals in `expose-secret-approved.toml`).
4. Storing a credential writes the buffer of the credential itself to the keystore; `encode` is no longer needed.
5. The handover to `gix` stays as short as `gix` allows: the two `String`s are made inside the loan, only when Git asks for the credential of the origin of the source, and move into the answer for that one request. `gix` takes plain `String`s and does not wipe them (blueprint 15, "OS and library copies").

## Residual risk (unchanged, stated honestly)

- The bytes that the operating system keystore library returns are an unlocked copy. They are wiped after they have been moved into the locked buffer.
- The `String`s that `gix` receives for one request are not wiped by `gix`.
- A secret that the user types on the terminal or sends from the UI first lies in a `SecretString` (zeroized on drop) until it has been moved into the credential, and an OAuth refresh token stays a `SecretString` while the request to the provider runs.
- Short-lived plain copies remain where a stored token is handed on as text: the installation token for the browser opener (`open.rs`), the refresh entry (`refresh.rs`) and the secret entered through `credentials.set`. Each lives in a `Zeroizing` or `SecretString` value and is wiped on drop.
- Locking can fail on systems with a small `RLIMIT_MEMLOCK`; that is `SEC-901` as before.

## Consequences

- `Credential` is no longer constructible without the secrets adapter; tests of other adapters use a small `LockedSecret` double of plain memory.
- `codeskimr_application` no longer depends on `secrecy`.
- `Credential::from_packed` is public so that the adapter can build credentials from its own locked buffers. It trusts the buffer to be locked; only `codeskimr_secrets` and test doubles call it.
- Blueprint 12.4 names the credential as locked memory.
