import type { MessageKey } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import type { ThemeName } from "../../shared/theme/tokens";

/** Name and one-line description of every theme, as message keys. A theme that is added to
 * `design/tokens.toml` needs an entry here and texts in `locales/`; the compiler asks for both. */
export const THEME_TEXTS: Record<ThemeName, { name: MessageKey; note: MessageKey }> = {
  hell: {
    name: I18N_KEYS.SETTINGS_THEME_NAME_HELL,
    note: I18N_KEYS.SETTINGS_THEME_NOTE_HELL,
  },
  petrol: {
    name: I18N_KEYS.SETTINGS_THEME_NAME_PETROL,
    note: I18N_KEYS.SETTINGS_THEME_NOTE_PETROL,
  },
  dunkel: {
    name: I18N_KEYS.SETTINGS_THEME_NAME_DUNKEL,
    note: I18N_KEYS.SETTINGS_THEME_NOTE_DUNKEL,
  },
};

/** Tokens shown as swatches under each preview, in the order of the mock. */
export const SWATCH_TOKENS = ["bg", "surface", "ink", "accent", "good", "warn", "bad"] as const;

/** Language names are shown in their own language. */
export const LANGUAGE_NAME_KEYS = {
  de: I18N_KEYS.SETTINGS_LANGUAGE_NAME_DE,
  en: I18N_KEYS.SETTINGS_LANGUAGE_NAME_EN,
} as const;

/** The projects of the miniature in the preview (sample data, as in the mock). */
export const PREVIEW_NEW_FILES = 12;

/** Ids that tie a label to its field. */
export const FOLLOW_SYSTEM_ID = "follow-system";
export const LANGUAGE_FIELD_ID = "language";
/** Name of the second sample project in the preview. */
export const PREVIEW_OTHER_PROJECT = "haac-bridge";
