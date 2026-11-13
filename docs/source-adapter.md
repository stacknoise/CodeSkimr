# Source adapter (`codeskimr_source`)

Implements the driven ports `SourceProvider` and `FileDiscovery` of `codeskimr_application` and the file watcher (blueprint section 7).

| Part | What it does |
|---|---|
| `SourceFetcher` | Folder source: checks that the folder is readable (`SRC-201`). Git source: shallow clone (depth 1) of one branch into the cache folder. |
| `GitCache` | One folder per URL and branch, named by a hash (the URL never appears in a path or an error). An update clones next to the old copy and swaps, so a failed or cancelled update keeps the old copy. Errors: `SRC-202` branch missing, `SRC-401` credentials needed, `SRC-601` anything else. HTTPS uses the platform TLS stack (`native-tls`) because the rustls variant pulls `webpki-root-certs` (licence CDLA-Permissive-2.0, not allowed in `deny.toml`). |
| `WalkDiscovery` | Parallel walk with `.gitignore` (also outside Git repositories), skips `scan.excluded_folders`, detects the language by extension through a `LanguageMap`, hashes content with BLAKE3, marks files whose hash differs from the previous snapshot. Stops with `SRC-501` when the summed size passes the limit. |
| `LanguageMap` | Extension to language id. Filled by the composition root from the installed language packs; this crate names no language. |
| `watch` | Debounced batches of changed relative paths; ignores excluded folders. Errors: `SRC-602`. |

## Limits of this package

- Credentials: Git access uses what the Git library finds (credential helper). The loan pattern and keystore belong to the secrets package (blueprint section 12).
- Files that are deleted since the previous snapshot are not listed by `FileDiscovery`; the pipeline can derive them from `previous`. Reporting them to the store is issue #20.
- `watch` is not yet connected to the event bus (`FilesChanged`, issue #21).
- The size limit is a constructor value; a configuration key for it is a separate issue.
