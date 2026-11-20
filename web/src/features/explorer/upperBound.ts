import { OUTCOME, type ScanReport } from "../../shared/project/report";

/** Whether the coupling counts of a symbol are only an upper bound (ADR 0037): level 1 counts
 * every reference, also the probable ones, so a count is firm only where level 2 checked the
 * language of the symbol in the newest scan. A project without a stored report is not confirmed. */
export function isUpperBound(report: ScanReport | null | undefined, language: string): boolean {
  const line = report?.level2?.languages.find((entry) => entry.language === language);
  return line?.outcome !== OUTCOME.checked;
}
