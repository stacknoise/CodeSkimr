# Source adapter (`codeskimr_source`)

Implements the driven ports `SourceProvider` and `FileDiscovery` of `codeskimr_application` and the file watcher (blueprint section 7).

| Part | What it does |
|---|---|
| `SourceFetcher` | Folder source: checks that the folder is readable (`SRC-201`). Git source: shallow clone (depth 1) of one branch into the cache folder. |
| `GitCache` | One folder per URL and branch, named by a hash (the URL never appears in a path or an error). An update clones into a private work folder (unique per call, so parallel fetches do not clash) and swaps: the old copy is parked, the new one moved in, the old one deleted last; on any failure the old copy is restored and the work folder removed. A cancelled token stops the fetch before and during the clone. Errors: `SRC-202` branch missing, `SRC-401` credentials needed, `SRC-601` anything else. HTTPS uses the platform TLS stack (`native-tls`) because the rustls variant pulls `webpki-root-certs` (licence CDLA-Permissive-2.0, not allowed in `deny.toml`). |
| `WalkDiscovery` | Parallel walk with `.gitignore` (also outside Git repositories), skips `scan.excluded_folders`, detects the language by extension through a `LanguageMap`, hashes content with BLAKE3, marks files whose hash differs from the previous snapshot. The size limit is checked on the file sizes before any content is read (`SRC-501`; files of unknown languages do not count). The result does not depend on the machine: only `.gitignore` files inside the scanned folder count, hidden files are scanned, `.git` is never entered. A file that cannot be read fails the run with `SRC-201`, because dropping it would look like a deletion. On cancellation the files found so far come back as `Ok`; the caller checks its token. |
| `LanguageMap` | Extension to language id. Filled by the composition root from the installed language packs; this crate names no language. |
| `watch` | Batches of changed relative paths (`WatchBatch`). A batch is due after a quiet window (`debounce`) but at latest `max_wait` after its first event, so constant writes cannot starve the listener. Access events and excluded folders are ignored before the timer is touched; lost events or watch errors set `rescan`. Dropping the handle delivers what is pending and joins the thread, so never drop it inside the callback. Errors: `SRC-602`. |

## Limits of this package

- Credentials: Git access uses what the Git library finds (credential helper). The loan pattern and keystore belong to the secrets package (blueprint section 12).
- Files that are deleted since the previous snapshot are not listed by `FileDiscovery`; the pipeline can derive them from `previous`. Reporting them to the store is issue #20.
- `watch` is not yet connected to the event bus (`FilesChanged`, issue #21).
- The size limit comes from `scan.max_source_mb` (default 2048, WP-25, closes #28); `max_wait` is still a constructor value.
- Credentials inside a Git URL are not written to errors, but Git stores the URL in `.git/config` of the cached clone; avoiding that belongs to the secrets package.
- An update is a full shallow re-clone, not a fetch.
