import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { uiConfig } from "../../shared/settings/ui-config";
import { languageName } from "../projects/format";

/** Whether the usages of `language` are an approximation: the server lists the languages whose
 * level 2 agrees with the compiler in less than 99 % of the measured places
 * (`ui.approximate_languages`, `docs/accuracy.md`). A server that tells nothing marks none. */
export function isApproximate(language: string | undefined): boolean {
  return language !== undefined && (uiConfig()?.approximate_languages ?? []).includes(language);
}

/** The short explanation of the approximation, or `null` for a language that is not one. */
export function approximationNote(language: string | undefined): string | null {
  return isApproximate(language)
    ? t(I18N_KEYS.USAGES_APPROX_NOTE, { language: languageName(language ?? "") })
    : null;
}
