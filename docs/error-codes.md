<!-- Generated from errors/error-codes.toml by tools/codegen. Do not edit. -->

# Error code catalogue

Codes have the form `<PREFIX>-<C><NN>` (blueprint section 11). User-facing messages live in `locales/<lang>/errors.ftl` as `error-<id>`.

## APP – Application start and lifecycle

Owner: `codeskimr`

| Code | Name | Severity | Retryable | Since | Description |
|---|---|---|---|---|---|
| APP-201 | `data_dir_unavailable` | error | no | 0.1.0 | The user data directory cannot be created or opened. |
| APP-202 | `launch_file_unavailable` | error | yes | 0.1.0 | The private file for starting the browser could not be created. |
| APP-301 | `instance_already_running` | error | no | 0.1.0 | Another CodeSkimr instance holds the workspace; connecting to it failed. |
| APP-901 | `unexpected_internal_error` | error | no | 0.1.0 | An unexpected internal error occurred (bug). |
| APP-601 | `browser_start_failed` | error | yes | 0.1.0 | The system did not start a browser for the launch file. |

## CFG – Configuration

Owner: `codeskimr_application`

| Code | Name | Severity | Retryable | Since | Description |
|---|---|---|---|---|---|
| CFG-101 | `config_invalid_value` | error | no | 0.1.0 | A configuration value is outside its allowed range or has the wrong type. |
| CFG-102 | `config_parse_failed` | error | no | 0.1.0 | A configuration file is not valid TOML. |
| CFG-103 | `config_unknown_key` | error | no | 0.1.0 | A configuration file or flag sets a key that does not exist. |

## PRJ – Projects and snapshots

Owner: `codeskimr_application`

| Code | Name | Severity | Retryable | Since | Description |
|---|---|---|---|---|---|
| PRJ-101 | `project_source_invalid` | error | no | 0.1.0 | The given source is neither an existing folder nor a valid Git URL. |
| PRJ-201 | `project_not_found` | error | no | 0.1.0 | No project with this id exists. |
| PRJ-301 | `project_already_registered` | error | no | 0.1.0 | A project with the same source is already registered. |
| PRJ-302 | `snapshot_in_progress` | error | no | 0.1.0 | A scan for this project is already running. |

## SRC – Sources: file system, Git, watcher

Owner: `codeskimr_source`

| Code | Name | Severity | Retryable | Since | Description |
|---|---|---|---|---|---|
| SRC-201 | `source_path_not_found` | error | no | 0.1.0 | The configured source folder does not exist or is not readable. |
| SRC-202 | `git_branch_not_found` | error | no | 0.1.0 | The requested branch does not exist on the remote. |
| SRC-401 | `git_auth_required` | error | no | 0.1.0 | The remote requires authentication and no usable credential is available. |
| SRC-501 | `source_too_large` | error | no | 0.1.0 | The source exceeds the configured size limit. |
| SRC-601 | `git_fetch_failed` | error | yes | 0.1.0 | Fetching from the Git remote failed (network or remote error). |
| SRC-602 | `watcher_failed` | error | no | 0.1.0 | The file-system watcher could not be started or stopped unexpectedly. |

## PRS – Parsing and language packs

Owner: `codeskimr_parsing`

| Code | Name | Severity | Retryable | Since | Description |
|---|---|---|---|---|---|
| PRS-101 | `file_encoding_unsupported` | error | no | 0.1.0 | A source file is not valid UTF-8/UTF-16 and was skipped. |
| PRS-201 | `language_pack_missing` | error | no | 0.1.0 | No language pack is installed for this file type. |
| PRS-301 | `language_pack_invalid` | error | no | 0.1.0 | A language pack's TOML or query files are invalid. |
| PRS-501 | `parse_timeout` | error | no | 0.1.0 | Parsing a single file exceeded the configured time limit. |

## RES – Reference resolution

Owner: `codeskimr_parsing`

| Code | Name | Severity | Retryable | Since | Description |
|---|---|---|---|---|---|
| RES-301 | `ambiguous_reference` | warning | no | 0.1.0 | A reference matches several symbols; it was stored as probable. |

## IDX – Search and read models

Owner: `codeskimr_storage`

| Code | Name | Severity | Retryable | Since | Description |
|---|---|---|---|---|---|
| IDX-301 | `read_model_stale` | info | no | 0.1.0 | A read model is older than the current snapshot and is being rebuilt. |

## STO – Persistence

Owner: `codeskimr_storage`

| Code | Name | Severity | Retryable | Since | Description |
|---|---|---|---|---|---|
| STO-301 | `schema_migration_failed` | error | no | 0.1.0 | Applying a database migration failed. |
| STO-302 | `transaction_conflict` | error | yes | 0.1.0 | A concurrent write conflicted; the operation can be retried. |
| STO-501 | `disk_full` | error | no | 0.1.0 | Not enough disk space to persist the snapshot. |
| STO-601 | `database_failed` | error | no | 0.1.0 | The database reported an error that is not covered by a more specific code. |
| STO-901 | `stored_data_invalid` | error | no | 0.1.0 | Data read from the database violates the domain model; this is a bug or a damaged database. |

## SEC – Credentials and security

Owner: `codeskimr_secrets`

| Code | Name | Severity | Retryable | Since | Description |
|---|---|---|---|---|---|
| SEC-201 | `credential_not_found` | error | no | 0.1.0 | The referenced credential does not exist in the keystore. |
| SEC-202 | `credential_damaged` | error | no | 0.1.0 | The keystore entry of the credential cannot be read; delete it and create the credential again. |
| SEC-401 | `credential_rejected` | error | no | 0.1.0 | The platform rejected the credential. |
| SEC-402 | `keystore_access_denied` | error | no | 0.1.0 | The OS keystore denied access. |
| SEC-601 | `keystore_unavailable` | error | no | 0.1.0 | The OS keystore service is not available. |
| SEC-901 | `memory_lock_failed` | error | no | 0.1.0 | Locking secret memory against swapping failed. |

## NET – Network and platforms (GitHub, GitLab ...)

Owner: `codeskimr_source`

| Code | Name | Severity | Retryable | Since | Description |
|---|---|---|---|---|---|
| NET-501 | `request_timeout` | error | yes | 0.1.0 | A request to an external platform timed out. |
| NET-502 | `rate_limited` | error | yes | 0.1.0 | The external platform rate-limited the request. |
| NET-601 | `oauth_flow_failed` | error | no | 0.1.0 | The OAuth device flow did not complete. |

## AUTH – Caller identity and permissions

Owner: `codeskimr_application`

| Code | Name | Severity | Retryable | Since | Description |
|---|---|---|---|---|---|
| AUTH-401 | `permission_denied` | error | no | 0.1.0 | The caller lacks the permission this action needs. |

## API – REST and WebSocket

Owner: `codeskimr_server`

| Code | Name | Severity | Retryable | Since | Description |
|---|---|---|---|---|---|
| API-101 | `request_invalid` | error | no | 0.1.0 | The REST request failed validation. |
| API-102 | `bind_address_not_local` | error | no | 0.1.0 | The server was asked to listen on an address that is not a loopback address. |
| API-201 | `command_not_found` | error | no | 0.1.0 | The REST request names a command or route that does not exist. |
| API-301 | `server_bind_failed` | error | no | 0.1.0 | The server could not listen on its address, for example because the port is taken. |
| API-401 | `token_missing_or_invalid` | error | no | 0.1.0 | A state-changing request lacks a valid installation token. |
| API-402 | `host_or_origin_rejected` | error | no | 0.1.0 | The Host or Origin header of the request is not one of this server. |
| API-403 | `launch_code_invalid` | error | no | 0.1.0 | The one-time launch code is wrong, already used or expired. |
| API-501 | `request_too_large` | error | no | 0.1.0 | The request body is larger than the server accepts. |
| API-601 | `server_not_reachable` | error | yes | 0.1.0 | The command needs a running local server and could not reach it. |

## MCP – MCP server

Owner: `codeskimr_mcp`

| Code | Name | Severity | Retryable | Since | Description |
|---|---|---|---|---|---|
| MCP-101 | `tool_arguments_invalid` | error | no | 0.1.0 | MCP tool arguments failed validation. |
| MCP-201 | `project_not_registered` | error | no | 0.1.0 | The requested path is not inside a registered project. |
| MCP-202 | `symbol_not_found` | error | no | 0.1.0 | No symbol with this id or qualified name exists in the newest snapshot. |
| MCP-203 | `project_not_scanned` | error | yes | 0.1.0 | The project has no snapshot yet; scan it first. |
| MCP-501 | `response_limit_exceeded` | info | no | 0.1.0 | The result was truncated to the configured limit. |

## CLI – Command line

Owner: `codeskimr_cli`

| Code | Name | Severity | Retryable | Since | Description |
|---|---|---|---|---|---|
| CLI-101 | `cli_arguments_invalid` | error | no | 0.1.0 | Command-line arguments are invalid. |
| CLI-102 | `watch_needs_folder` | error | no | 0.1.0 | Watch mode needs a local folder; a Git URL cannot be watched. |
| CLI-103 | `secret_input_invalid` | error | no | 0.1.0 | No secret could be read from the terminal or standard input, or it was empty. |

## I18N – Translations

Owner: `codeskimr_i18n`

| Code | Name | Severity | Retryable | Since | Description |
|---|---|---|---|---|---|
| I18N-201 | `message_key_missing` | error | no | 0.1.0 | A message key is missing for the active language (falls back to English). |

## EXP – Export

Owner: `codeskimr_application`

| Code | Name | Severity | Retryable | Since | Description |
|---|---|---|---|---|---|
| EXP-201 | `export_bundle_missing` | error | no | 0.1.0 | The built export page of the UI is missing or unusable. |
| EXP-601 | `export_write_failed` | error | no | 0.1.0 | The export file could not be written. |

## UI – Frontend

Owner: `web`

| Code | Name | Severity | Retryable | Since | Description |
|---|---|---|---|---|---|
| UI-201 | `api_unreachable` | error | no | 0.1.0 | The UI cannot reach the local server. |
| UI-901 | `unexpected_ui_error` | error | no | 0.1.0 | An uncaught exception was converted at a feature boundary (bug). |

## CHK – Convention-check findings (not failures)

Owner: `codeskimr_application`

| Code | Name | Severity | Retryable | Since | Description |
|---|---|---|---|---|---|
| CHK-101 | `literal_found` | error | no | 0.1.0 | A literal outside the allowed exceptions was found. |
| CHK-102 | `duplicate_block` | error | no | 0.1.0 | A code block duplicates existing code. |
| CHK-103 | `layer_violation` | error | no | 0.1.0 | A dependency points against the allowed direction. |
| CHK-104 | `function_too_long` | error | no | 0.1.0 | A function exceeds the configured length. |
| CHK-105 | `complexity_too_high` | error | no | 0.1.0 | A function exceeds the configured cognitive complexity. |
