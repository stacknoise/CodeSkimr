import {
  DATE_FORMAT,
  DATE_LENGTH,
  DAY_START_UTC,
  DUE_FORMAT,
  DUE_OFFSET_SECONDS,
  MS_PER_SECOND,
  SECONDS_PER_DAY,
} from "./consts";

/** A time in seconds since 1970 as a short date in the language of the UI (`12.11.2026`). */
export function shortDate(seconds: number, language: string): string {
  return new Intl.DateTimeFormat(language, DATE_FORMAT).format(new Date(seconds * MS_PER_SECOND));
}

/** A due date as a short date; the day it was entered, whatever the time zone. */
export function dueDate(seconds: number, language: string): string {
  return new Intl.DateTimeFormat(language, DUE_FORMAT).format(new Date(seconds * MS_PER_SECOND));
}

/** The value of a date field (`2026-11-20`) for a due date; empty for none. */
export function dateFieldOf(seconds: number | null | undefined): string {
  return seconds === null || seconds === undefined
    ? ""
    : new Date(seconds * MS_PER_SECOND).toISOString().slice(0, DATE_LENGTH);
}

/** The due date of the value of a date field, at midday UTC so that every time zone shows the
 * same day; `null` for an empty field. */
export function dueOfField(value: string): number | null {
  if (value === "") {
    return null;
  }
  const day = Date.parse(`${value}${DAY_START_UTC}`);
  return Number.isNaN(day) ? null : day / MS_PER_SECOND + DUE_OFFSET_SECONDS;
}

/** Whether a due date lies before today. */
export function isLate(due: number, now: number): boolean {
  return due < now - (now % SECONDS_PER_DAY);
}
