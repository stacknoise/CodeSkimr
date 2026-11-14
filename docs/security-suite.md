# Security suite (WP-74)

Canary tests for blueprint rules S-02 (no secret leaves the process) and S-05 (no file outside the
project is read). They run in CI with the normal `test` job.

## What is tested

| Test | Surface | Proves |
| --- | --- | --- |
| `security_tests::no_canary_leaves_the_running_app` | REST and MCP over HTTP (valid, wrong and missing token, hostile paths, malformed input) | A credential canary (sent inside a Git address of `projects.add`) and the installation token appear in no answer and in no file of the data folder. Calls without or with a wrong token get `401`. A symlink inside the project that points outside is never served; the control shows that the class inside the project is. |
| `commands::check_conventions_never_reads_a_file_outside_the_project` | MCP stdio, `check_conventions` | `../`, absolute, `src/../../` and symlink paths are refused in a controlled way (`files_checked` 0, path listed as unmatched). |
| `commands::the_cli_check_never_reads_a_file_outside_the_current_folder` | CLI `check` | The same paths give exit code 1 and "No file matches". |

Each traversal test has a positive control (a class inside the project is found).

## Added with #176

| Test | Surface | Proves |
| --- | --- | --- |
| `security_http_tests::bad_tokens_hosts_and_origins_are_refused_and_the_event_stream_leaks_nothing` | REST, MCP over HTTP, event stream | Calls without or with a wrong or empty token get `401` on `projects.add` (changes something), `explorer.source` (source text, ADR 0032) and MCP, each with the right token as the control; a foreign or `null` `Host` or `Origin` gets `API-402`; no answer allows a cross-site read (no `access-control-allow-origin`, a preflight from a foreign site is refused); every answer carries the content security policy, `nosniff` and `cache-control`; the WebSocket refuses a foreign origin, and what it streams while a credential canary is sent contains no secret (its events carry ids and counts, so this is a guard against a future change, not proof of a present leak). |
| `commands::a_secret_in_a_git_address_stays_out_of_the_output_and_the_data_folder` | CLI `scan`, trace log | The canary in a Git address is in no output and no file of the data folder. |
| `commands::a_secret_sent_to_the_stdio_server_stays_out_of_its_log_and_the_data_folder` | MCP stdio, trace log | The canary sent in every place a client can (client name, query, paths, tool arguments) is not in the log or the data folder. |
| `commands::the_export_holds_no_secret_from_a_method_body_and_no_path` | CLI `export --html` | A secret in a method body, and the project and data folder paths, are not in the page. |
| `security_login_tests::a_stored_credential_reaches_the_remote_and_nothing_else` | REST, Git login | A credential stored over `credentials.set` is used for a login at a remote (the remote saw `Basic` for that user), and neither the secret nor the installation token is in any answer or file of the data folder; the remote never got the installation token. |

## Added with the level 2 runner (ADR 0033)

| Test | Surface | Proves |
| --- | --- | --- |
| `codeskimr_indexer` `the_environment_holds_no_secret_and_nothing_unlisted` | environment of an indexer | Canary values in variables named like tokens, passwords, keys and in `CODESKIMR_*`, and in a variable that no allow list names, are not in the environment of the indexer; a variable that a pack lists but whose name looks secret is dropped as well (S-02). |
| `a_failing_indexer_is_reported_with_its_redacted_output` | console output of an indexer | A canary in the console output is not in the error or its `Debug` text (S-01, S-02). |
| `a_program_inside_the_project_is_never_used` | program lookup | A copy of the indexer inside the project folder, on the `PATH`, is not started (S-05). |
| `the_project_file_cannot_switch_level_2_on_or_raise_its_limits`, `by_default_nothing_runs` | consent | The `level2` table of `.codeskimr.toml` has no effect. |
| `a_restricted_principal_may_neither_confirm_nor_start` | permissions | Without `confirm_indexer` or `run_indexer` the call gets `SEC-403` and nothing is written (S-06). |

Crash output is covered by the unit test `logging::tests::a_panic_is_reported_without_its_secret`:
the binary has no input that makes it panic, so the hook is tested in the process.

## Known gaps

- The tests use an in-memory keystore; the keystore of the operating system cannot run in CI.
- Among the REST commands and MCP tools only `check_conventions` takes file paths (`paths` in
  `ConventionsArgs`, `crates/mcp/src/args.rs`); the others take ids, which the symbol loop of the
  canary test walks. A new command with a path input needs its own traversal test.
- Symlinks are created on unix only; on Windows the symlink path is a plain missing file.
