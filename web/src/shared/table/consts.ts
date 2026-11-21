/** Prefix of the keys under which the column widths of a table are kept in `localStorage`. */
export const TABLE_STORAGE_PREFIX = "codeskimr.table.";
/** The limits of a column, in pixels: narrower hides its head, wider is never needed. */
export const COLUMN_MIN_PX = 48;
export const COLUMN_MAX_PX = 1600;
/** Pixels a column grows or shrinks with one press of an arrow key, and with Shift held. */
export const COLUMN_KEY_STEP_PX = 12;
export const COLUMN_KEY_STEP_LARGE_PX = 48;
/** The keys that move a column handle, and by how many steps (a key not listed moves nothing). */
export const COLUMN_KEY_DELTA: Record<string, number> = { ArrowRight: 1, ArrowLeft: -1 };
/** No movement. */
export const NO_DELTA = 0;
/** The unit of a width, and the CSS that a table with fixed column widths needs. */
export const PIXEL_UNIT = "px";
export const FIXED_LAYOUT = "fixed" as const;
/** What marks a head cell, so that the widths can be read from the rendered table. */
export const COLUMN_ATTRIBUTE = "data-column";
export const COLUMN_SELECTOR = "th[data-column]";
export const ROW_SELECTOR = "tr";
/** The stored text that has no column (a table without widths). */
export const NO_WIDTHS = "{}";
