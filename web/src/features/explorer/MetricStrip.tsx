import { For } from "solid-js";
import { currentLanguage, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { cx } from "../../shared/ui/cx";
import { METRICS, RATING } from "./consts";
import { metricValue } from "./format";
import type { SymbolInfo } from "./types";
import styles from "./MetricStrip.module.css";

const RATING_CLASS: Record<string, string | undefined> = {
  [RATING.good]: styles.good,
  [RATING.medium]: styles.medium,
  [RATING.poor]: styles.poor,
};

/** The grid of metrics of the selected symbol (mock 2); the colour follows the thresholds of
 * the configuration, the number says the same, so colour is not the only carrier. */
export function MetricStrip(props: { info: SymbolInfo }) {
  const tiles = () =>
    METRICS.map((metric) => {
      const found = props.info.metrics.find((row) => row.kind === metric.kind);
      return {
        key: metric.kind,
        label: t(metric.label),
        tip: t(metric.tip),
        value: metricValue(found?.value, currentLanguage()),
        rating: found?.rating ?? null,
      };
    });
  return (
    <div class={styles.strip}>
      <div class={styles.tile} title={t(I18N_KEYS.EXPLORER_METRIC_MEMBERS_TIP)}>
        <div class={styles.value}>{metricValue(props.info.members.length, currentLanguage())}</div>
        <div class={styles.label}>{t(I18N_KEYS.EXPLORER_METRIC_MEMBERS)}</div>
      </div>
      <For each={tiles()}>
        {(tile) => (
          <div class={styles.tile} title={tile.tip}>
            <div class={cx(styles.value, RATING_CLASS[tile.rating ?? ""])}>{tile.value}</div>
            <div class={styles.label}>{tile.label}</div>
          </div>
        )}
      </For>
    </div>
  );
}
