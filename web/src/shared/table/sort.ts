import { SORT_DIRECTION, SORT_SIGN, TEXT_COLLATION, type SortDirection } from "./consts";

/** How a table is sorted: by the column with this id, in this direction. */
export interface SortState {
  column: string;
  direction: SortDirection;
}

function opposite(direction: SortDirection): SortDirection {
  return direction === SORT_DIRECTION.ascending
    ? SORT_DIRECTION.descending
    : SORT_DIRECTION.ascending;
}

/** The sort after a press on a column head: another column starts with its natural direction
 * (texts from A, numbers largest first); the same column turns around once and the next press
 * removes the sort (`null`, the table keeps its own order). */
export function pressSort(
  current: SortState | null,
  column: string,
  numeric: boolean,
): SortState | null {
  const first = numeric ? SORT_DIRECTION.descending : SORT_DIRECTION.ascending;
  if (current?.column !== column) {
    return { column, direction: first };
  }
  return current.direction === first ? { column, direction: opposite(first) } : null;
}

/** Compares two values of a column: numbers by size, texts the way the language sorts them. A
 * missing value (`null`) comes last in either direction. */
export function compareValues(
  a: string | number | null,
  b: string | number | null,
  direction: SortDirection,
  language: string,
): number {
  if (a === null || b === null) {
    return Number(a === null) - Number(b === null);
  }
  const order =
    typeof a === "number" && typeof b === "number"
      ? a - b
      : String(a).localeCompare(String(b), language, TEXT_COLLATION);
  return order * SORT_SIGN[direction];
}
