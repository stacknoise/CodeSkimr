import { currentLanguage } from "../../shared/i18n/i18n";
import { tilesOf } from "./metrics";
import { TileGrid } from "./TileGrid";
import type { SymbolInfo, Thresholds } from "./types";
import styles from "./MetricStrip.module.css";

/** The grid of ten metrics of the selected symbol (mock 2); the colour follows the thresholds of
 * the configuration, the number says the same, so colour is not the only carrier. */
export function MetricStrip(props: { info: SymbolInfo; thresholds: Thresholds | null }) {
  const tiles = () => tilesOf(props.info, props.thresholds, currentLanguage());
  return <TileGrid tiles={tiles()} classes={styles} />;
}
