import { errorText, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import type { Failure } from "./projects";

/** The text of a failure: the registered message, or the general one without a code. */
export function failureText(failure: Failure): string {
  return failure.code === null ? t(I18N_KEYS.PROJECTS_LOAD_FAILED) : errorText(failure.code);
}
