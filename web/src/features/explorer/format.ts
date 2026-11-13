import { METRIC_DECIMALS, MISSING } from "./consts";

/** A metric value in the number format of the language: whole numbers without decimals. */
export function metricValue(value: number | undefined, language: string): string {
  if (value === undefined) {
    return MISSING;
  }
  return new Intl.NumberFormat(language, {
    maximumFractionDigits: Number.isInteger(value) ? 0 : METRIC_DECIMALS,
  }).format(value);
}
