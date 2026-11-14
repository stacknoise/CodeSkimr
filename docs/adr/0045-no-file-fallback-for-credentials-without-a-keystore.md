# 0045 – No file fallback for credentials without an operating system keystore

Status: proposed

## Context

On Linux without a Secret Service (a server, a container, a desktop without a keyring) `codeskimr serve` and every credential command stop with `SEC-601` (issue #269). Blueprint 12.2 allows credentials only in the operating system keystore and forbids files, SQLite, configuration and logs. The installation token (S-04) also lives in the keystore, so without one the server cannot start at all. People ask for a file fallback so that headless machines work.

## Options

1. No fallback in 1.0. Keep `SEC-601`, improve its message and the manual (how to start a Secret Service such as `gnome-keyring` or `KeePassXC` without a desktop). Headless use waits for the team server vault (blueprint 12.5, ADR 0019).
2. A plain file with mode 0600 in the data folder, only when the user asks for it with a flag (`--insecure-file-store`). Easy, but it contradicts 12.2 point 4 and every backup, sync folder or support bundle may leak it.
3. An encrypted file, the key derived from a passphrase that the user types or sets in an environment variable. The environment variable is itself a stored secret (12.2 point 4 lists environment dumps), the typed passphrase blocks every non-interactive start.

## Recommendation

Option 1. Options 2 and 3 weaken the one rule that the canary suite (S-02) relies on, and option 3 only moves the secret into another place. The vault of 12.5 solves the headless case properly (envelope encryption, master key in an HSM or KMS) and is already designed. Until then the message of `SEC-601` names the cause and the fix.

## Consequences if accepted

- No code change except the message text of `SEC-601` (de/en) and a section in `docs/secrets.md` and the manual.
- Blueprint 12.2 stays as it is; the exclusion of a file fallback is written down there with a pointer to this ADR.
- Headless installations in 1.0 must provide a Secret Service; the installation token cannot be replaced by a file.
