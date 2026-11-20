import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import {
  ORIGIN_TEXT,
  OUTCOME,
  outcomeCounts,
  reasonKey,
  staleFiles,
  type LanguageReport,
  type ScanReport,
} from "../../shared/project/report";
import { languageName } from "./format";

/** "Stage 1 (browser)" or "Stage 2 (terminal)": which level the last scan reached and who asked. */
export function lastScanLine(report: ScanReport): string {
  return t(I18N_KEYS.PROJECTS_REPORT_LINE, {
    level: report.last_scan.level,
    origin: t(ORIGIN_TEXT[report.last_scan.origin]),
  });
}

/** What the second pass did in numbers, and what is wrong with it: the lines under the last scan
 * in the project table. Empty if no second pass is stored. */
export function level2Summary(report: ScanReport): string[] {
  const second = report.level2;
  if (second === null) {
    return [];
  }
  const counts = outcomeCounts(second);
  const lines = [t(I18N_KEYS.PROJECTS_REPORT_COUNTS, counts)];
  const stale = staleFiles(second);
  if (stale > 0) {
    lines.push(t(I18N_KEYS.PROJECTS_REPORT_STALE, { count: stale }));
  }
  for (const line of second.languages) {
    if (line.outcome === OUTCOME.failed && line.code) {
      lines.push(t(I18N_KEYS.PROJECTS_REPORT_FAILED_CODE, { code: line.code }));
    }
  }
  return lines;
}

/** The sentence about one language in the report of the second pass. */
export function languageLine(line: LanguageReport): string {
  const language = languageName(line.language);
  if (line.outcome === OUTCOME.checked) {
    const files = line.files ?? 0;
    const stale = line.stale ?? 0;
    return stale > 0
      ? t(I18N_KEYS.PROJECTS_REPORT_CHECKED_STALE, { language, files, stale })
      : t(I18N_KEYS.PROJECTS_REPORT_CHECKED, { language, files });
  }
  if (line.outcome === OUTCOME.failed) {
    return t(I18N_KEYS.PROJECTS_REPORT_FAILED, { language, code: line.code ?? "" });
  }
  const key = reasonKey(line.reason);
  return t(I18N_KEYS.PROJECTS_REPORT_SKIPPED, {
    language,
    reason: key === null ? (line.reason ?? "") : t(key),
  });
}
