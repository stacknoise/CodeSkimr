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

For a repository that needs a login you store a **credential**: a user name and a secret (a password or a token) under a name you choose, for example `work-github`. It lives in the keystore of your operating system (Windows Credential Manager, macOS Keychain, Linux Secret Service) under the service name `codeskimr`. A project only remembers the **name** of its credential, never the secret.

### Store a token or password

```
codeskimr credential set work-github --user alice
```

The secret is read from the terminal without echo. Without a terminal it is read from standard input, which suits scripts:

```
printf %s "$TOKEN" | codeskimr credential set work-github --user alice
```

There is deliberately **no option for the secret**, so it never lands in your shell history, a process list or an environment dump. An empty secret is refused (`CLI-103`). Running `set` again for the same name replaces the credential. Secrets from standard input are limited to 64 KiB.

### Sign in with GitHub or GitLab (no password)

```
codeskimr credential login work-github --provider github --client-id <id>
codeskimr credential login work-gitlab --provider gitlab --client-id <id> --host https://gitlab.example.com
```

This is the OAuth device flow: the command prints a web page and a short code, you open the page in your browser, enter the code and approve, and the command stores the token. You need the `--client-id` of an OAuth application registered at the platform; it is not a secret. `--host` selects a self-managed installation and must be an HTTPS address. The token gets the smallest scope that reading needs (GitHub `repo`, GitLab `read_repository`). If you do not finish in the browser the command fails with `NET-601`.

GitHub tokens from an OAuth application do not expire. GitLab tokens expire after two hours. `credential login` also keeps the refresh token the platform sends, in the keystore next to the token; when a Git source is rejected with `SEC-401`, CodeSkimr trades it for a new token once and repeats the fetch. If the platform refuses the refresh token (revoked, or you removed the application), run `credential login` again. A token that you stored with `credential set` has no refresh token and is never renewed.

### Remove a credential

```
codeskimr credential delete work-github
```

A name that does not exist is not an error. The name `installation-token` belongs to the local server and is refused.

### Use it

```
codeskimr scan https://github.com/org/private-repo --credential work-github
```

`--credential` only applies to Git sources; naming one for a folder is `PRJ-101`. Without `--credential` the credential helper of your system decides, as it does for `git` itself. **There is never a prompt on the terminal**, so a server or an MCP session cannot hang on a login question; if no login is available the scan fails with `SRC-401`.

### What CodeSkimr does with the secret

- It is read from the keystore at the moment a request needs it, handed to the Git login and wiped from memory afterwards. Nothing is cached, logged or written to the database or the cache folder.
- It is only sent to the server the source points at. After a redirect to another host nothing is sent.
- Git refuses to send credentials over plain `http://`; use `https://` or SSH.

> **State:** the browser UI has no form for credentials yet; create them on the command line.

## Installation token

A second secret exists: the **installation token** (`csk_` and 64 hex digits). It protects everything that changes state or returns source text, and the MCP endpoint, so that other programs on your machine cannot use the local server. It is created at the first `codeskimr serve`, kept in the same keystore (entry `installation-token`), never shown by a command and never logged. `codeskimr open` gives the browser a one-time launch code (valid 60 seconds, usable once) in place of the token; the browser UI keeps the token in its tab only.

## Problems

| Code | What happened | What to do |
|---|---|---|
| `SRC-401` | the remote wants a login and none is available | create a credential ([see above](#credentials)) and pass `--credential`, or check the system's Git credential helper |
| `SEC-201` | the named credential is not in the keystore | create it with `credential set` or `credential login` |
| `SEC-202` | the keystore entry is damaged | `credential delete` it and create it again |
| `NET-601` | the browser sign-in did not finish | run `credential login` again and approve in time |
| `CLI-103` | no secret could be read, or it was empty | pipe the secret in or type it at the prompt |
| `SEC-401` | the remote rejected the credential (GitHub also answers an invalid token for a hidden repository this way) | check user name and secret and the rights of the token; tokens expire (GitLab OAuth tokens after two hours; a token from `credential login` is renewed automatically, otherwise log in again) |
| `SEC-402` | the keystore denied access | allow access in the system prompt |
| `SEC-601` | no keystore available | on Linux start a Secret Service (for example GNOME Keyring); `serve` needs it |
| `SRC-202` | the branch does not exist on the remote | check the name with `--branch` |
| `SRC-601` | network or remote error | try again later; check the connection |
