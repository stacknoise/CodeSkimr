import { t } from "../../shared/i18n/i18n";
import { MISSING, TILE_SOURCE, TILES } from "./consts";
import { metricValue } from "./format";
import type { SymbolInfo, Thresholds } from "./types";

/** A tile of the metric strip, ready to show. */
export interface Tile {
  id: string;
  label: string;
  tip: string;
  value: string;
  /** `good`, `medium` or `poor` for metrics that have thresholds. */
  rating: string | null;
}

const LIMITED: readonly string[] = ["cc", "mi", "instability"];

/** The numbers a tip names, by metric; a dash for each while the thresholds are not known. */
function limits(
  id: string,
  thresholds: Thresholds | null,
): Record<string, number | string> | undefined {
  if (!LIMITED.includes(id)) {
    return undefined;
  }
  if (thresholds === null) {
    return { good: MISSING, medium: MISSING };
  }
  switch (id) {
    case "cc":
      return { good: thresholds.complexity_good_max, medium: thresholds.complexity_medium_max };
    case "mi":
      return {
        good: thresholds.maintainability_good_min,
        medium: thresholds.maintainability_medium_min,
      };
    default:
      return { good: thresholds.instability_stable_max, medium: thresholds.instability_medium_max };
  }
}

/** The tip of a metric column or tile, with its thresholds when it has some. */
export function tipFor(id: string, tip: Parameters<typeof t>[0], thresholds: Thresholds | null) {
  return t(tip, limits(id, thresholds));
}

function kindCount(info: SymbolInfo, kinds: readonly string[]): number {
  return info.members.filter((member) => kinds.includes(member.kind)).length;
}

/** The ten tiles of a symbol (blueprint B.2): member count, the stored metrics and the counts of
 * the member kinds. A metric the scan did not measure has no value. */
export function tilesOf(info: SymbolInfo, thresholds: Thresholds | null, language: string): Tile[] {
  return TILES.map((tile) => {
    const stored = info.metrics.find((row) => row.kind === tile.id);
    let value: number | undefined = stored?.value;
    if (tile.source === TILE_SOURCE.members) {
      value = info.members.length;
    } else if (tile.source === TILE_SOURCE.kinds) {
      value = kindCount(info, tile.kinds);
    }
    return {
      id: tile.id,
      label: t(tile.label),
      tip: tipFor(tile.id, tile.tip, thresholds),
      value: metricValue(value, language),
      rating: stored?.rating ?? null,
    };
  });
}
