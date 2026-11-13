# Source adapter (`codeskimr_source`)

Implements the driven ports `SourceProvider` and `FileDiscovery` of `codeskimr_application` and the file watcher (blueprint section 7).

| Part | What it does |
|---|---|
| `SourceFetcher` | Folder source: checks that the folder is readable (`SRC-201`). Git source: shallow clone (depth 1) of one branch into the cache folder. |
| `GitCache` | One folder per URL and branch, named by a hash (the URL never appears in a path or an error). An update clones into a private work folder (unique per call, so parallel fetches do not clash) and swaps: the old copy is parked, the new one moved in, the old one deleted last; on any failure the old copy is restored and the work folder removed. A cancelled token stops the fetch before and during the clone. Errors: `SRC-202` branch missing, `SRC-401` credentials needed, `SRC-601` anything else. HTTPS uses the platform TLS stack (`native-tls`) because the rustls variant pulls `webpki-root-certs` (licence CDLA-Permissive-2.0, not allowed in `deny.toml`). |
| `WalkDiscovery` | Parallel walk with `.gitignore` (also outside Git repositories), skips `scan.excluded_folders`, detects the language by extension through a `LanguageMap`, hashes content with BLAKE3, marks files whose hash differs from the previous snapshot. The size limit is checked on the file sizes before any content is read (`SRC-501`; files of unknown languages do not count). The result does not depend on the machine: only `.gitignore` files inside the scanned folder count, hidden files are scanned, `.git` is never entered. A file that cannot be read fails the run with `SRC-201`, because dropping it would look like a deletion. On cancellation the files found so far come back as `Ok`; the caller checks its token. |
| `LanguageMap` | Extension to language id. Filled by the composition root from the installed language packs; this crate names no language. |
| `watch` | Batches of changed relative paths (`WatchBatch`). A batch is due after a quiet window (`debounce`) but at latest `max_wait` after its first event, so constant writes cannot starve the listener. Access events and excluded folders are ignored before the timer is touched; lost events or watch errors set `rescan`. Dropping the handle delivers what is pending and joins the thread, so never drop it inside the callback. Errors: `SRC-602`. |

## Credentials (WP-71)

- A Git source may name a credential (`CredentialId`). `SourceFetcher<S: CredentialStore>` then lends it to the login callback of the connection: the store is asked again for every request of the connection and nothing is cached. Without a credential the system's Git credential helper decides.
- Secrets never go into the URL. A password in a Git URL is dropped before Git sees it (Git would write the URL into `.git/config`, issue #35); the user name stays. Tests clone with a canary secret and search the whole cache folder for it.
- Git refuses to send credentials over plain `http://`; this stays as it is. Only the test build switches it off (`gix-transport` feature `http-client-insecure-credentials` as a dev dependency) to talk to a local server.
- Errors: `SEC-201` the credential is not in the store, `SEC-401` the remote rejected it, `SEC-402` and `SEC-601` the keystore cannot be used. `SRC-401` stays for remotes that want credentials when none was configured.

## Limits of this package

- Files that are deleted since the previous snapshot are not listed by `FileDiscovery`; the pipeline can derive them from `previous`. Reporting them to the store is issue #20.
- `watch` is not yet connected to the event bus (`FilesChanged`, issue #21).
- The size limit and `max_wait` are constructor values; configuration keys for them are issue #28.
- An update is a full shallow re-clone, not a fetch.
