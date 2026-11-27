import { showLayers } from "../rules/tab";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import styles from "./LayersScreen.module.css";

/** The note that a project has no layers yet, with the button that opens the editor. It stands
 * where the missing layers show: the overview of the explorer and the empty dependency map. */
export function LayersHint() {
  return (
    <div class={styles.hint}>
      <strong>{t(I18N_KEYS.LAYERS_HINT_TITLE)}</strong>
      <span>{t(I18N_KEYS.LAYERS_HINT_TEXT)}</span>
      <button type="button" class={styles.primary} onClick={showLayers}>
        {t(I18N_KEYS.LAYERS_HINT_ACTION)}
      </button>
    </div>
  );
}
