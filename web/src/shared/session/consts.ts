/** Key of the token in `sessionStorage`: survives a reload, is not shared with other tabs. */
export const TOKEN_STORAGE_KEY = "codeskimr.token";
/** Name of the fragment parameter that carries the launch code (ADR 0028). */
export const CODE_PARAMETER = "code";
/** The command that opens CodeSkimr with a new launch code. */
export const OPEN_COMMAND = "codeskimr open";

/** Where the connection stands. */
export const KIND = {
  connecting: "connecting",
  connected: "connected",
  reopen: "reopen",
} as const;

/** Why the UI has no valid token. */
export const REASON = {
  missing: "missing",
  codeInvalid: "code-invalid",
  unreachable: "unreachable",
} as const;
