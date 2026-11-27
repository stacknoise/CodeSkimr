import { currentLanguage } from "../../shared/i18n/i18n";
import { HISTORY_MOMENT, MS_PER_SECOND } from "./consts";

/** The day and the time of a scan, in the language of the screen. `created` is in seconds since
 * 1970. */
export function moment(created: number): string {
  return new Date(created * MS_PER_SECOND).toLocaleString(currentLanguage(), HISTORY_MOMENT);
}
