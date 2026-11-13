# 0018 – OS keystore through `keyring-core` and the native stores

Status: proposed

## Context

Blueprint 12.2 and 4.4 name the `keyring` crate for the OS keystore. In version 4 that crate is a thin wrapper whose Linux store (`zbus-secret-service-keyring-store`) depends on `zbus` 5 and with it on `zvariant`, `zvariant_derive` and `zbus_names`. The advisory database flags all three as unmaintained (RUSTSEC-2026-0372 and its siblings), so `cargo deny` fails and the required `deps` check (S-03) blocks every pull request. Loosening `deny.toml` needs an ADR and is not wanted for the crate that holds credentials. The authors of `keyring` recommend that applications which choose their stores link `keyring-core` and the stores they need.

## Options

1. Use `keyring` and ignore the three advisories in `deny.toml`.
2. Use `keyring-core` with `windows-native-keyring-store` (Windows Credential Manager), `apple-native-keyring-store` (macOS Keychain) and `dbus-secret-service-keyring-store` (Linux Secret Service, `libdbus` built from source with the `vendored` feature).
3. Write the three platform bindings ourselves.

## Decision

Option 2. The keystore is still the one of the operating system, behind the same `keyring` project API (`Entry`), only without the wrapper crate and without `zbus`.

- Blueprint 4.4 and 12.2 name `keyring-core` and the native stores instead of the `keyring` crate.
- `codeskimr_secrets::OsKeystore` sets the store of the platform once, on first use. All entries are filed under the service name `codeskimr`.
- `deny.toml` is not changed.

## Consequences

- The Linux build compiles `libdbus` and its crypto from source (`vendored`): no system package is needed, the build is slower.
- If a maintained `zbus` store appears later, switching back is a change in `crates/adapters/secrets/Cargo.toml` and one function in `keystore.rs`.
- The Windows and macOS stores are compiled only by the matching `build` jobs of CI.
