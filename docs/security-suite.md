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
| `security_http_tests::the_guard_holds_on_every_call_and_the_event_stream_leaks_nothing` | REST, MCP over HTTP, event stream | Every call without or with a wrong token gets `401` (one by one); a foreign or `null` `Host` or `Origin` gets `API-402`; no answer allows a cross-site read (no `access-control-allow-origin`, a preflight from a foreign site is refused); every answer carries the content security policy, `nosniff` and `cache-control`; the WebSocket refuses a foreign origin, and what it streams while a credential canary is sent contains no secret. |
| `commands::a_secret_in_a_git_address_stays_out_of_the_output_and_the_data_folder` | CLI `scan`, trace log | The canary in a Git address is in no output and no file of the data folder. |
| `commands::a_secret_sent_to_the_stdio_server_stays_out_of_its_log_and_the_data_folder` | MCP stdio, trace log | The canary sent in every place a client can (client name, query, paths, tool arguments) is not in the log or the data folder. |
| `commands::the_export_holds_no_secret_from_a_method_body_and_no_path` | CLI `export --html` | A secret in a method body, and the project and data folder paths, are not in the page. |

Crash output is covered by the unit test `logging::tests::a_panic_is_reported_without_its_secret`:
the binary has no input that makes it panic, so the hook is tested in the process.

## Known gaps

- The credential store of the test app is in memory and no REST or MCP path reads it, so the
  credential canary reaches the app only through the Git address; "not in the data folder" is
  therefore weak for stored credentials. Login with a real keystore cannot run in CI.
- The only command that takes a file path is `check_conventions`; there is no other API input to
  test for traversal.
- Symlinks are created on unix only; on Windows the symlink path is a plain missing file.
