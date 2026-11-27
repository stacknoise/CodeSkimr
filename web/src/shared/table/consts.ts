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
/** The key that takes the widths back to the first ones (the double click does the same). */
export const COLUMN_RESET_KEY = "Home";
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
/** The two directions of a sort, as `aria-sort` names them. */
export const SORT_DIRECTION = { ascending: "ascending", descending: "descending" } as const;
export type SortDirection = (typeof SORT_DIRECTION)[keyof typeof SORT_DIRECTION];
/** What a sorted column head shows, and the sign a direction gives to a comparison. */
export const SORT_MARK = { ascending: "▲", descending: "▼" } as const;
export const SORT_SIGN = { ascending: 1, descending: -1 } as const;
/** Compares texts without regard to case and accents, and digits by their value. */
export const TEXT_COLLATION: Intl.CollatorOptions = { sensitivity: "base", numeric: true };
