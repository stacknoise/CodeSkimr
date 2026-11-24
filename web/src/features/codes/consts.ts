import type { ErrorCodeInfo } from "../../shared/api/error-codes.generated";
import { I18N_KEYS } from "../../shared/i18n/keys";

/** The severity filter: all codes, or the codes of one severity. */
export const SEVERITY_ALL = "all";
/** The group filter: all groups. */
export const GROUP_ALL = "all";

/** The severities in the order of the filter buttons. */
export const SEVERITIES: ReadonlyArray<ErrorCodeInfo["severity"]> = ["error", "warning", "info"];

/** The tone of the pill of each severity. */
export const SEVERITY_TONE = { error: "bad", warning: "warn", info: "neutral" } as const;

/** Id of the group filter, so that the label finds it. */
export const GROUP_FIELD_ID = "codes-group";
/** Id of the search field, so that the label finds it. */
export const SEARCH_FIELD_ID = "codes-search";

/** Separates the lines of the details that are copied. */
export const LINE_BREAK = "\n";
/** Separates a code from its message in the copied details. */
export const CODE_SEPARATOR = ": ";

/** The text of each severity. */
export const SEVERITY_KEY = {
  error: I18N_KEYS.CODES_SEVERITY_ERROR,
  warning: I18N_KEYS.CODES_SEVERITY_WARNING,
  info: I18N_KEYS.CODES_SEVERITY_INFO,
} as const;
