import { For } from "solid-js";
import { currentLanguage } from "../../shared/i18n/i18n";
import { cx } from "../../shared/ui/cx";
import { tilesOf } from "./metrics";
import { ratingClass } from "./ratingClass";
import type { SymbolInfo, Thresholds } from "./types";
import styles from "./MetricStrip.module.css";

/** The grid of ten metrics of the selected symbol (mock 2); the colour follows the thresholds of
 * the configuration, the number says the same, so colour is not the only carrier. */
export function MetricStrip(props: { info: SymbolInfo; thresholds: Thresholds | null }) {
  const tiles = () => tilesOf(props.info, props.thresholds, currentLanguage());
  return (
    <div class={styles.strip}>
      <For each={tiles()}>
        {(tile) => (
          <div class={styles.tile} title={tile.tip}>
            <div class={cx(styles.value, ratingClass(tile.rating, styles))}>{tile.value}</div>
            <div class={styles.label}>{tile.label}</div>
          </div>
        )}
      </For>
    </div>
  );
}
