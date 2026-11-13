# Security suite (WP-74)

Canary tests for blueprint rules S-02 (no secret leaves the process) and S-05 (no file outside the
project is read). They run in CI with the normal `test` job.

## What is tested

| Test | Surface | Proves |
| --- | --- | --- |
| `security_tests::no_canary_leaves_the_running_app` | REST, MCP over HTTP, SSE, error answers, data folder | A credential canary and the installation token appear in no answer and in no file of the data folder; the controls show that the calls really ran. |
| `commands::check_conventions_never_reads_a_file_outside_the_project` | MCP stdio, `check_conventions` | A symlink inside the project that points outside is not followed; the outside class is never reported. |
| `commands::the_cli_check_never_reads_a_file_outside_the_current_folder` | CLI `check` | Same for the command line. |

Each traversal test has a positive control (a class inside the project is found), so a green run
cannot come from a call that did nothing.

## Known gaps

- Credential login with a canary cannot run in-process: `wiring::build_scanner` uses the OS
  keystore, which CI does not have. The secrets crate tests cover the store itself.
- Panic output is covered by the tests in `logging.rs`; the binary has no way to trigger a panic.
- HTML export (WP-75) does not exist yet. Its canary check is a follow-up once it lands.
