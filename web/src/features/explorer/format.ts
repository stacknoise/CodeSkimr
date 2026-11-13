import { LINE_RANGE_SEPARATOR, METRIC_DECIMALS, MISSING } from "./consts";

/** A metric value in the number format of the language: whole numbers without decimals. */
export function metricValue(value: number | undefined, language: string): string {
  if (value === undefined) {
    return MISSING;
  }
  return new Intl.NumberFormat(language, {
    maximumFractionDigits: Number.isInteger(value) ? 0 : METRIC_DECIMALS,
  }).format(value);
}

/** The lines of a declaration as the source column shows them: `61–65`, or `61` for one line. */
export function lineRange(first: number, last: number): string {
  return last === first ? String(first) : `${first}${LINE_RANGE_SEPARATOR}${last}`;
}
