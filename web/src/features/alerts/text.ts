import { currentLanguage, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import {
  CHANGE_SEPARATOR,
  CHANGE_TEXT,
  COMMIT_SHORT,
  KIND,
  META_SEPARATOR,
  MS_PER_SECOND,
  RELATIVE_DAYS,
  RELATIVE_STYLE,
  SECONDS,
  TRIGGER_TEXT,
  UNIT,
} from "./consts";
import type { Alert } from "./types";

/** The sentence under the name of an alert, from the fields the server gives. */
export function sentenceOf(alert: Alert): string {
  if (alert.kind === KIND.removed) {
    return alert.references === null
      ? t(I18N_KEYS.ALERTS_TEXT_REMOVED_PLAIN)
      : t(I18N_KEYS.ALERTS_TEXT_REMOVED, { count: alert.references });
  }
  if (alert.kind === KIND.orphaned) {
    return alert.closest === null
      ? t(I18N_KEYS.ALERTS_TEXT_ORPHANED_PLAIN)
      : t(I18N_KEYS.ALERTS_TEXT_ORPHANED, { closest: alert.closest });
  }
  if (alert.changes.length === 0) {
    return t(I18N_KEYS.ALERTS_TEXT_CHANGED_PLAIN);
  }
  const what = alert.changes.map((change) => t(CHANGE_TEXT[change])).join(CHANGE_SEPARATOR);
  return t(I18N_KEYS.ALERTS_TEXT_CHANGED, { what });
}

/** A time in seconds since 1970 as "2 min ago" in the language of the UI; a date after a week. */
export function whenText(seconds: number, now: number): string {
  const language = currentLanguage();
  const age = Math.max(0, now - seconds);
  const format = new Intl.RelativeTimeFormat(language, RELATIVE_STYLE);
  if (age < SECONDS.minute) {
    return format.format(0, UNIT.second);
  }
  if (age < SECONDS.hour) {
    return format.format(-Math.floor(age / SECONDS.minute), UNIT.minute);
  }
  if (age < SECONDS.day) {
    return format.format(-Math.floor(age / SECONDS.hour), UNIT.hour);
  }
  if (age < SECONDS.day * RELATIVE_DAYS) {
    return format.format(-Math.floor(age / SECONDS.day), UNIT.day);
  }
  return new Intl.DateTimeFormat(language).format(new Date(seconds * MS_PER_SECOND));
}

/** The line under an alert: when, how often, what raised it and the commit. */
export function metaOf(alert: Alert, now: number): string {
  const parts = [whenText(alert.last_seen, now)];
  if (alert.counter > 1) {
    parts.push(t(I18N_KEYS.ALERTS_SEEN, { count: alert.counter }));
  }
  const trigger = TRIGGER_TEXT[alert.trigger];
  if (trigger !== undefined) {
    parts.push(t(trigger));
  }
  if (alert.commit !== null) {
    parts.push(alert.commit.slice(0, COMMIT_SHORT));
  }
  return parts.join(META_SEPARATOR);
}
