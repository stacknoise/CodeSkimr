# 0033 – Level 2 references come from SCIP indexes; level 1 keeps every match and marks it unchecked

Status: proposed

## Context

Level 1 (blueprint 9.2) resolves names without type information. The accuracy measurement (`tools/accuracy`, `docs/accuracy.md`, issues #134 and #145) shows that this does not get reliable: with 150 sampled symbols, 35 % (Python) and 23 % (TypeScript) have exactly the usages the language tooling finds, and the first sample on a private C# project had 8 of 20 correct. The purpose of CodeSkimr is documentation and judging extensions and reworks (`impact_of_change`). The product owner ranks missing usages above speed and above a few wrong ones, and wants to see when a result is not checked (13 November 2026).

Blueprint 9.3 plans level 2 "after 1.0, optional". This ADR moves it forward and fixes the rules.

## Options considered

- **A. Improve level 1 only.** Types from initializers, return types, chains. No toolchain needed, but it rebuilds a compiler piece by piece and never becomes reliable (estimate 70 to 85 % exact for typed languages, unmeasured).
- **B. Level 2 only, nothing without a toolchain.** Accurate where it runs, nothing (or the old wrong answers) elsewhere.
- **C. Level 2 plus an honest level 1 (chosen).** Level 2 from indexers where the toolchain exists; level 1 keeps every match and marks it as unchecked.
- **D. Ship the indexers with CodeSkimr (rejected, 13 November 2026).** The indexers and their runtimes could be bundled, but the packs are large (measured: scip-dotnet tool 264 MB and a .NET 8 SDK 479 MB; Node 120 MB; a JDK about 290 MB; about 1 GB for all languages) and licenses and updates would be ours to manage. More important, bundling does not remove what the project itself needs: its dependencies (NuGet restore, `node_modules`, a Python environment, Cargo dependencies, the Gradle or Maven build) and a matching SDK (in the spike, `scip-dotnet` ran in 9 s but the restore failed with `NETSDK1045` because the public C# corpus targets a newer .NET than the installed SDK 8). The user must provide these anyway, so the user installs the indexers too and CodeSkimr checks and reports what is missing.

## Decision

### Source and mapping

1. **Level 2 imports SCIP indexes.** Indexers (to be confirmed by the spike): C# scip-dotnet (Roslyn), TypeScript and JavaScript scip-typescript, Python scip-python, Rust `rust-analyzer scip`, Java and Kotlin scip-java. Only **installed, configured and version-checked** indexers are run. CodeSkimr never downloads, installs or runs a package from the network (no `npx`); the user installs the indexers. Go and the basic-tier languages have no level 2 and stay on level 1. A project with several languages indexes each language on its own.
2. **Mapping.** An occurrence maps to a CodeSkimr symbol by file and the range of its definition. Rules for partial classes, generated code and overloads are fixed in the import package; a mapping rate below a configured threshold (`level2.min_mapped`) makes the index count as failed for that language. Occurrences that map to no project symbol are dropped and counted in the scan summary.
3. **Replacement per file (change to blueprint 9.3).** For every file the index covers, level 2 references replace **all** level 1 references of that file, not only those of the same span, because level 1 may hold wrong matches in the same file. They are `certain`. Files without an index keep level 1.
4. **The index is untrusted input.** Paths are canonicalised against the project root (blueprint S-05), indexes above `level2.max_index_bytes` are refused, the decoder works on bounded buffers, and a corrupt or unreadable index is an error (new code) that leaves level 1 in place. It never panics.
5. **`scip.proto` is generated at build time** from a pinned copy that is checked in under the import crate; the decoding library is one new dependency (label `new-dependency`, decided in the import package).

### Level 1 and what the user sees

6. **Level 1 never drops a candidate.** This reverses the ambiguity rule of PR #133 (`resolve/lookup.rs::is_ambiguous`), which lowered the recall in TypeScript from 61 % to 42 %. The same package changes `docs/references.md`, blueprint 9.2, and the tests of #133 in `resolve.rs` and `resolve_csharp.rs`. It is done first, before any level 2 work.
7. **Unchecked results are marked.** A reference carries `verified`: `true` if level 2 produced it, `false` otherwise. Level 1 keeps `certain` and `probable` as before; `verified = false` is shown as "unchecked" (wording in the UI package, de and en). `find_usages`, `impact_of_change`, REST and UI count and show verified and unverified results separately and sort verified first. Without level 2, precision falls again (TypeScript from 58 % to about 28 % by the measurement), which is why the separate count and order are required. An analysis without level 2 for the files involved is labelled preliminary.
8. **Tools are checked whenever a project is loaded.** On every load (registration, scan, reopen) CodeSkimr checks per language of the project whether the indexer and its toolchain exist and have a usable version. The result is stored with the project and shown: which languages have level 2 ready, and for each missing tool its name, what it is needed for and how to install it (UI, `codeskimr scan` summary, MCP `get_project`). A missing tool never stops a scan. The check runs no project code.

### Running an indexer

9. **Opt-in, trusted configuration only.** Level 2 runs only if the user enabled it for the project in the **user configuration or the user's own project record** and confirmed it once. A `level2` key in the project's `.codeskimr.toml` is ignored (blueprint 14 lets the project file override the user file; this key is the exception), so a scanned repository cannot switch on running its own build scripts.
10. **Who may start it.** Only the local owner. Confirming and starting an indexer need their own `Permission` in the registry; MCP, REST tokens, watch mode and CI never stand in for the confirmation. A restricted principal gets `SEC-…` (new code); a test covers it. The team server of ADR 0019 does not run indexers until a later ADR says so.
11. **Limits, all configuration keys with defaults in `config/default.toml`:** `level2.timeout_seconds`, `level2.max_memory_mb`, `level2.max_output_bytes`, `level2.max_index_bytes`, `level2.min_mapped`. The process starts in its own process group and is killed as a group on timeout or cancel. The environment is built from an allow list (no credentials, no tokens of the application), output goes to a temporary directory inside the data directory, and the output of the indexer is redacted before it is logged. Network: not blocked in the first version (restoring dependencies needs it), which is the main residual risk and the reason for the consent. A sandbox (container or OS-level) is decided in the security review of the runner package.
12. **The runner is security-sensitive.** The package that starts indexers carries `security-sensitive` and gets a security review.

### Pipeline

13. **A second pass, not part of Resolve.** Level 1 runs inside the scan as today (ADR 0013) and the snapshot is complete and usable. Level 2 is an `IndexRunner` port (new) used by a second pass that writes into the same snapshot in one transaction per language and replaces references per file (decision 3). Cancellation stops the process and leaves level 1 unchanged. An index counts as current for a file if the content hash of the file equals the one at indexing time; a changed file (incremental scan, watch mode) falls back to level 1 and is marked unchecked until level 2 runs again, which happens on the next scan or on demand. The statement "no other component changes" of blueprint 9.3 is therefore replaced: new are the `IndexRunner` port, a consent store, the configuration keys above, the permissions and the error codes (with de and en texts).

### Acceptance

14. **Acceptance** (added to `docs/acceptance`, replacing the 98 % of blueprint 17.2 with the numbers below):
   - Loading a project whose tool is missing shows the missing tool and the install hint in UI, CLI summary and MCP, and the results carry the unchecked mark (test with a project whose indexer is not installed).
   - With level 2, per language with an indexer: recall at least 99 % and at least 99 % correct among verified references on at least 150 sampled symbols. The reference is **independent of level 2**: the language tooling of `tools/accuracy` (Jedi, TypeScript language service, rust-analyzer, javac, Roslyn), not the SCIP index. The private C# sample of 20 symbols is complete and without wrong usages for **all 20** (an individual miss must be explained and fixed or recorded as an issue with the reason).
   - Without level 2: recall at least as high as before PR #133, measured for every language with a reference tool (first Python and TypeScript, then Rust and Java).
   - `tools/accuracy` is extended to those languages and run by the acceptance, not by CI.

### Order of work

15. (1) Level 1 keeps all matches and `verified` (decision 6 and 7). (2) Tool check on load (decision 8). (3) Spike: indexers on the public corpora, C# and Rust first. (4) `IndexRunner`, consent, permissions, limits. (5) Import and mapping. (6) Per language: C#, Rust, TypeScript, Python, Java and Kotlin.

## Implementation (package 5, PR #216)

The import package deviates from the text above in three points:

- **Decoder (decision 5).** Instead of generating code from a pinned `scip.proto` with a decoding library (a new dependency), the crate has a small hand-written reader that knows only the fields it needs and skips everything else (`crates/adapters/scip/src/wire.rs`, `decode.rs`). It has bounded buffers, a depth limit for skipped groups, strict varints and limits on documents, occurrences and distinct symbols. The pinned copy is `crates/adapters/scip/scip-used.proto`: the messages and field numbers that are read, which must equal `src/consts.rs`. No new dependency, no `protoc`, a smaller attack surface.
- **Measure of `level2.min_mapped` (decision 2).** It is `mapped / (mapped + unmapped)` among the uses of project symbols, where a project symbol is one that the index itself defines. Library symbols are counted apart and do not enter the share, because about half of all occurrences of real projects point into libraries (measured on C# and Rust: 50 to 51 %), so counting them would fail every real index at 0.5. An index in which uses exist but none maps, for example because the indexer never marks definitions, fails as well, and so does an index that covers no file of the snapshot.
- **Columns.** Occurrence columns are stored in the unit of the index and are not converted to UTF-8 bytes yet (issue #217).

## Consequences

- Blueprint changes (by the pull request that implements packages 1 and 5, with the label `blueprint-change`): sections 1 and 7 (scope and levels), 9.2 to 9.4, 14 (the `level2` exception and keys), 16.1 (time of a level 2 scan), 17.2 (acceptance numbers) and 20.2 (roadmap: level 2 before 1.0); `docs/usages.md`, `docs/mcp.md`, `docs/references.md`.
- Users need the toolchain of the language for full accuracy; without it they get level 1, marked.
- Running build tools is a new security-relevant function with consent, limits and review.
- New error codes with texts in de and en (index corrupt, tool missing, permission, limit exceeded), registered in `errors/error-codes.toml`.
- The reversal of #133 costs precision without level 2; the marking and the separate count carry that.

## Addendum: decisions made while wiring level 2

- **Who starts indexers.** Decision 10 forbids that MCP, REST, watch mode and CI stand in for the confirmation. Beyond that, after the owner confirmed, only the scan of the command line (`codeskimr scan`, the first scan of `--watch`) runs the second pass. `ProjectScanner::rescan` (MCP, UI) and later watch scans are level 1 only, so an AI agent or a browser client cannot start the build tools of a project. A `Job` wants level 2 only if it was built `with_level2()` (the default is off); only `Session::scan` of the command line does that, and a test pins that direction. `mcp` and `serve` also run under their own principal (`principal::remote_client()`) that holds neither `ConfirmIndexer` nor `RunIndexer`, so the check does not rest on the job flag alone (a test confirms that it gets `SEC-403`).
- **Where confirming lives.** Only `codeskimr level2 confirm|revoke` calls `level2::confirm`/`revoke`, on a terminal, after a text that names file and network access, the missing sandbox and the binding to source, location and branch. The port enforces it technically: `ConsentStore::store_consent` takes a `ConsentGrant`, which only `level2::confirm`/`revoke` can make, and only after they checked `Permission::ConfirmIndexer`; holding the store gives no way to write a consent. A test in `crates/app/tests/consent_origin.rs` is the second layer and fails if another product file names the functions, the grant or `store_consent` in any spelling (path, `use`, braces, alias, glob, re-export, several lines).
- **What the user confirms.** The prompt shows the name, the location (folder or Git address) and the branch of the project that the argument resolved to (also for a numeric id), and the user types the name of the project instead of a fixed word, so a pasted or habitual `yes` never confirms. After the answer the project is read again; if its name or source changed in the meantime, nothing is stored (`SEC-403`). The storage binds the confirmation to the source it has at the moment of writing. A name, location or branch that holds a control character (line break, escape sequence) or an invisible or direction character (U+202A-202E, U+2066-2069, zero-width and similar) is never shown: the command fails with `SEC-403` without asking. A Git URL as argument resolves only the default branch; another branch is addressed by its id.
- **Residual risk: pseudo-terminal.** The terminal check (`is_terminal` on standard input and output) cannot tell a person from a program that drives a pseudo-terminal (for example an agent that runs a shell on the owner's account). Such a program already has the rights of the owner's user account and could also edit the database or the configuration directly, so the check protects against accidents and against MCP, REST and CI, not against code that runs as the owner. Typing the project name makes an unintended confirmation harder but not impossible for such a program. The confirmation is not bound to a commit; a later pull of new build scripts runs under it (issue #230).
- **Run folder.** `authorize_run` takes the folder from the project record (folder source) or the clone folder of the source adapter (Git), never from the caller.
- **Replacing across snapshots.** Rows are shared between snapshots (ADR 0032), so a replacement also updates the totals and read models of every snapshot that holds the file.
- **Usages answer.** `level2` is `active` if at least one direct usage of the answer is verified, otherwise `unavailable`.
