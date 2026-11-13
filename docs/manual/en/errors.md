# Error codes

Every failure CodeSkimr reports has a code of the form `PREFIX-CNN`, for example `SRC-401`. The prefix says where it happened, the first digit of the number says what kind it is:

| Digit | Class |
|---|---|
| `1` | invalid input |
| `2` | not found |
| `3` | conflict |
| `4` | not allowed (login, permission, token) |
| `5` | a limit was exceeded |
| `6` | an outside system failed (network, keystore, file system) |
| `9` | a bug; please report it |

## Where you see a code

- **Command line:** `error[SRC-401]: text` on standard error, in your language. Exit code 1 (2 for invalid arguments).
- **Browser:** the message of the screen, with the code.
- **REST:** `{ "code": "API-401", "description": "…" }` with the matching HTTP status.
- **MCP:** a tool result with `isError: true` and `code`, `description` (English) and `params`.

The code never changes; the text can improve. Quote the code when you report a problem.

## The published catalogue

The complete list of codes with name, severity, whether retrying helps, the version since which it exists and a description is generated from the registry `errors/error-codes.toml` and published in [`docs/error-codes.md`](../../error-codes.md). Do not edit it by hand.

## Codes you are likely to meet

| Code | Meaning | What to do |
|---|---|---|
| `PRJ-101` | the source is no existing folder and no valid Git address, or the address contains secrets or the folder is relative | give an absolute path; take secrets out of the address ([Sources and credentials](sources-and-credentials.md)) |
| `PRJ-201` | no such project | add and scan it first |
| `PRJ-302` | a scan of this project is already running | wait for it |
| `SRC-201` | the source folder or a file does not exist or cannot be read | check the path and rights |
| `SRC-401`, `SEC-*` | login to a Git remote or the keystore failed | see [Sources and credentials](sources-and-credentials.md#problems) |
| `SRC-501` | the source or a file is too large | raise `scan.max_source_mb` or exclude folders (`scan.excluded_folders`) |
| `PRS-201` | no language pack for a file type | the files of that type are not analysed; install the pack |
| `PRS-501` | a file took too long to parse | raise `scan.parse_timeout_ms` if the file is legitimate |
| `CFG-101`, `CFG-102`, `CFG-103` | invalid value, unreadable file, unknown key in the configuration | correct the setting named in the message |
| `API-301` | the port is taken | `--set server.port=…` |
| `API-401` | the installation token is missing or wrong | use `codeskimr open` for the browser |
| `API-402` | `Host` or `Origin` is not this server | open the UI at `http://127.0.0.1:<port>` or `localhost` |
| `API-601` | the command needs a running server | start `codeskimr serve` |
| `AUTH-401` | the caller lacks the permission | in single-user mode this should not happen; report it |
| `MCP-101` | invalid tool arguments; `params.detail` names the one | correct the call |
| `MCP-202` | symbol not found | take the id from `search_symbols` |
| `MCP-203` | the project has no snapshot | scan it |
| `MCP-501` | not an error: the result was cut to the limit | ask for less, or raise `limit` |
| `STO-501` | disk full | free space |
| `CLI-101` | invalid command-line arguments | see `codeskimr --help` |
| `*-901` | an unexpected internal error | report it with the code |

Convention findings (`CHK-101` to `CHK-105`) are results of `codeskimr check`, not failures of the program: a literal, a duplicate, a layer violation, a too long or too complex function ([`docs/conventions.md`](../../conventions.md)).
