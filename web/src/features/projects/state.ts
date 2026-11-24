import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { OUTCOME } from "../../shared/project/report";
import { TONE } from "../../shared/ui/consts";
import type { PillTone } from "../../shared/ui/Pill";
import { languageName } from "./format";
import { interruptedNow, removingNow } from "./removing";
import { toolsOf } from "./projects";
import { problemsOf } from "./tools";
import type { ProjectRow } from "./types";

/** What a project is doing or lacks, in words and in a tone. The tone is never the only sign. */
export interface RowState {
  text: string;
  tone: PillTone;
}

/** The languages for which stage 2 failed in its last run, and the code of the first failure. */
export function level2Failure(row: ProjectRow): { languages: string[]; code: string } | null {
  const failed = row.report?.level2?.languages.filter((line) => line.outcome === OUTCOME.failed);
  if (failed === undefined || failed.length === 0) {
    return null;
  }
  return {
    languages: failed.map((line) => languageName(line.language)),
    code: failed[0]?.code ?? "",
  };
}

function removalState(row: ProjectRow): RowState | null {
  if (interruptedNow(row)) {
    return { text: t(I18N_KEYS.PROJECTS_REMOVAL_INTERRUPTED), tone: TONE.bad };
  }
  return removingNow(row) ? { text: t(I18N_KEYS.PROJECTS_REMOVING), tone: TONE.warn } : null;
}

/** The one state of a project: removal, scan running, a failed stage 2, missing tools, never
 * scanned, or current. */
export function stateOf(row: ProjectRow, scanning: boolean): RowState {
  const removal = removalState(row);
  if (removal !== null) {
    return removal;
  }
  if (row.running || scanning) {
    return { text: t(I18N_KEYS.PROJECTS_STATUS_RUNNING), tone: TONE.accent };
  }
  if (level2Failure(row) !== null) {
    return { text: t(I18N_KEYS.PROJECTS_STATE_LEVEL2_FAILED), tone: TONE.bad };
  }
  const missing = problemsOf(toolsOf(row)).length;
  if (missing > 0) {
    return { text: t(I18N_KEYS.PROJECTS_TOOLS_MISSING_COUNT, { count: missing }), tone: TONE.warn };
  }
  return row.scan === null
    ? { text: t(I18N_KEYS.PROJECTS_STATUS_UNSCANNED), tone: TONE.neutral }
    : { text: t(I18N_KEYS.PROJECTS_STATUS_CURRENT), tone: TONE.good };
}
