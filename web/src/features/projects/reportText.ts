import { codeText, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import {
  FAILED_TODO,
  NO_PROJECT_FILE_TEXT,
  ORIGIN_TEXT,
  OUTCOME,
  outcomeCounts,
  reasonKey,
  SKIP_TODO,
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

/** What the person can press next to a language of the report. */
export const REPORT_ACTION = { rescan: "rescan", tools: "tools" } as const;

/** One language of the report as the screen shows it: what happened, what to do, and the button. */
export interface ReportRow {
  language: string;
  /** The text of the reason or of the error. */
  text: string;
  /** The error code of a failed language. */
  code: string | null;
  /** The sentence "What to do". */
  todo: string;
  action: (typeof REPORT_ACTION)[keyof typeof REPORT_ACTION];
}

function skippedText(line: LanguageReport): string {
  const key =
    line.reason === "no_project_file"
      ? (NO_PROJECT_FILE_TEXT[line.language] ?? reasonKey(line.reason))
      : reasonKey(line.reason);
  return key === null ? (line.reason ?? "") : t(key);
}

/** The row of a language that failed or was skipped. */
export function reportRow(line: LanguageReport): ReportRow {
  const language = languageName(line.language);
  if (line.outcome === OUTCOME.failed) {
    const code = line.code ?? "";
    const todo = FAILED_TODO[code] ?? I18N_KEYS.PROJECTS_REPORT_TODO_RETRY;
    return {
      language,
      text: codeText(code),
      code,
      todo: t(I18N_KEYS.PROJECTS_REPORT_TODO, { text: t(todo) }),
      action: REPORT_ACTION.rescan,
    };
  }
  const todo = SKIP_TODO[line.reason ?? ""] ?? I18N_KEYS.PROJECTS_REPORT_TODO_RETRY;
  return {
    language,
    text: skippedText(line),
    code: null,
    todo: t(I18N_KEYS.PROJECTS_REPORT_TODO, { text: t(todo) }),
    action: line.reason === "tools_unavailable" ? REPORT_ACTION.tools : REPORT_ACTION.rescan,
  };
}

/** The chip of a checked language. */
export function checkedChip(line: LanguageReport): string {
  const language = languageName(line.language);
  const files = line.files ?? 0;
  const stale = line.stale ?? 0;
  return stale > 0
    ? t(I18N_KEYS.PROJECTS_REPORT_CHIP_STALE, { language, files, stale })
    : t(I18N_KEYS.PROJECTS_REPORT_CHIP, { language, files });
}
