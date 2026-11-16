import {
  DEFAULT_SORT,
  NAME_COLLATION,
  NUMBER_COLUMNS,
  SORT_COLUMN,
  SORT_DIRECTION,
  SORT_SIGN,
  type SortColumn,
  type SortDirection,
} from "./consts";
import type { MemberRow } from "./types";

/** How the member table is sorted. */
export interface MemberSort {
  column: SortColumn;
  direction: SortDirection;
}

/** The value of a row in a column, `null` where the row has none. */
function valueOf(row: MemberRow, column: SortColumn): string | number | null {
  switch (column) {
    case SORT_COLUMN.signature:
      return row.signature;
    case SORT_COLUMN.access:
      return row.visibility;
    case SORT_COLUMN.complexity:
      return row.complexity;
    case SORT_COLUMN.usages:
      return row.usages;
    case SORT_COLUMN.lines:
      return row.first_line;
    default:
      return row.name;
  }
}

function compare(a: string | number, b: string | number): number {
  return typeof a === "number" && typeof b === "number"
    ? a - b
    : String(a).localeCompare(String(b), undefined, NAME_COLLATION);
}

/** The rows in the order of `sort`. A row without a value in the column comes last in either
 * direction; equal rows keep to the order of their names, then of their lines. */
export function sortMembers(rows: readonly MemberRow[], sort: MemberSort): MemberRow[] {
  const sign = SORT_SIGN[sort.direction];
  return [...rows].sort((a, b) => {
    const left = valueOf(a, sort.column);
    const right = valueOf(b, sort.column);
    if (left === null || right === null) {
      return left === right ? 0 : left === null ? SORT_SIGN.ascending : SORT_SIGN.descending;
    }
    return sign * compare(left, right) || compare(a.name, b.name) || a.first_line - b.first_line;
  });
}

/** The sort after a press on the head of `column`: the same column turns around, another one
 * starts with its natural direction (numbers largest first). */
export function pressColumn(current: MemberSort, column: SortColumn): MemberSort {
  if (current.column === column) {
    return {
      column,
      direction:
        current.direction === SORT_DIRECTION.ascending
          ? SORT_DIRECTION.descending
          : SORT_DIRECTION.ascending,
    };
  }
  return {
    column,
    direction: NUMBER_COLUMNS.includes(column)
      ? SORT_DIRECTION.descending
      : SORT_DIRECTION.ascending,
  };
}

export const INITIAL_SORT: MemberSort = { ...DEFAULT_SORT };
