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

## Known gaps

- The credential store of the test app is in memory and no REST or MCP path reads it, so the
  credential canary reaches the app only through the Git address; "not in the data folder" is
  therefore weak for stored credentials. Login with a real keystore cannot run in CI.
- No canary scan of MCP stdio answers, no log files and no crash output (panic tests live in
  `logging.rs`), no Host, Origin, CORS and CSP tests (S-04), no traversal tests for other API
  inputs, no SSE request and no HTML export (WP-75). These are tracked in issue #176.
- Symlinks are created on unix only; on Windows the symlink path is a plain missing file.
