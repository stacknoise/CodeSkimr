import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { cx } from "../../shared/ui/cx";
import styles from "./Legend.module.css";

/** The legend and the totals below the map (mock 3). */
export function Legend(props: { layers: number; links: number; violations: number }) {
  return (
    <div class={styles.legend}>
      <span>
        <i class={cx(styles.swatch, styles.out)} /> {t(I18N_KEYS.GRAPH_LEGEND_USES)}
      </span>
      <span>
        <i class={cx(styles.swatch, styles.in)} /> {t(I18N_KEYS.GRAPH_LEGEND_USED_BY)}
      </span>
      <span>
        <i class={cx(styles.swatch, styles.violation)} />{" "}
        {t(I18N_KEYS.GRAPH_LEGEND_VIOLATION, { count: props.violations })}
      </span>
      <span class={styles.total}>
        {t(I18N_KEYS.GRAPH_SUMMARY, { nodes: props.layers, edges: props.links })}
      </span>
    </div>
  );
}
