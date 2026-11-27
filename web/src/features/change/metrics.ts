import { currentLanguage } from "../../shared/i18n/i18n";
import { DELTA_SIGN, METRIC_ALL, METRIC_DECIMALS, OBJECT_KEY_JOIN } from "./consts";
import type { ChangeAnswer, Listed, MetricChangeOut, MetricMoveOut, MetricsAnswer } from "./types";

/** The objects of a list that moved in `metric`, with only the moves of that metric; the whole
 * list for the choice "all". */
export function narrowedTo(list: Listed<MetricChangeOut>, metric: string): MetricChangeOut[] {
  if (metric === METRIC_ALL) {
    return list.items;
  }
  return list.items
    .map((item) => ({ ...item, moves: item.moves.filter((move) => move.metric === metric) }))
    .filter((item) => item.moves.length > 0);
}

/** A value of a metric with at most two decimals, in the language of the screen. */
export function metricValue(value: number): string {
  return new Intl.NumberFormat(currentLanguage(), {
    maximumFractionDigits: METRIC_DECIMALS,
  }).format(value);
}

/** The size of a move with its sign: `+8` or `−13`. */
export function deltaText(move: MetricMoveOut): string {
  const size = move.to - move.from;
  return `${size < 0 ? DELTA_SIGN.down : DELTA_SIGN.up}${metricValue(Math.abs(size))}`;
}

/** The key that tells one object from another across the two lists. */
function objectKey(item: MetricChangeOut): string {
  return [item.kind, item.path, item.name].join(OBJECT_KEY_JOIN);
}

/** How many objects the newer scan has on the level that did not move: those looked at minus the
 * objects of the two lists (a mixed object is on both). `null` if a list was cut, because then
 * the objects that moved are not all known. */
export function unchangedCount(answer: MetricsAnswer): number | null {
  if (answer.worse.is_cut || answer.better.is_cut) {
    return null;
  }
  const moved = new Set([...answer.worse.items, ...answer.better.items].map(objectKey));
  return Math.max(answer.looked_at - moved.size, 0);
}

/** How many objects of a list crossed the bound of the rating. */
export function crossingCount(items: readonly MetricChangeOut[]): number {
  return items.filter((item) => item.crosses).length;
}

/** The objects of the comparison that cannot be compared: new and removed ones. */
export function notComparable(answer: ChangeAnswer): { added: number; removed: number } {
  return { added: answer.added.total, removed: answer.removed.total };
}
