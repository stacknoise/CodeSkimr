import {
  LANGUAGE_NAMES,
  LANGUAGE_TONE,
  LANGUAGE_TONE_OTHER,
  MS_PER_SECOND,
  PERCENT,
  RELATIVE_OPTIONS,
  SECOND_UNIT,
  MICROS_PER_SECOND,
  PARSE_TIME_DECIMALS,
  SECONDS_DECIMALS,
  TIME_UNITS,
} from "./consts";
import type { LanguageInfo } from "./types";

/** A language with its share of the lines of the project. */
export interface LanguageShare {
  language: string;
  name: string;
  tone: string;
  percent: number;
}

/** The display name of a language; an unknown id is shown as it is. */
export function languageName(id: string): string {
  return LANGUAGE_NAMES[id] ?? id;
}

/** Shares of the languages in percent. The shares add up to 100: the rounding error goes to the
 * largest language, so the bar never has a gap. */
export function languageShares(languages: readonly LanguageInfo[]): LanguageShare[] {
  const total = languages.reduce((sum, item) => sum + item.loc, 0);
  if (total === 0) {
    return [];
  }
  const shares = languages.map((item) => ({
    language: item.language,
    name: languageName(item.language),
    tone: LANGUAGE_TONE[item.language] ?? LANGUAGE_TONE_OTHER,
    percent: Math.round((item.loc / total) * PERCENT),
  }));
  const gap = PERCENT - shares.reduce((sum, item) => sum + item.percent, 0);
  const largest = shares[0];
  if (largest) {
    largest.percent += gap;
  }
  return shares.filter((item) => item.percent > 0);
}

/** "3 min ago" or "yesterday", in the given language. `created` is in seconds since 1970. */
export function ago(created: number, nowMs: number, language: string): string {
  const seconds = Math.max(0, Math.round(nowMs / MS_PER_SECOND - created));
  const format = new Intl.RelativeTimeFormat(language, RELATIVE_OPTIONS);
  for (const [unit, size] of TIME_UNITS) {
    if (seconds >= size) {
      return format.format(-Math.floor(seconds / size), unit);
    }
  }
  return format.format(-seconds, SECOND_UNIT);
}

/** A whole number with the thousands separators of the language. */
export function count(value: number, language: string): string {
  return new Intl.NumberFormat(language).format(value);
}

/** Seconds with one decimal, in the notation of the language. */
export function seconds(value: number, language: string): string {
  return new Intl.NumberFormat(language, {
    minimumFractionDigits: SECONDS_DECIMALS,
    maximumFractionDigits: SECONDS_DECIMALS,
  }).format(value);
}

/** The parse time in seconds with two decimals and the unit, in the notation of the language. */
export function parseTime(micros: number, language: string): string {
  const value = new Intl.NumberFormat(language, {
    minimumFractionDigits: PARSE_TIME_DECIMALS,
    maximumFractionDigits: PARSE_TIME_DECIMALS,
  }).format(micros / MICROS_PER_SECOND);
  return `${value} s`;
}
