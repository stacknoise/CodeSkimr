import {
  HALVES,
  HISTOGRAM,
  HOTSPOTS,
  LOG_BASE,
  PATH_SEPARATOR,
  PATH_TAIL,
  USAGE_BAR,
} from "./consts";
import type { Bar, Row } from "./types";

/** The last two parts of a path, which is what a card has room for. */
export function shortPath(path: string): string {
  return path.split(PATH_SEPARATOR).slice(-PATH_TAIL).join(PATH_SEPARATOR);
}

/** The widths of the certain and the probable part of a usage bar whose full width stands for
 * `most` usages. */
export function usageWidths(row: Row, most: number): { certain: number; probable: number } {
  const scale = most > 0 ? USAGE_BAR.width / most : 0;
  return { certain: row.certain * scale, probable: (row.total - row.certain) * scale };
}

/** The most usages of any row, by the upper bound. */
export function mostUsages(rows: readonly Row[]): number {
  return rows.reduce((most, row) => Math.max(most, row.total), 0);
}

function logarithm(value: number): number {
  return Math.log(value + 1) / Math.log(LOG_BASE);
}

/** One bar of the histogram as a rectangle. */
export interface BarShape {
  bar: Bar;
  x: number;
  /** The middle of the bar, where its texts stand. */
  middle: number;
  y: number;
  width: number;
  height: number;
}

/** Places the bars of the histogram: equal widths, heights on a logarithmic scale so that the few
 * complex functions stay visible next to the many simple ones. */
export function histogramShapes(bars: readonly Bar[]): BarShape[] {
  const room = HISTOGRAM.height - HISTOGRAM.top - HISTOGRAM.bottom;
  const most = logarithm(bars.reduce((top, bar) => Math.max(top, bar.count), 0));
  const step = bars.length > 0 ? HISTOGRAM.width / bars.length : 0;
  return bars.map((bar, index) => {
    const share = most > 0 ? logarithm(bar.count) / most : 0;
    const height = Math.max(share * room, HISTOGRAM.minBar);
    return {
      bar,
      x: index * step + HISTOGRAM.gap / HALVES,
      middle: index * step + step / HALVES,
      y: HISTOGRAM.top + room - height,
      width: Math.max(step - HISTOGRAM.gap, 1),
      height,
    };
  });
}

/** One function of the hotspot picture: the dot at its certain usages, the line up to all. */
export interface Spot {
  row: Row;
  x: number;
  y: number;
  top: number;
}

/** The scales of the hotspot picture. */
export interface SpotScale {
  maxX: number;
  maxY: number;
  x: (complexity: number) => number;
  y: (usages: number) => number;
}

/** The scales for these rows: complexity to the right, usages (logarithmic) upwards. */
export function spotScale(rows: readonly Row[]): SpotScale {
  const maxX = rows.reduce<number>(
    (top, row) => Math.max(top, row.complexity ?? 0),
    HOTSPOTS.minMaxX,
  );
  const maxY = rows.reduce<number>((top, row) => Math.max(top, row.total), HOTSPOTS.minMaxY);
  const wide = HOTSPOTS.width - HOTSPOTS.left - HOTSPOTS.right;
  const high = HOTSPOTS.height - HOTSPOTS.top - HOTSPOTS.bottom;
  return {
    maxX,
    maxY,
    x: (complexity) => HOTSPOTS.left + (complexity / maxX) * wide,
    y: (usages) => HOTSPOTS.top + high - (logarithm(usages) / logarithm(maxY)) * high,
  };
}

/** Places the functions of the hotspot picture. */
export function spots(rows: readonly Row[], scale: SpotScale): Spot[] {
  return rows
    .filter((row) => row.complexity !== null)
    .map((row) => ({
      row,
      x: scale.x(row.complexity ?? 0),
      y: scale.y(row.certain),
      top: scale.y(row.total),
    }));
}

/** The steps of the usages axis: 1, 10, 100 and so on up to the largest value. */
export function usageTicks(maxY: number): number[] {
  const ticks = [0];
  for (let tick = 1; tick <= maxY; tick *= LOG_BASE) {
    ticks.push(tick);
  }
  return ticks;
}

/** The steps of the complexity axis. */
export function complexityTicks(maxX: number): number[] {
  const step = Math.max(Math.ceil(maxX / HOTSPOTS.ticksX), 1);
  const ticks = [];
  for (let tick = 0; tick <= maxX; tick += step) {
    ticks.push(tick);
  }
  return ticks;
}
