import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { cx } from "../../shared/ui/cx";
import styles from "./Legend.module.css";

/** The legend of the usage graph: line colours and what the number on a card counts (mock 6). */
export function Legend() {
  return (
    <div class={styles.legend}>
      <span>
        <i class={cx(styles.swatch, styles.calls)} /> {t(I18N_KEYS.USAGES_KIND_CALLS)}
      </span>
      <span>
        <i class={cx(styles.swatch, styles.uses)} /> {t(I18N_KEYS.USAGES_KIND_USES)}
      </span>
      <span>
        <i class={cx(styles.swatch, styles.presumed)} /> {t(I18N_KEYS.USAGES_KIND_PRESUMED)}
      </span>
      <span class={styles.note}>{t(I18N_KEYS.USAGES_LEGEND_COUNT)}</span>
    </div>
  );
}
