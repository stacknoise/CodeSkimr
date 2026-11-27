import { compareValues, type SortState } from "../../shared/table/sort";
import { COLUMN } from "./consts";
import type { Finding } from "./types";

/** The number a finding measures, whatever its rule: lines, complexity or parameters, literals, or
 * the share of a duplicate. */
export function measureOf(finding: Finding): number {
  return finding.lines ?? finding.value ?? finding.literals ?? finding.similarity_percent ?? 0;
}

/** The value of a finding in a column. */
function valueOf(finding: Finding, column: string): string | number {
  switch (column) {
    case COLUMN.place:
      return finding.path;
    case COLUMN.symbol:
      return finding.symbol;
    case COLUMN.measure:
      return measureOf(finding);
    default:
      return finding.code;
  }
}

/** The findings in the order of a column; two that are equal keep the order of the server (the sort
 * of the browser is stable). Without a sort the order of the server stays. */
export function sortFindings(
  findings: readonly Finding[],
  language: string,
  sort: SortState | null,
): Finding[] {
  if (sort === null) {
    return [...findings];
  }
  return [...findings].sort((a, b) =>
    compareValues(valueOf(a, sort.column), valueOf(b, sort.column), sort.direction, language),
  );
}
