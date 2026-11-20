import { createSignal } from "solid-js";
import { readSetting, writeSetting } from "../settings/storage";
import { DECIMAL, LAYOUT_STORAGE_PREFIX } from "./consts";

/** The limits and the first value of a pane size, in pixels. */
export interface PaneBounds {
  initial: number;
  min: number;
  max: number;
}

/** Keeps `value` within the bounds. */
export function clampSize(value: number, bounds: PaneBounds): number {
  return Math.min(bounds.max, Math.max(bounds.min, Math.round(value)));
}

/** The size that was stored for `key`, or the first value if there is none or it is not a number. */
export function storedSize(key: string, bounds: PaneBounds): number {
  const stored = readSetting(LAYOUT_STORAGE_PREFIX + key);
  const value = stored === null ? Number.NaN : Number.parseInt(stored, DECIMAL);
  return Number.isFinite(value) ? clampSize(value, bounds) : bounds.initial;
}

/** The size of a pane that the user can change: a signal that remembers its value in the browser,
 * so the choice survives a reload. `reset` goes back to the first value and forgets the choice. */
export function createPaneSize(key: string, bounds: PaneBounds) {
  const [size, setSize] = createSignal(storedSize(key, bounds));
  return {
    size,
    bounds,
    set: (value: number) => {
      const next = clampSize(value, bounds);
      setSize(next);
      writeSetting(LAYOUT_STORAGE_PREFIX + key, String(next));
    },
    reset: () => {
      setSize(bounds.initial);
      writeSetting(LAYOUT_STORAGE_PREFIX + key, null);
    },
  };
}

/** What `createPaneSize` returns. */
export type PaneSize = ReturnType<typeof createPaneSize>;
