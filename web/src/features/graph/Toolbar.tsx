import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import styles from "./Toolbar.module.css";

/** The switches above the map (mock 3): "all connections weak", "violations only" and the hint on
 * the mouse; `F` fits the picture to the view. */
export function Toolbar(props: {
  weak: boolean;
  onWeak: (on: boolean) => void;
  violationsOnly: boolean;
  onViolationsOnly: (on: boolean) => void;
  onFit: () => void;
}) {
  return (
    <div class={styles.bar}>
      <label class={styles.toggle}>
        <input
          type="checkbox"
          checked={props.weak}
          onChange={(event) => props.onWeak(event.currentTarget.checked)}
        />
        {t(I18N_KEYS.GRAPH_TOGGLE_WEAK)}
      </label>
      <label class={styles.toggle}>
        <input
          type="checkbox"
          checked={props.violationsOnly}
          onChange={(event) => props.onViolationsOnly(event.currentTarget.checked)}
        />
        {t(I18N_KEYS.GRAPH_TOGGLE_VIOLATIONS_ONLY)}
      </label>
      <button type="button" class={styles.fit} onClick={props.onFit}>
        {t(I18N_KEYS.GRAPH_ACTION_FIT)} <kbd class={styles.kbd}>{t(I18N_KEYS.GRAPH_FIT_KEY)}</kbd>
      </button>
      <span class={styles.hint}>{t(I18N_KEYS.GRAPH_HINT_NAVIGATION)}</span>
    </div>
  );
}
