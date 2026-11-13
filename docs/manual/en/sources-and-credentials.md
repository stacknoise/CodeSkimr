# Sources and credentials

## Sources

| Source | How |
|---|---|
| Folder | an absolute path; scanned in place. A watch mode exists (`codeskimr scan <folder> --watch`). |
| Git repository | an address (`https://…`, `ssh://…`, `git@host:org/repo`, or ending in `.git`) and a branch (default `main`). CodeSkimr makes a shallow clone (depth 1) of that one branch into its own cache folder `git/` in the data folder and scans it. A second scan updates the copy; if the update fails, the old copy stays. |

A Git source cannot be watched (`CLI-102`).

## Secrets never go into an address

An address with a user name and password, a token, a query or a fragment (for example `https://user:token@host/repo` or `…?token=abc`) is **refused with `PRJ-101`**, with a second line that names the reason. The database would store it and the program would print it. A bare SSH user name (`git@host:org/repo`) is fine. Cache folders are named by a hash, so the address does not appear in a path or an error either.

## Credentials

For a repository that needs a login you create a **credential**: a user name and a secret (a password or a token). It is stored in the keystore of your operating system (Windows Credential Manager, macOS Keychain, Linux Secret Service) under the service name `codeskimr`. The project stores only the name of the credential, never the secret.

- The secret is read from the keystore at the moment a request needs it, handed to the Git login and wiped from memory afterwards. Nothing is cached, logged or written to the database or the cache folder.
- A credential is only sent to the server the source points at. After a redirect to another host nothing is sent.
- Git refuses to send credentials over plain `http://`; use `https://` or SSH.
- Without a credential the credential helper of your system decides, as it does for `git` itself.

> **State:** the UI and command line to create, change and delete credentials come with work package WP-73 (issue #160). Until then no command stores a credential, and a private repository answers `SRC-401`. Public repositories and folders work without one. This chapter will describe the steps when WP-73 is in.

## Installation token

A second secret exists: the **installation token** (`csk_` and 64 hex digits). It protects everything that changes state or returns source text, and the MCP endpoint, so that other programs on your machine cannot use the local server. It is created at the first `codeskimr serve`, kept in the same keystore (entry `installation-token`), never shown by a command and never logged. `codeskimr open` gives the browser a one-time launch code (valid 60 seconds, usable once) in place of the token; the browser UI keeps the token in its tab only.

## Problems

| Code | What happened | What to do |
|---|---|---|
| `SRC-401` | the remote wants a login and none is available | create a credential ([state above](#credentials)), or check the system's Git credential helper |
| `SEC-201` | the named credential is not in the keystore | create it again |
| `SEC-401` | the remote rejected the credential | check user name and secret; tokens expire |
| `SEC-402` | the keystore denied access | allow access in the system prompt |
| `SEC-601` | no keystore available | on Linux start a Secret Service (for example GNOME Keyring); `serve` needs it |
| `SRC-202` | the branch does not exist on the remote | check the name with `--branch` |
| `SRC-601` | network or remote error | try again later; check the connection |
