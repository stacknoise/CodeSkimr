// Generated from errors/error-codes.toml by tools/codegen. Do not edit.

/** Id of a registered error code, e.g. `SRC-201`. */
export type ErrorCode =
  | "APP-201"
  | "APP-301"
  | "APP-901"
  | "CFG-101"
  | "CFG-102"
  | "CFG-103"
  | "PRJ-101"
  | "PRJ-201"
  | "PRJ-301"
  | "PRJ-302"
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
  | "RES-301"
  | "IDX-301"
  | "STO-301"
  | "STO-302"
  | "STO-501"
  | "STO-601"
  | "STO-901"
  | "SEC-201"
  | "SEC-401"
  | "SEC-402"
  | "SEC-601"
  | "SEC-901"
  | "NET-501"
  | "NET-502"
  | "NET-601"
  | "API-101"
  | "API-401"
  | "MCP-101"
  | "MCP-201"
  | "MCP-501"
  | "CLI-101"
  | "I18N-201"
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
  "APP-301": { id: "APP-301", name: "instance_already_running", group: "APP", class: "conflict", severity: "error", retryable: false, description: "Another CodeSkimr instance holds the workspace; connecting to it failed.", since: "0.1.0", deprecated: false, messageKey: "error-APP-301" },
  "APP-901": { id: "APP-901", name: "unexpected_internal_error", group: "APP", class: "internal", severity: "error", retryable: false, description: "An unexpected internal error occurred (bug).", since: "0.1.0", deprecated: false, messageKey: "error-APP-901" },
  "CFG-101": { id: "CFG-101", name: "config_invalid_value", group: "CFG", class: "invalid_input", severity: "error", retryable: false, description: "A configuration value is outside its allowed range or has the wrong type.", since: "0.1.0", deprecated: false, messageKey: "error-CFG-101" },
  "CFG-102": { id: "CFG-102", name: "config_parse_failed", group: "CFG", class: "invalid_input", severity: "error", retryable: false, description: "A configuration file is not valid TOML.", since: "0.1.0", deprecated: false, messageKey: "error-CFG-102" },
  "CFG-103": { id: "CFG-103", name: "config_unknown_key", group: "CFG", class: "invalid_input", severity: "error", retryable: false, description: "A configuration file or flag sets a key that does not exist.", since: "0.1.0", deprecated: false, messageKey: "error-CFG-103" },
  "PRJ-101": { id: "PRJ-101", name: "project_source_invalid", group: "PRJ", class: "invalid_input", severity: "error", retryable: false, description: "The given source is neither an existing folder nor a valid Git URL.", since: "0.1.0", deprecated: false, messageKey: "error-PRJ-101" },
  "PRJ-201": { id: "PRJ-201", name: "project_not_found", group: "PRJ", class: "not_found", severity: "error", retryable: false, description: "No project with this id exists.", since: "0.1.0", deprecated: false, messageKey: "error-PRJ-201" },
  "PRJ-301": { id: "PRJ-301", name: "project_already_registered", group: "PRJ", class: "conflict", severity: "error", retryable: false, description: "A project with the same source is already registered.", since: "0.1.0", deprecated: false, messageKey: "error-PRJ-301" },
  "PRJ-302": { id: "PRJ-302", name: "snapshot_in_progress", group: "PRJ", class: "conflict", severity: "error", retryable: false, description: "A scan for this project is already running.", since: "0.1.0", deprecated: false, messageKey: "error-PRJ-302" },
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
  "RES-301": { id: "RES-301", name: "ambiguous_reference", group: "RES", class: "conflict", severity: "warning", retryable: false, description: "A reference matches several symbols; it was stored as probable.", since: "0.1.0", deprecated: false, messageKey: "error-RES-301" },
  "IDX-301": { id: "IDX-301", name: "read_model_stale", group: "IDX", class: "conflict", severity: "info", retryable: false, description: "A read model is older than the current snapshot and is being rebuilt.", since: "0.1.0", deprecated: false, messageKey: "error-IDX-301" },
  "STO-301": { id: "STO-301", name: "schema_migration_failed", group: "STO", class: "conflict", severity: "error", retryable: false, description: "Applying a database migration failed.", since: "0.1.0", deprecated: false, messageKey: "error-STO-301" },
  "STO-302": { id: "STO-302", name: "transaction_conflict", group: "STO", class: "conflict", severity: "error", retryable: true, description: "A concurrent write conflicted; the operation can be retried.", since: "0.1.0", deprecated: false, messageKey: "error-STO-302" },
  "STO-501": { id: "STO-501", name: "disk_full", group: "STO", class: "limit", severity: "error", retryable: false, description: "Not enough disk space to persist the snapshot.", since: "0.1.0", deprecated: false, messageKey: "error-STO-501" },
  "STO-601": { id: "STO-601", name: "database_failed", group: "STO", class: "external", severity: "error", retryable: false, description: "The database reported an error that is not covered by a more specific code.", since: "0.1.0", deprecated: false, messageKey: "error-STO-601" },
  "STO-901": { id: "STO-901", name: "stored_data_invalid", group: "STO", class: "internal", severity: "error", retryable: false, description: "Data read from the database violates the domain model; this is a bug or a damaged database.", since: "0.1.0", deprecated: false, messageKey: "error-STO-901" },
  "SEC-201": { id: "SEC-201", name: "credential_not_found", group: "SEC", class: "not_found", severity: "error", retryable: false, description: "The referenced credential does not exist in the keystore.", since: "0.1.0", deprecated: false, messageKey: "error-SEC-201" },
  "SEC-401": { id: "SEC-401", name: "credential_rejected", group: "SEC", class: "permission", severity: "error", retryable: false, description: "The platform rejected the credential.", since: "0.1.0", deprecated: false, messageKey: "error-SEC-401" },
  "SEC-402": { id: "SEC-402", name: "keystore_access_denied", group: "SEC", class: "permission", severity: "error", retryable: false, description: "The OS keystore denied access.", since: "0.1.0", deprecated: false, messageKey: "error-SEC-402" },
  "SEC-601": { id: "SEC-601", name: "keystore_unavailable", group: "SEC", class: "external", severity: "error", retryable: false, description: "The OS keystore service is not available.", since: "0.1.0", deprecated: false, messageKey: "error-SEC-601" },
  "SEC-901": { id: "SEC-901", name: "memory_lock_failed", group: "SEC", class: "internal", severity: "error", retryable: false, description: "Locking secret memory against swapping failed.", since: "0.1.0", deprecated: false, messageKey: "error-SEC-901" },
  "NET-501": { id: "NET-501", name: "request_timeout", group: "NET", class: "limit", severity: "error", retryable: true, description: "A request to an external platform timed out.", since: "0.1.0", deprecated: false, messageKey: "error-NET-501" },
  "NET-502": { id: "NET-502", name: "rate_limited", group: "NET", class: "limit", severity: "error", retryable: true, description: "The external platform rate-limited the request.", since: "0.1.0", deprecated: false, messageKey: "error-NET-502" },
  "NET-601": { id: "NET-601", name: "oauth_flow_failed", group: "NET", class: "external", severity: "error", retryable: false, description: "The OAuth device flow did not complete.", since: "0.1.0", deprecated: false, messageKey: "error-NET-601" },
  "API-101": { id: "API-101", name: "request_invalid", group: "API", class: "invalid_input", severity: "error", retryable: false, description: "The REST request failed validation.", since: "0.1.0", deprecated: false, messageKey: "error-API-101" },
  "API-401": { id: "API-401", name: "token_missing_or_invalid", group: "API", class: "permission", severity: "error", retryable: false, description: "A state-changing request lacks a valid installation token.", since: "0.1.0", deprecated: false, messageKey: "error-API-401" },
  "MCP-101": { id: "MCP-101", name: "tool_arguments_invalid", group: "MCP", class: "invalid_input", severity: "error", retryable: false, description: "MCP tool arguments failed validation.", since: "0.1.0", deprecated: false, messageKey: "error-MCP-101" },
  "MCP-201": { id: "MCP-201", name: "project_not_registered", group: "MCP", class: "not_found", severity: "error", retryable: false, description: "The requested path is not inside a registered project.", since: "0.1.0", deprecated: false, messageKey: "error-MCP-201" },
  "MCP-501": { id: "MCP-501", name: "response_limit_exceeded", group: "MCP", class: "limit", severity: "info", retryable: false, description: "The result was truncated to the configured limit.", since: "0.1.0", deprecated: false, messageKey: "error-MCP-501" },
  "CLI-101": { id: "CLI-101", name: "cli_arguments_invalid", group: "CLI", class: "invalid_input", severity: "error", retryable: false, description: "Command-line arguments are invalid.", since: "0.1.0", deprecated: false, messageKey: "error-CLI-101" },
  "I18N-201": { id: "I18N-201", name: "message_key_missing", group: "I18N", class: "not_found", severity: "error", retryable: false, description: "A message key is missing for the active language (falls back to English).", since: "0.1.0", deprecated: false, messageKey: "error-I18N-201" },
  "EXP-601": { id: "EXP-601", name: "export_write_failed", group: "EXP", class: "external", severity: "error", retryable: false, description: "The export file could not be written.", since: "0.1.0", deprecated: false, messageKey: "error-EXP-601" },
  "UI-201": { id: "UI-201", name: "api_unreachable", group: "UI", class: "not_found", severity: "error", retryable: false, description: "The UI cannot reach the local server.", since: "0.1.0", deprecated: false, messageKey: "error-UI-201" },
  "UI-901": { id: "UI-901", name: "unexpected_ui_error", group: "UI", class: "internal", severity: "error", retryable: false, description: "An uncaught exception was converted at a feature boundary (bug).", since: "0.1.0", deprecated: false, messageKey: "error-UI-901" },
  "CHK-101": { id: "CHK-101", name: "literal_found", group: "CHK", class: "invalid_input", severity: "error", retryable: false, description: "A literal outside the allowed exceptions was found.", since: "0.1.0", deprecated: false, messageKey: "error-CHK-101" },
  "CHK-102": { id: "CHK-102", name: "duplicate_block", group: "CHK", class: "invalid_input", severity: "error", retryable: false, description: "A code block duplicates existing code.", since: "0.1.0", deprecated: false, messageKey: "error-CHK-102" },
  "CHK-103": { id: "CHK-103", name: "layer_violation", group: "CHK", class: "invalid_input", severity: "error", retryable: false, description: "A dependency points against the allowed direction.", since: "0.1.0", deprecated: false, messageKey: "error-CHK-103" },
  "CHK-104": { id: "CHK-104", name: "function_too_long", group: "CHK", class: "invalid_input", severity: "error", retryable: false, description: "A function exceeds the configured length.", since: "0.1.0", deprecated: false, messageKey: "error-CHK-104" },
  "CHK-105": { id: "CHK-105", name: "complexity_too_high", group: "CHK", class: "invalid_input", severity: "error", retryable: false, description: "A function exceeds the configured cognitive complexity.", since: "0.1.0", deprecated: false, messageKey: "error-CHK-105" },
};
