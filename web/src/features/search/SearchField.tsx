import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { openSearch } from "./search";
import styles from "./SearchField.module.css";

/** The search field of the top bar: it opens the quick search (blueprint 15.2). */
export function SearchField() {
  return (
    <button type="button" class={styles.search} onClick={openSearch}>
      <span>{t(I18N_KEYS.APP_SEARCH_PLACEHOLDER)}</span>
      <kbd>{t(I18N_KEYS.APP_SEARCH_SHORTCUT)}</kbd>
    </button>
  );
}
