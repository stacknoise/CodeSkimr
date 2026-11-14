import { For } from "solid-js";
import { currentLanguage, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { count, languageName } from "../projects/format";
import type { LanguageInfo } from "../projects/types";
import styles from "./Languages.module.css";

const TIER_TEXT = {
  full: I18N_KEYS.SCAN_TIER_FULL,
  basic: I18N_KEYS.SCAN_TIER_BASIC,
} as const;

/** The languages of the project with tier, files, lines and symbols (mock 5, language table). The
 * parse time per language is not stored by the scan, so the column is left out. */
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
              </tr>
            )}
          </For>
        </tbody>
      </table>
    </div>
  );
}
