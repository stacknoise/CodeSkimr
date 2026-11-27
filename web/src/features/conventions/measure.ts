import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { CODE } from "./consts";
import type { Finding, Place } from "./types";

/** A place as `file:line`. */
export function placeText(place: Pick<Place, "path" | "line">, separator: string): string {
  return `${place.path}${separator}${place.line}`;
}

/** What a finding measured and what its rule allows, in words. A rule that is not listed has no
 * measure (the text of its code says it all). */
export function measureText(finding: Finding, separator: string): string {
  switch (finding.code) {
    case CODE.lines:
      return t(I18N_KEYS.CONVENTIONS_MEASURE_LINES, {
        value: finding.lines ?? 0,
        max: finding.max_lines ?? 0,
      });
    case CODE.complexity:
      return t(I18N_KEYS.CONVENTIONS_MEASURE_COMPLEXITY, {
        value: finding.value ?? 0,
        max: finding.max_value ?? 0,
      });
    case CODE.parameters:
      return t(I18N_KEYS.CONVENTIONS_MEASURE_PARAMETERS, {
        value: finding.value ?? 0,
        max: finding.max_value ?? 0,
      });
    case CODE.literals:
      return t(I18N_KEYS.CONVENTIONS_MEASURE_LITERALS, { value: finding.literals ?? 0 });
    case CODE.duplicate:
      return t(I18N_KEYS.CONVENTIONS_MEASURE_DUPLICATE, {
        percent: finding.similarity_percent ?? 0,
        place: finding.duplicate_of === null ? "" : placeText(finding.duplicate_of, separator),
      });
    default:
      return "";
  }
}
