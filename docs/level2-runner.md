# Level 2: running the indexers (ADR 0033, package 4)

`crates/adapters/indexer` (`codeskimr_indexer`) starts the indexer of a language as a child process and returns its SCIP index. The port is `IndexRunner` (`crates/application/src/ports/indexer.rs`), the rules of consent are in `crates/application/src/level2/`. The import of the index is `docs/level2-import.md`. Nothing calls the runner from the scan yet; that is the second package.

An indexer runs the build tools of the project, which is project code. So the runner is security sensitive, and everything below is tested (`crates/adapters/indexer/tests`, with a stand-in program instead of a real indexer, `crates/application/tests/level2.rs`).

## Who may start it (decisions 9 and 10)

- Level 2 runs only if the user enabled it for the project **in the user configuration (`level2.enabled`) or in the user's own project record** and **confirmed it once** (the record, table `level2_consent` of the database). `authorize_run` checks both.
- The `level2` table in the project's `.codeskimr.toml` is dropped before the files are merged (`Config::resolve`), so a scanned repository can neither enable level 2 nor raise a limit. Test: `the_project_file_cannot_switch_level_2_on_or_raise_its_limits`.
- Two permissions, `confirm_indexer` and `run_indexer`. Only the local owner holds them. A principal without them gets `SEC-403`. MCP, REST tokens, watch mode and CI have no way to confirm: only `level2::confirm` writes the record, and the screens and commands of the local owner are its only callers. Without the record they get `SEC-301`.
- **Bound to the source.** The confirmation stores the kind, location and branch the project had; the store reports `confirmed` only while the project still has them. A project that is pointed at another folder or repository (or an id that is assigned anew) is unconfirmed again. A Git source can change its content at the same place (a pull); that is covered by "confirmed once" and belongs in the confirmation text.
- **Proof, not convention.** `authorize_run` returns a `RunPermit` that only it can create, and `IndexJob` needs one. The runner calls `permit.recheck()`, which reads the consent again, right before it starts the process. A withdrawal in between gives `SEC-301`.

## What the runner does (decision 11)

1. **Program.** Looked up in the `PATH` of the application as read at start. Relative entries and entries inside the project are dropped from `PATH`, also for the indexer, so a repository cannot plant `dotnet` or `node` in its own folder. Never `npx`, never a download. On Windows `.exe`, `.cmd` and `.bat` are tried.
2. **Tools checked first.** The indexer and every tool of the toolchain are asked for their version in a neutral folder (not in the project: the `rustup` proxy and `dotnet` read files of the working folder). Missing or older than `min_version` gives `RES-201` with the tool names in the cause. `IndexRunner::check` does the same without running an indexer, for the tool check on load (decision 8, which another package builds).
3. **Environment.** Built from an allow list (`consts.rs` of the crate plus `env_allow` of the pack), never inherited. A name that contains `TOKEN`, `SECRET`, `PASSWORD`, `CREDENTIAL`, `API_KEY`, `PRIVATE_KEY` or `AUTH`, or starts with `CODESKIMR`, never passes, even if a pack lists it. `TMPDIR`, `TEMP` and `TMP` point into the output folder.
4. **Output.** A folder inside the data directory (`level2-…`), removed when the returned `IndexArtifact` is dropped, also when the run fails. The index goes to `{output}`.
5. **Process group.** Own process group on Unix, a job object on Windows (crate `process-wrap`, no `unsafe` in our code). On timeout, memory or output limit, cancel and at the end of every run the whole group is killed, so build servers and daemons that the build started do not stay.
6. **Limits.** `level2.timeout_seconds` (`RES-502`), `level2.max_memory_mb` for the resident memory of the whole process tree, polled every 250 ms (`RES-504`; crate `sysinfo`), `level2.max_output_bytes` for console plus files of the output folder (`RES-503`); a folder nested deeper than the measure goes counts as over the limit. A cancelled run is `PRJ-303`.
7. **Redaction.** The console output goes through the redaction function that the composition root passes in (`redact` of the app) first and is cut to its last 2 KiB afterwards, so a secret at the cut is masked whole; the same holds for the version output in `RES-201` (200 characters). It is logged or put in an error (`RES-601`) only like that. The index file must be a regular file, not a link. The runner logs nothing else of the output.

## Not done here

- The indexer builds in the project folder (`obj`, `bin`, `target`, `node_modules`); that is not counted against the output limit. Folders inside the project are removed from `PATH`, so a virtual environment inside the project (`.venv`) is not found for scip-python; the user activates it outside or installs the tools globally.
- The permit carries the folder of its project (`RunPermit::project_root`), so a permit cannot be used with another project's folder. The caller must take the id and the folder from the same project record.
- The network is not blocked (restoring dependencies needs it); that stays the main residual risk of decision 11 and the reason for the consent. A sandbox is a later decision.
- The memory limit is polled: a process that allocates very fast can exceed it for up to a quarter of a second.
- A test with the real `redact` of the app (Bearer line, URL with credentials, secret across the cut) comes with the wiring, because the indexer crate cannot depend on the app.
- Files and network: the indexer can read what the user can read and use the network. Tracked in a follow-up issue (sandbox, empty `HOME`).
- Confirming needs a screen or command of the local owner (the second package, with the wiring into the scan).

## Who starts indexers

Only the scan of the command line (`codeskimr scan`, first scan of watch mode) runs the second pass. `ProjectScanner::rescan`, which MCP and the UI use, builds its job without level 2 (a `Job` wants it only through `with_level2()`, which only the scan of the command line calls), so an AI agent or a browser client can never start the build tools of a project, even after the owner confirmed level 2. Later scans of watch mode are level 1 too. `mcp` and `serve` also run as `principal::remote_client()`, which holds neither `ConfirmIndexer` nor `RunIndexer`. A test in `crates/application/tests/scan.rs` pins all three. The confirmation is bound to source, location and branch, not to the commit (issue #230).
