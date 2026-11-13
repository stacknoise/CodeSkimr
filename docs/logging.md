# Logging and secret redaction (WP-72, blueprint S-01)

`codeskimr` writes its log with `tracing` to **standard error**. Standard output stays free for command results and the MCP protocol.

## Level

The environment variable `CODESKIMR_LOG` sets the level: `error`, `warn` (default), `info`, `debug`, `trace` or `off`. A value that is not understood falls back to `warn`.

## Redaction

Every log line passes through `RedactingWriter` (`crates/app/src/log_writer.rs`) before it leaves the process, so it does not matter whether a secret sits in the message, in a field or in the text of an error. The masking lives in `crates/app/src/redact.rs`, a pure function `redact(&str) -> String`. It masks:

| What | Example | Result |
|---|---|---|
| User info in URLs | `https://user:tok@host/r.git` | `https://host/r.git` (reuses `codeskimr_source::redact_credentials`) |
| `Authorization` and `Cookie` values, to the end of the line or the closing quote | `Authorization: Bearer abc` | `Authorization: [redacted]` |
| Bearer tokens anywhere | `sent Bearer abc` | `sent Bearer [redacted]` |
| Values of fields named `password`, `passwd`, `secret`, `token`, `api_key`, `api-key`, `apikey`, `private_key`, `credential`, `session_id` (also inside `access_token` and similar names) | `?access_token=abc&page=2` | `?access_token=[redacted]&page=2` |
| Well-known token formats (GitHub, GitLab, Slack, AWS, Google API keys, JWT, `sk-` keys) with at least 16 characters after the prefix | `ghp_…` | `[redacted]` |

Masking errs on the safe side: a value that only looks like a secret is hidden as well, and text such as `token: expired` loses its value. Applying `redact` twice gives the same result.

## Panics

`logging::init` replaces the default panic hook. A panic is logged as an `ERROR` event with `code=APP-901`, the location and the message, and therefore goes through the same redaction. With `CODESKIMR_LOG=off` a panic prints nothing.

## Limits

Redaction recognises shapes, not values. A secret in an unknown shape (a random password inside an ordinary sentence) is not found. This is why secrets never reach a log in the first place (blueprint 12.4: no `format!`, `SecretString` never prints its content); the redaction layer is the second line of defence.

## Tests

- `redact.rs`: canaries in every shape, ordinary text stays unchanged, idempotence.
- `log_writer.rs`: a secret split over several writes is masked.
- `logging.rs`: canaries in messages, fields, `Debug` output and a panic never reach the output of a real subscriber.
- `tests/commands.rs`: the log level does not disturb standard output.

The full canary suite S-02 (REST/MCP responses, the database file, crash output of a child process) belongs to the security role and builds on these tests.
