import { ERROR_CODES, type ErrorCodeInfo } from "../../shared/api/error-codes.generated";
import { GROUP_ALL, SEVERITY_ALL } from "./consts";

/** What the user chose to see. */
export interface CodeFilter {
  /** Part of an id or of the message; an exact id also finds a retired code. */
  query: string;
  /** `SEVERITY_ALL` or one severity. */
  severity: string;
  /** `GROUP_ALL` or a group prefix such as `SRC`. */
  group: string;
}

/** All registered codes, in the order of the registry. */
export function allCodes(): ErrorCodeInfo[] {
  return Object.values(ERROR_CODES);
}

/** The group prefixes of the active codes, sorted. */
export function groupsOf(codes: readonly ErrorCodeInfo[]): string[] {
  return [...new Set(codes.filter((code) => !code.deprecated).map((code) => code.group))].sort();
}

function matchesQuery(code: ErrorCodeInfo, query: string, message: string): boolean {
  const needle = query.trim().toLowerCase();
  if (needle === "") {
    return true;
  }
  return code.id.toLowerCase().includes(needle) || message.toLowerCase().includes(needle);
}

/** A retired code is shown only when its id is typed in full (`codeskimr codes <id>`, ADR 0078). */
function visible(code: ErrorCodeInfo, query: string): boolean {
  return !code.deprecated || code.id.toLowerCase() === query.trim().toLowerCase();
}

/** The codes that pass the filter; `messageOf` gives the message in the language of the UI. */
export function filterCodes(
  codes: readonly ErrorCodeInfo[],
  filter: CodeFilter,
  messageOf: (code: ErrorCodeInfo) => string,
): ErrorCodeInfo[] {
  return codes.filter(
    (code) =>
      visible(code, filter.query) &&
      (filter.severity === SEVERITY_ALL || code.severity === filter.severity) &&
      (filter.group === GROUP_ALL || code.group === filter.group) &&
      matchesQuery(code, filter.query, messageOf(code)),
  );
}
