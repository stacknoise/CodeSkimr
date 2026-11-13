import { For } from "solid-js";
import { currentLanguage, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { count, languageName } from "../projects/format";
import type { LanguageInfo } from "../projects/types";
import styles from "./Languages.module.css";

/** The languages of the project with files and lines (mock 5, language table). Tier, symbols and
 * parse time per language are not reported by the server yet, so they are not shown. */
export function Languages(props: { languages: readonly LanguageInfo[] }) {
  return (
    <div class={styles.wrap}>
      <table class={styles.table}>
        <thead>
          <tr>
            <th>{t(I18N_KEYS.SCAN_COLUMN_LANGUAGE)}</th>
            <th class={styles.number}>{t(I18N_KEYS.SCAN_COLUMN_FILES)}</th>
            <th class={styles.number}>{t(I18N_KEYS.SCAN_COLUMN_LINES)}</th>
          </tr>
        </thead>
        <tbody>
          <For each={props.languages}>
            {(item) => (
              <tr>
                <td class={styles.name}>{languageName(item.language)}</td>
                <td class={styles.number}>{count(item.files, currentLanguage())}</td>
                <td class={styles.number}>{count(item.loc, currentLanguage())}</td>
              </tr>
            )}
          </For>
        </tbody>
      </table>
    </div>
  );
}
