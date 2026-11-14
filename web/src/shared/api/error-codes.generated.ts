// Generated from errors/error-codes.toml by tools/codegen. Do not edit.

/** Id of a registered error code, e.g. `SRC-201`. */
export type ErrorCode =
  | "APP-201"
  | "APP-202"
  | "APP-301"
  | "APP-901"
  | "APP-601"
  | "CFG-101"
  | "CFG-102"
  | "CFG-103"
  | "PRJ-101"
  | "PRJ-201"
  | "PRJ-301"
  | "PRJ-302"
  | "PRJ-303"
  | "SRC-201"
  | "SRC-202"
  | "SRC-401"
  | "SRC-501"
  | "SRC-601"
  | "SRC-602"
  | "PRS-101"
  | "PRS-201"
  | "PRS-301"
  | "PRS-501"
  | "RES-101"
  | "RES-201"
  | "RES-301"
  | "RES-302"
  | "RES-501"
  | "RES-502"
  | "RES-503"
  | "RES-504"
  | "RES-601"
  | "IDX-301"
  | "STO-301"
  | "STO-302"
  | "STO-501"
  | "STO-601"
  | "STO-901"
  | "SEC-201"
  | "SEC-202"
  | "SEC-301"
  | "SEC-401"
  | "SEC-402"
  | "SEC-403"
  | "SEC-601"
  | "SEC-901"
  | "NET-501"
  | "NET-502"
  | "NET-601"
  | "AUTH-401"
  | "API-101"
  | "API-102"
  | "API-201"
  | "API-301"
  | "API-401"
  | "API-402"
  | "API-403"
  | "API-501"
  | "API-601"
  | "MCP-101"
  | "MCP-201"
  | "MCP-202"
  | "MCP-203"
  | "MCP-501"
  | "CLI-101"
  | "CLI-102"
  | "CLI-103"
  | "I18N-201"
  | "EXP-201"
  | "EXP-601"
  | "UI-201"
  | "UI-901"
  | "CHK-101"
  | "CHK-102"
  | "CHK-103"
  | "CHK-104"
  | "CHK-105"
;

/** Static data of one error code. */
export interface ErrorCodeInfo {
  id: ErrorCode;
  name: string;
  group: string;
  class: string;
  severity: "error" | "warning" | "info";
  retryable: boolean;
  description: string;
  since: string;
  deprecated: boolean;
  messageKey: string;
}

/** All registered error codes by id. */
export const ERROR_CODES: Record<ErrorCode, ErrorCodeInfo> = {
  "APP-201": { id: "APP-201", name: "data_dir_unavailable", group: "APP", class: "not_found", severity: "error", retryable: false, description: "The user data directory cannot be created or opened.", since: "0.1.0", deprecated: false, messageKey: "error-APP-201" },
  "APP-202": { id: "APP-202", name: "launch_file_unavailable", group: "APP", class: "not_found", severity: "error", retryable: true, description: "The private file for starting the browser could not be created.", since: "0.1.0", deprecated: false, messageKey: "error-APP-202" },
  "APP-301": { id: "APP-301", name: "instance_already_running", group: "APP", class: "conflict", severity: "error", retryable: false, description: "Another CodeSkimr instance holds the workspace; connecting to it failed.", since: "0.1.0", deprecated: false, messageKey: "error-APP-301" },
  "APP-901": { id: "APP-901", name: "unexpected_internal_error", group: "APP", class: "internal", severity: "error", retryable: false, description: "An unexpected internal error occurred (bug).", since: "0.1.0", deprecated: false, messageKey: "error-APP-901" },
  "APP-601": { id: "APP-601", name: "browser_start_failed", group: "APP", class: "external", severity: "error", retryable: true, description: "The system did not start a browser for the launch file.", since: "0.1.0", deprecated: false, messageKey: "error-APP-601" },
  "CFG-101": { id: "CFG-101", name: "config_invalid_value", group: "CFG", class: "invalid_input", severity: "error", retryable: false, description: "A configuration value is outside its allowed range or has the wrong type.", since: "0.1.0", deprecated: false, messageKey: "error-CFG-101" },
  "CFG-102": { id: "CFG-102", name: "config_parse_failed", group: "CFG", class: "invalid_input", severity: "error", retryable: false, description: "A configuration file is not valid TOML.", since: "0.1.0", deprecated: false, messageKey: "error-CFG-102" },
  "CFG-103": { id: "CFG-103", name: "config_unknown_key", group: "CFG", class: "invalid_input", severity: "error", retryable: false, description: "A configuration file or flag sets a key that does not exist.", since: "0.1.0", deprecated: false, messageKey: "error-CFG-103" },
  "PRJ-101": { id: "PRJ-101", name: "project_source_invalid", group: "PRJ", class: "invalid_input", severity: "error", retryable: false, description: "The given source is neither an existing folder nor a valid Git URL.", since: "0.1.0", deprecated: false, messageKey: "error-PRJ-101" },
  "PRJ-201": { id: "PRJ-201", name: "project_not_found", group: "PRJ", class: "not_found", severity: "error", retryable: false, description: "No project with this id exists.", since: "0.1.0", deprecated: false, messageKey: "error-PRJ-201" },
  "PRJ-301": { id: "PRJ-301", name: "project_already_registered", group: "PRJ", class: "conflict", severity: "error", retryable: false, description: "A project with the same source is already registered.", since: "0.1.0", deprecated: false, messageKey: "error-PRJ-301" },
  "PRJ-302": { id: "PRJ-302", name: "snapshot_in_progress", group: "PRJ", class: "conflict", severity: "error", retryable: false, description: "A scan for this project is already running.", since: "0.1.0", deprecated: false, messageKey: "error-PRJ-302" },
  "PRJ-303": { id: "PRJ-303", name: "scan_cancelled", group: "PRJ", class: "conflict", severity: "info", retryable: true, description: "The scan was cancelled by the user; no snapshot was written.", since: "0.1.0", deprecated: false, messageKey: "error-PRJ-303" },
  "SRC-201": { id: "SRC-201", name: "source_path_not_found", group: "SRC", class: "not_found", severity: "error", retryable: false, description: "The configured source folder does not exist or is not readable.", since: "0.1.0", deprecated: false, messageKey: "error-SRC-201" },
  "SRC-202": { id: "SRC-202", name: "git_branch_not_found", group: "SRC", class: "not_found", severity: "error", retryable: false, description: "The requested branch does not exist on the remote.", since: "0.1.0", deprecated: false, messageKey: "error-SRC-202" },
  "SRC-401": { id: "SRC-401", name: "git_auth_required", group: "SRC", class: "permission", severity: "error", retryable: false, description: "The remote requires authentication and no usable credential is available.", since: "0.1.0", deprecated: false, messageKey: "error-SRC-401" },
  "SRC-501": { id: "SRC-501", name: "source_too_large", group: "SRC", class: "limit", severity: "error", retryable: false, description: "The source exceeds the configured size limit.", since: "0.1.0", deprecated: false, messageKey: "error-SRC-501" },
  "SRC-601": { id: "SRC-601", name: "git_fetch_failed", group: "SRC", class: "external", severity: "error", retryable: true, description: "Fetching from the Git remote failed (network or remote error).", since: "0.1.0", deprecated: false, messageKey: "error-SRC-601" },
  "SRC-602": { id: "SRC-602", name: "watcher_failed", group: "SRC", class: "external", severity: "error", retryable: false, description: "The file-system watcher could not be started or stopped unexpectedly.", since: "0.1.0", deprecated: false, messageKey: "error-SRC-602" },
  "PRS-101": { id: "PRS-101", name: "file_encoding_unsupported", group: "PRS", class: "invalid_input", severity: "error", retryable: false, description: "A source file is not valid UTF-8/UTF-16 and was skipped.", since: "0.1.0", deprecated: false, messageKey: "error-PRS-101" },
  "PRS-201": { id: "PRS-201", name: "language_pack_missing", group: "PRS", class: "not_found", severity: "error", retryable: false, description: "No language pack is installed for this file type.", since: "0.1.0", deprecated: false, messageKey: "error-PRS-201" },
  "PRS-301": { id: "PRS-301", name: "language_pack_invalid", group: "PRS", class: "conflict", severity: "error", retryable: false, description: "A language pack's TOML or query files are invalid.", since: "0.1.0", deprecated: false, messageKey: "error-PRS-301" },
  "PRS-501": { id: "PRS-501", name: "parse_timeout", group: "PRS", class: "limit", severity: "error", retryable: false, description: "Parsing a single file exceeded the configured time limit.", since: "0.1.0", deprecated: false, messageKey: "error-PRS-501" },
  "RES-101": { id: "RES-101", name: "scip_index_invalid", group: "RES", class: "invalid_input", severity: "error", retryable: false, description: "A SCIP index cannot be read, is corrupt or names a path outside the project; level 1 references stay.", since: "0.1.0", deprecated: false, messageKey: "error-RES-101" },
  "RES-201": { id: "RES-201", name: "indexer_unavailable", group: "RES", class: "not_found", severity: "error", retryable: false, description: "The indexer of a language or its toolchain is not installed or is older than the version the language pack requires; level 1 references stay.", since: "0.1.0", deprecated: false, messageKey: "error-RES-201" },
  "RES-301": { id: "RES-301", name: "ambiguous_reference", group: "RES", class: "conflict", severity: "warning", retryable: false, description: "A reference matches several symbols; it was stored as probable.", since: "0.1.0", deprecated: false, messageKey: "error-RES-301" },
  "RES-302": { id: "RES-302", name: "scip_mapping_too_low", group: "RES", class: "conflict", severity: "error", retryable: false, description: "Too few occurrences of a SCIP index map onto project symbols; level 1 references stay.", since: "0.1.0", deprecated: false, messageKey: "error-RES-302" },
  "RES-501": { id: "RES-501", name: "scip_index_too_large", group: "RES", class: "limit", severity: "error", retryable: false, description: "A SCIP index is larger than the configured limit and was refused; level 1 references stay.", since: "0.1.0", deprecated: false, messageKey: "error-RES-501" },
  "RES-502": { id: "RES-502", name: "indexer_timeout", group: "RES", class: "limit", severity: "error", retryable: false, description: "An indexer ran longer than level2.timeout_seconds and was stopped with all its child processes; level 1 references stay.", since: "0.1.0", deprecated: false, messageKey: "error-RES-502" },
  "RES-503": { id: "RES-503", name: "indexer_output_too_large", group: "RES", class: "limit", severity: "error", retryable: false, description: "An indexer wrote more than level2.max_output_bytes and was stopped; level 1 references stay.", since: "0.1.0", deprecated: false, messageKey: "error-RES-503" },
  "RES-504": { id: "RES-504", name: "indexer_memory_limit", group: "RES", class: "limit", severity: "error", retryable: false, description: "An indexer used more memory than level2.max_memory_mb and was stopped; level 1 references stay.", since: "0.1.0", deprecated: false, messageKey: "error-RES-504" },
  "RES-601": { id: "RES-601", name: "indexer_failed", group: "RES", class: "external", severity: "error", retryable: false, description: "An indexer could not be started or ended with an error; level 1 references stay.", since: "0.1.0", deprecated: false, messageKey: "error-RES-601" },
  "IDX-301": { id: "IDX-301", name: "read_model_stale", group: "IDX", class: "conflict", severity: "info", retryable: false, description: "A read model is older than the current snapshot and is being rebuilt.", since: "0.1.0", deprecated: false, messageKey: "error-IDX-301" },
  "STO-301": { id: "STO-301", name: "schema_migration_failed", group: "STO", class: "conflict", severity: "error", retryable: false, description: "Applying a database migration failed.", since: "0.1.0", deprecated: false, messageKey: "error-STO-301" },
  "STO-302": { id: "STO-302", name: "transaction_conflict", group: "STO", class: "conflict", severity: "error", retryable: true, description: "A concurrent write conflicted; the operation can be retried.", since: "0.1.0", deprecated: false, messageKey: "error-STO-302" },
  "STO-501": { id: "STO-501", name: "disk_full", group: "STO", class: "limit", severity: "error", retryable: false, description: "Not enough disk space to persist the snapshot.", since: "0.1.0", deprecated: false, messageKey: "error-STO-501" },
  "STO-601": { id: "STO-601", name: "database_failed", group: "STO", class: "external", severity: "error", retryable: false, description: "The database reported an error that is not covered by a more specific code.", since: "0.1.0", deprecated: false, messageKey: "error-STO-601" },
  "STO-901": { id: "STO-901", name: "stored_data_invalid", group: "STO", class: "internal", severity: "error", retryable: false, description: "Data read from the database violates the domain model; this is a bug or a damaged database.", since: "0.1.0", deprecated: false, messageKey: "error-STO-901" },
  "SEC-201": { id: "SEC-201", name: "credential_not_found", group: "SEC", class: "not_found", severity: "error", retryable: false, description: "The referenced credential does not exist in the keystore.", since: "0.1.0", deprecated: false, messageKey: "error-SEC-201" },
  "SEC-202": { id: "SEC-202", name: "credential_damaged", group: "SEC", class: "not_found", severity: "error", retryable: false, description: "The keystore entry of the credential cannot be read; delete it and create the credential again.", since: "0.1.0", deprecated: false, messageKey: "error-SEC-202" },
  "SEC-301": { id: "SEC-301", name: "indexer_not_confirmed", group: "SEC", class: "conflict", severity: "error", retryable: false, description: "Level 2 is not enabled and confirmed for this project in the user configuration or the user's own project record, so no indexer is started.", since: "0.1.0", deprecated: false, messageKey: "error-SEC-301" },
  "SEC-401": { id: "SEC-401", name: "credential_rejected", group: "SEC", class: "permission", severity: "error", retryable: false, description: "The platform rejected the credential.", since: "0.1.0", deprecated: false, messageKey: "error-SEC-401" },
  "SEC-402": { id: "SEC-402", name: "keystore_access_denied", group: "SEC", class: "permission", severity: "error", retryable: false, description: "The OS keystore denied access.", since: "0.1.0", deprecated: false, messageKey: "error-SEC-402" },
  "SEC-403": { id: "SEC-403", name: "indexer_not_permitted", group: "SEC", class: "permission", severity: "error", retryable: false, description: "The caller may not confirm or start indexers; only the local owner may (ADR 0033, decision 10).", since: "0.1.0", deprecated: false, messageKey: "error-SEC-403" },
  "SEC-601": { id: "SEC-601", name: "keystore_unavailable", group: "SEC", class: "external", severity: "error", retryable: false, description: "The OS keystore service is not available.", since: "0.1.0", deprecated: false, messageKey: "error-SEC-601" },
  "SEC-901": { id: "SEC-901", name: "memory_lock_failed", group: "SEC", class: "internal", severity: "error", retryable: false, description: "Locking secret memory against swapping failed.", since: "0.1.0", deprecated: false, messageKey: "error-SEC-901" },
  "NET-501": { id: "NET-501", name: "request_timeout", group: "NET", class: "limit", severity: "error", retryable: true, description: "A request to an external platform timed out.", since: "0.1.0", deprecated: false, messageKey: "error-NET-501" },
  "NET-502": { id: "NET-502", name: "rate_limited", group: "NET", class: "limit", severity: "error", retryable: true, description: "The external platform rate-limited the request.", since: "0.1.0", deprecated: false, messageKey: "error-NET-502" },
  "NET-601": { id: "NET-601", name: "oauth_flow_failed", group: "NET", class: "external", severity: "error", retryable: false, description: "The OAuth device flow did not complete.", since: "0.1.0", deprecated: false, messageKey: "error-NET-601" },
  "AUTH-401": { id: "AUTH-401", name: "permission_denied", group: "AUTH", class: "permission", severity: "error", retryable: false, description: "The caller lacks the permission this action needs.", since: "0.1.0", deprecated: false, messageKey: "error-AUTH-401" },
  "API-101": { id: "API-101", name: "request_invalid", group: "API", class: "invalid_input", severity: "error", retryable: false, description: "The REST request failed validation.", since: "0.1.0", deprecated: false, messageKey: "error-API-101" },
  "API-102": { id: "API-102", name: "bind_address_not_local", group: "API", class: "invalid_input", severity: "error", retryable: false, description: "The server was asked to listen on an address that is not a loopback address.", since: "0.1.0", deprecated: false, messageKey: "error-API-102" },
  "API-201": { id: "API-201", name: "command_not_found", group: "API", class: "not_found", severity: "error", retryable: false, description: "The REST request names a command or route that does not exist.", since: "0.1.0", deprecated: false, messageKey: "error-API-201" },
  "API-301": { id: "API-301", name: "server_bind_failed", group: "API", class: "conflict", severity: "error", retryable: false, description: "The server could not listen on its address, for example because the port is taken.", since: "0.1.0", deprecated: false, messageKey: "error-API-301" },
  "API-401": { id: "API-401", name: "token_missing_or_invalid", group: "API", class: "permission", severity: "error", retryable: false, description: "A state-changing request lacks a valid installation token.", since: "0.1.0", deprecated: false, messageKey: "error-API-401" },
  "API-402": { id: "API-402", name: "host_or_origin_rejected", group: "API", class: "permission", severity: "error", retryable: false, description: "The Host or Origin header of the request is not one of this server.", since: "0.1.0", deprecated: false, messageKey: "error-API-402" },
  "API-403": { id: "API-403", name: "launch_code_invalid", group: "API", class: "permission", severity: "error", retryable: false, description: "The one-time launch code is wrong, already used or expired.", since: "0.1.0", deprecated: false, messageKey: "error-API-403" },
  "API-501": { id: "API-501", name: "request_too_large", group: "API", class: "limit", severity: "error", retryable: false, description: "The request body is larger than the server accepts.", since: "0.1.0", deprecated: false, messageKey: "error-API-501" },
  "API-601": { id: "API-601", name: "server_not_reachable", group: "API", class: "external", severity: "error", retryable: true, description: "The command needs a running local server and could not reach it.", since: "0.1.0", deprecated: false, messageKey: "error-API-601" },
  "MCP-101": { id: "MCP-101", name: "tool_arguments_invalid", group: "MCP", class: "invalid_input", severity: "error", retryable: false, description: "MCP tool arguments failed validation.", since: "0.1.0", deprecated: false, messageKey: "error-MCP-101" },
  "MCP-201": { id: "MCP-201", name: "project_not_registered", group: "MCP", class: "not_found", severity: "error", retryable: false, description: "The requested path is not inside a registered project.", since: "0.1.0", deprecated: false, messageKey: "error-MCP-201" },
  "MCP-202": { id: "MCP-202", name: "symbol_not_found", group: "MCP", class: "not_found", severity: "error", retryable: false, description: "No symbol with this id or qualified name exists in the newest snapshot.", since: "0.1.0", deprecated: false, messageKey: "error-MCP-202" },
  "MCP-203": { id: "MCP-203", name: "project_not_scanned", group: "MCP", class: "not_found", severity: "error", retryable: true, description: "The project has no snapshot yet; scan it first.", since: "0.1.0", deprecated: false, messageKey: "error-MCP-203" },
  "MCP-501": { id: "MCP-501", name: "response_limit_exceeded", group: "MCP", class: "limit", severity: "info", retryable: false, description: "The result was truncated to the configured limit.", since: "0.1.0", deprecated: false, messageKey: "error-MCP-501" },
  "CLI-101": { id: "CLI-101", name: "cli_arguments_invalid", group: "CLI", class: "invalid_input", severity: "error", retryable: false, description: "Command-line arguments are invalid.", since: "0.1.0", deprecated: false, messageKey: "error-CLI-101" },
  "CLI-102": { id: "CLI-102", name: "watch_needs_folder", group: "CLI", class: "invalid_input", severity: "error", retryable: false, description: "Watch mode needs a local folder; a Git URL cannot be watched.", since: "0.1.0", deprecated: false, messageKey: "error-CLI-102" },
  "CLI-103": { id: "CLI-103", name: "secret_input_invalid", group: "CLI", class: "invalid_input", severity: "error", retryable: false, description: "No secret could be read from the terminal or standard input, or it was empty.", since: "0.1.0", deprecated: false, messageKey: "error-CLI-103" },
  "I18N-201": { id: "I18N-201", name: "message_key_missing", group: "I18N", class: "not_found", severity: "error", retryable: false, description: "A message key is missing for the active language (falls back to English).", since: "0.1.0", deprecated: false, messageKey: "error-I18N-201" },
  "EXP-201": { id: "EXP-201", name: "export_bundle_missing", group: "EXP", class: "not_found", severity: "error", retryable: false, description: "The built export page of the UI is missing or unusable.", since: "0.1.0", deprecated: false, messageKey: "error-EXP-201" },
  "EXP-601": { id: "EXP-601", name: "export_write_failed", group: "EXP", class: "external", severity: "error", retryable: false, description: "The export file could not be written.", since: "0.1.0", deprecated: false, messageKey: "error-EXP-601" },
  "UI-201": { id: "UI-201", name: "api_unreachable", group: "UI", class: "not_found", severity: "error", retryable: false, description: "The UI cannot reach the local server.", since: "0.1.0", deprecated: false, messageKey: "error-UI-201" },
  "UI-901": { id: "UI-901", name: "unexpected_ui_error", group: "UI", class: "internal", severity: "error", retryable: false, description: "An uncaught exception was converted at a feature boundary (bug).", since: "0.1.0", deprecated: false, messageKey: "error-UI-901" },
  "CHK-101": { id: "CHK-101", name: "literal_found", group: "CHK", class: "invalid_input", severity: "error", retryable: false, description: "A literal outside the allowed exceptions was found.", since: "0.1.0", deprecated: false, messageKey: "error-CHK-101" },
  "CHK-102": { id: "CHK-102", name: "duplicate_block", group: "CHK", class: "invalid_input", severity: "error", retryable: false, description: "A code block duplicates existing code.", since: "0.1.0", deprecated: false, messageKey: "error-CHK-102" },
  "CHK-103": { id: "CHK-103", name: "layer_violation", group: "CHK", class: "invalid_input", severity: "error", retryable: false, description: "A dependency points against the allowed direction.", since: "0.1.0", deprecated: false, messageKey: "error-CHK-103" },
  "CHK-104": { id: "CHK-104", name: "function_too_long", group: "CHK", class: "invalid_input", severity: "error", retryable: false, description: "A function exceeds the configured length.", since: "0.1.0", deprecated: false, messageKey: "error-CHK-104" },
  "CHK-105": { id: "CHK-105", name: "complexity_too_high", group: "CHK", class: "invalid_input", severity: "error", retryable: false, description: "A function exceeds the configured cognitive complexity.", since: "0.1.0", deprecated: false, messageKey: "error-CHK-105" },
};
