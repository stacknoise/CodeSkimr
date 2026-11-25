import { codeText, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { OUTCOME, reasonKey } from "../../shared/project/report";
import { languageName } from "../projects/format";
import type { UsagesAnswer } from "./types";

/** What the stored report says about the language of the symbol: checked, checked in part, skipped
 * with its reason, failed with its code, or "no indexer" when the last second pass did not list
 * the language (ADR 0072, decision 10). `null` for an answer without the field (an older export),
 * so that the card keeps its general text there. */
export function accuracyNote(answer: UsagesAnswer): string | null {
  if (answer.scan_report === undefined) {
    return null;
  }
  const second = answer.scan_report?.level2 ?? null;
  const id = answer.language ?? "";
  const language = languageName(id);
  if (second === null) {
    return t(I18N_KEYS.USAGES_REPORT_NONE);
  }
  const line = second.languages.find((entry) => entry.language === id);
  if (line === undefined) {
    return t(I18N_KEYS.USAGES_REPORT_NO_INDEXER, { language });
  }
  if (line.outcome === OUTCOME.failed) {
    return t(I18N_KEYS.USAGES_REPORT_FAILED, {
      language,
      code: line.code ?? "",
      text: codeText(line.code),
    });
  }
  if (line.outcome === OUTCOME.skipped) {
    const key = reasonKey(line.reason);
    return t(I18N_KEYS.USAGES_REPORT_SKIPPED, {
      language,
      reason: key === null ? (line.reason ?? "") : t(key),
    });
  }
  const files = line.files ?? 0;
  const stale = line.stale ?? 0;
  return stale > 0
    ? t(I18N_KEYS.USAGES_REPORT_PARTIAL, { language, files, stale })
    : t(I18N_KEYS.USAGES_REPORT_CHECKED, { language, files });
}
