import { createSignal } from "solid-js";
import { ApiError } from "../../shared/api/client";
import { runCommand } from "../../shared/api/commands";
import type { ErrorCode } from "../../shared/api/error-codes.generated";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { COMMAND, DATE_LOCALE } from "./consts";
import { loadStats } from "./stats";
import type { IgnoreRule, Row } from "./types";

const [keepFailure, setKeepFailure] = createSignal<ErrorCode | null>(null);

/** The code of the last failed attempt to keep or lift a symbol, `null` if it went through. */
export { keepFailure };

/** Runs a command that changes the exception list and loads the statistics again. */
async function change(project: number, command: string, input: object): Promise<boolean> {
  try {
    await runCommand(command, { project, ...input });
    setKeepFailure(null);
    await loadStats(project);
    return true;
  } catch (error) {
    setKeepFailure(error instanceof ApiError ? error.code : null);
    return false;
  }
}

/** Writes the symbol of a row into the exception list of the project, with the reason and the
 * local date of today as YYYY-MM-DD (ADR 0101). */
export function keepSymbol(project: number, row: Row, reason: string): Promise<boolean> {
  const date = new Date().toLocaleDateString(DATE_LOCALE);
  const text = t(I18N_KEYS.STATS_KEEP_REASON_TEXT, { reason: reason.trim(), date });
  return change(project, COMMAND.keep, { symbol: row.qualified_name, reason: text });
}

/** Takes an entry out of the exception list again. */
export function liftRule(project: number, rule: IgnoreRule): Promise<boolean> {
  return change(project, COMMAND.lift, {
    paths: rule.paths ?? [],
    symbols: rule.symbols ?? [],
    reason: rule.reason,
  });
}
