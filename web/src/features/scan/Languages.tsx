import { For } from "solid-js";
import { currentLanguage, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { count, languageName, parseTime } from "../projects/format";
import type { LanguageInfo } from "../projects/types";
import styles from "./Languages.module.css";

const TIER_TEXT = {
  full: I18N_KEYS.SCAN_TIER_FULL,
  basic: I18N_KEYS.SCAN_TIER_BASIC,
} as const;

/** The languages of the project with tier, files, lines, symbols and the parse time of the last full
 * scan (mock 5, language table). */
export function Languages(props: { languages: readonly LanguageInfo[] }) {
  return (
    <div class={styles.wrap}>
      <table class={styles.table}>
        <thead>
          <tr>
            <th>{t(I18N_KEYS.SCAN_COLUMN_LANGUAGE)}</th>
            <th>{t(I18N_KEYS.SCAN_COLUMN_TIER)}</th>
            <th class={styles.number}>{t(I18N_KEYS.SCAN_COLUMN_FILES)}</th>
            <th class={styles.number}>{t(I18N_KEYS.SCAN_COLUMN_LINES)}</th>
            <th class={styles.number}>{t(I18N_KEYS.SCAN_COLUMN_SYMBOLS)}</th>
            <th class={styles.number}>{t(I18N_KEYS.SCAN_COLUMN_PARSE_TIME)}</th>
          </tr>
        </thead>
        <tbody>
          <For each={props.languages}>
            {(item) => (
              <tr>
                <td class={styles.name}>{languageName(item.language)}</td>
                <td>
                  {item.tier === null ? t(I18N_KEYS.SCAN_UNAVAILABLE) : t(TIER_TEXT[item.tier])}
                </td>
                <td class={styles.number}>{count(item.files, currentLanguage())}</td>
                <td class={styles.number}>{count(item.loc, currentLanguage())}</td>
                <td class={styles.number}>{count(item.symbols, currentLanguage())}</td>
                <td class={styles.number}>
                  {item.parse_micros === null
                    ? t(I18N_KEYS.SCAN_UNAVAILABLE)
                    : parseTime(item.parse_micros, currentLanguage())}
                </td>
              </tr>
            )}
          </For>
        </tbody>
      </table>
    </div>
  );
}
