import { I18N_KEYS } from "../i18n/keys";
import { SCAN_LEVELS } from "./consts";
import type { MessageKey } from "../i18n/i18n";

/** The levels a scan can reach, as the report names them. */
export type ScanLevel = (typeof SCAN_LEVELS)[number];

/** What `projects.list` and `usages.query` say about the last scan (ADR 0072, decision 11). */
export interface LastScan {
  /** Seconds since the Unix epoch. */
  at: number;
  level: ScanLevel;
  origin: ScanOrigin;
}

/** Who started a scan, as the server saw the channel. */
export type ScanOrigin = "terminal" | "browser" | "watch" | "mcp";

/** What level 2 did for one language of the project. */
export interface LanguageReport {
  language: string;
  outcome: "checked" | "skipped" | "failed";
  /** For `checked`: files whose references come from the index. */
  files?: number | null;
  /** For `checked`: files that changed meanwhile and kept level 1. */
  stale?: number | null;
  /** For `skipped`: a reason key. */
  reason?: string | null;
  /** For `failed`: the registered error code. */
  code?: string | null;
}

/** The newest second pass of a project. */
export interface Level2Report {
  at: number;
  origin: ScanOrigin;
  languages: LanguageReport[];
}

/** The report of the last scan of a project. */
export interface ScanReport {
  last_scan: LastScan;
  level2: Level2Report | null;
}

/** The outcomes of a language in the report. */
export const OUTCOME = { checked: "checked", skipped: "skipped", failed: "failed" } as const;

/** The texts of the origins. */
export const ORIGIN_TEXT: Record<ScanOrigin, MessageKey> = {
  terminal: I18N_KEYS.PROJECTS_REPORT_ORIGIN_TERMINAL,
  browser: I18N_KEYS.PROJECTS_REPORT_ORIGIN_BROWSER,
  watch: I18N_KEYS.PROJECTS_REPORT_ORIGIN_WATCH,
  mcp: I18N_KEYS.PROJECTS_REPORT_ORIGIN_MCP,
};

/** The reasons of the report that the screens treat specially. */
export const SKIP_REASON = {
  noProjectFile: "no_project_file",
  toolsUnavailable: "tools_unavailable",
} as const;

/** The texts of the reasons why a language was skipped, by the key of the report. */
export const REASON_TEXT: Record<string, MessageKey> = {
  not_permitted: I18N_KEYS.PROJECTS_REPORT_REASON_NOT_PERMITTED,
  not_confirmed: I18N_KEYS.PROJECTS_REPORT_REASON_NOT_CONFIRMED,
  [SKIP_REASON.toolsUnavailable]: I18N_KEYS.PROJECTS_REPORT_REASON_TOOLS_UNAVAILABLE,
  [SKIP_REASON.noProjectFile]: I18N_KEYS.PROJECTS_REPORT_REASON_NO_PROJECT_FILE,
  stale: I18N_KEYS.PROJECTS_REPORT_REASON_STALE,
  cancelled: I18N_KEYS.PROJECTS_REPORT_REASON_CANCELLED,
  busy: I18N_KEYS.PROJECTS_REPORT_REASON_BUSY,
  paused: I18N_KEYS.PROJECTS_REPORT_REASON_PAUSED,
};

/** The text of a skip reason; a reason of a newer server is shown as its key. */
export function reasonKey(reason: string | null | undefined): MessageKey | null {
  return reason ? (REASON_TEXT[reason] ?? null) : null;
}

/** How many languages ended in each way. */
export function outcomeCounts(report: Level2Report): Record<keyof typeof OUTCOME, number> {
  const counts = { checked: 0, skipped: 0, failed: 0 };
  for (const line of report.languages) {
    counts[line.outcome] += 1;
  }
  return counts;
}

/** Files of checked languages that changed since the run and keep level 1. */
export function staleFiles(report: Level2Report): number {
  return report.languages.reduce((sum, line) => sum + (line.stale ?? 0), 0);
}

/** The texts of the missing project file, by language; the generic reason text stands for the rest. */
export const NO_PROJECT_FILE_TEXT: Record<string, MessageKey> = {
  rust: I18N_KEYS.PROJECTS_REPORT_REASON_NO_PROJECT_FILE_RUST,
  csharp: I18N_KEYS.PROJECTS_REPORT_REASON_NO_PROJECT_FILE_CSHARP,
};

/** What to do about a skipped language, by the reason key of the report. */
export const SKIP_TODO: Record<string, MessageKey> = {
  not_permitted: I18N_KEYS.PROJECTS_REPORT_TODO_NOT_PERMITTED,
  not_confirmed: I18N_KEYS.PROJECTS_REPORT_TODO_NOT_CONFIRMED,
  [SKIP_REASON.toolsUnavailable]: I18N_KEYS.PROJECTS_REPORT_TODO_TOOLS_UNAVAILABLE,
  [SKIP_REASON.noProjectFile]: I18N_KEYS.PROJECTS_REPORT_TODO_NO_PROJECT_FILE,
  stale: I18N_KEYS.PROJECTS_REPORT_TODO_STALE,
  cancelled: I18N_KEYS.PROJECTS_REPORT_TODO_RETRY,
  busy: I18N_KEYS.PROJECTS_REPORT_TODO_LATER,
  paused: I18N_KEYS.PROJECTS_REPORT_TODO_LATER,
};

/** What to do about a failed language, by its error code; any other code is simply tried again. */
export const FAILED_TODO: Record<string, MessageKey> = {
  "RES-501": I18N_KEYS.PROJECTS_REPORT_TODO_RETRY,
  "RES-502": I18N_KEYS.PROJECTS_REPORT_TODO_RES_502,
  "RES-503": I18N_KEYS.PROJECTS_REPORT_TODO_RES_503,
  "RES-504": I18N_KEYS.PROJECTS_REPORT_TODO_RES_504,
  "RES-601": I18N_KEYS.PROJECTS_REPORT_TODO_RES_601,
};
