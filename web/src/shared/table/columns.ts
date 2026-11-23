import { createSignal, type JSX } from "solid-js";
import { readSetting, writeSetting } from "../settings/storage";
import { clampSize, type PaneBounds } from "../layout/paneSize";
import {
  COLUMN_ATTRIBUTE,
  COLUMN_MAX_PX,
  COLUMN_MIN_PX,
  COLUMN_SELECTOR,
  FIXED_LAYOUT,
  NO_WIDTHS,
  PIXEL_UNIT,
  ROW_SELECTOR,
  TABLE_STORAGE_PREFIX,
} from "./consts";

/** The widths of the columns by column id, in pixels. Empty while the table sizes itself. */
export type Widths = Record<string, number>;

/** The limits of every column of a table. */
export const COLUMN_BOUNDS: Omit<PaneBounds, "initial"> = {
  min: COLUMN_MIN_PX,
  max: COLUMN_MAX_PX,
};

/** What a handle changes: the width of one column. */
export interface ColumnWidth {
  size: () => number;
  bounds: PaneBounds;
  set: (value: number) => void;
  reset: () => void;
}

/** Reads the widths that were stored; anything that is not a width is dropped. */
export function parseWidths(text: string | null, ids: readonly string[]): Widths {
  try {
    const raw: unknown = JSON.parse(text ?? NO_WIDTHS);
    const entries = Object.entries(raw as Record<string, unknown>).filter(
      ([id, value]) => ids.includes(id) && typeof value === "number" && Number.isFinite(value),
    );
    return Object.fromEntries(
      entries.map(([id, value]) => [
        id,
        clampSize(value as number, { initial: 0, ...COLUMN_BOUNDS }),
      ]),
    );
  } catch {
    return {};
  }
}

/** The widths of the cells of the head row that `from` lies in, as the browser shows them now. */
function measured(from: Element): Widths {
  const row = from.closest(ROW_SELECTOR);
  const cells = Array.from(row?.querySelectorAll<HTMLElement>(COLUMN_SELECTOR) ?? []);
  return Object.fromEntries(
    cells.map((cell) => [
      cell.getAttribute(COLUMN_ATTRIBUTE) ?? "",
      clampSize(cell.offsetWidth, { initial: COLUMN_MIN_PX, ...COLUMN_BOUNDS }),
    ]),
  );
}

/** The column widths of one table. A table sizes itself until the user changes a column; then the
 * widths of all columns are fixed (taken from what is on the screen) and kept in the browser. */
export function createTableColumns(name: string, ids: readonly string[]) {
  const key = TABLE_STORAGE_PREFIX + name;
  const [widths, setWidths] = createSignal<Widths>(parseWidths(readSetting(key), ids));
  const keep = (next: Widths) => {
    setWidths(next);
    writeSetting(key, Object.keys(next).length === 0 ? null : JSON.stringify(next));
  };
  const fixed = () => ids.every((id) => widths()[id] !== undefined);
  const total = () => ids.reduce((sum, id) => sum + (widths()[id] ?? 0), 0);
  const bounds = { initial: COLUMN_MIN_PX, ...COLUMN_BOUNDS };
  return {
    ids,
    fixed,
    width: (id: string) => widths()[id],
    total,
    /** Fixes the widths of all columns to what the head row shows now. */
    freeze: (from: Element) => {
      if (fixed()) {
        return;
      }
      keep(measured(from));
    },
    /** The handle of one column. */
    column: (id: string): ColumnWidth => ({
      size: () => widths()[id] ?? 0,
      bounds,
      set: (value) => keep({ ...widths(), [id]: clampSize(value, bounds) }),
      reset: () => keep({}),
    }),
    /** The `style` of the table: fixed layout and a width that fits all columns once they are fixed. */
    style: (): JSX.CSSProperties =>
      fixed()
        ? {
            "table-layout": FIXED_LAYOUT,
            "min-width": `${total()}${PIXEL_UNIT}`,
          }
        : {},
  };
}

/** What `createTableColumns` returns. */
export type TableColumns = ReturnType<typeof createTableColumns>;
