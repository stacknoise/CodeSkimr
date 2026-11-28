import type { MessageKey } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { FLOAT_PATTERN, INT_PATTERN, SWITCH_OFF, SWITCH_ON, VALUE_KIND } from "./consts";

/** One of the kinds of value the dialog edits. */
export type ValueKind = (typeof VALUE_KIND)[keyof typeof VALUE_KIND];

/** The kind of a value from the TOML text of its default: a switch, a whole number or a number
 * with a fraction. */
export function kindOf(defaultText: string): ValueKind {
  if (defaultText === SWITCH_ON || defaultText === SWITCH_OFF) {
    return VALUE_KIND.switch;
  }
  return INT_PATTERN.test(defaultText) ? VALUE_KIND.int : VALUE_KIND.float;
}

/** The words that name a kind in the hint under the field and in the message about a bad entry. */
export const KIND_TEXTS: Record<ValueKind, MessageKey> = {
  switch: I18N_KEYS.SETTINGS_SETTING_KIND_SWITCH,
  int: I18N_KEYS.SETTINGS_SETTING_KIND_INT,
  float: I18N_KEYS.SETTINGS_SETTING_KIND_FLOAT,
};

/** The value the field text stands for, or `null` if the text is no value of the kind. */
export function parseEntry(kind: ValueKind, text: string): number | boolean | null {
  const trimmed = text.trim();
  if (kind === VALUE_KIND.switch) {
    return trimmed === SWITCH_ON ? true : trimmed === SWITCH_OFF ? false : null;
  }
  const pattern = kind === VALUE_KIND.int ? INT_PATTERN : FLOAT_PATTERN;
  return pattern.test(trimmed) ? Number(trimmed) : null;
}
