import {
  CHIP,
  DEFAULT_SORT,
  NAME_COLLATION,
  METHOD_KINDS,
  NUMBER_COLUMNS,
  PROPERTY_KINDS,
  PUBLIC,
  SORT_COLUMN,
  SORT_DIRECTION,
  SORT_SIGN,
  type ChipId,
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
export function sortMembers(rows: readonly MemberRow[], sort: MemberSort | null): MemberRow[] {
  if (sort === null) {
    return [...rows].sort((a, b) => a.first_line - b.first_line || compare(a.name, b.name));
  }
  const sign = SORT_SIGN[sort.direction];
  return [...rows].sort((a, b) => {
    const left = valueOf(a, sort.column);
    const right = valueOf(b, sort.column);
    const byValue =
      left === null || right === null
        ? Number(left === null) - Number(right === null)
        : sign * compare(left, right);
    return byValue || compare(a.name, b.name) || a.first_line - b.first_line;
  });
}

/** The sort after a press on the head of `column`: another column starts with its natural
 * direction (numbers largest first); the same column turns around once and the next press
 * removes the sort (`null`, the rows keep to the order of the code). */
export function pressColumn(current: MemberSort | null, column: SortColumn): MemberSort | null {
  if (current?.column !== column) {
    return {
      column,
      direction: NUMBER_COLUMNS.includes(column)
        ? SORT_DIRECTION.descending
        : SORT_DIRECTION.ascending,
    };
  }
  const first = NUMBER_COLUMNS.includes(column)
    ? SORT_DIRECTION.descending
    : SORT_DIRECTION.ascending;
  return current.direction === first ? { column, direction: turned(first) } : null;
}

function turned(direction: SortDirection): SortDirection {
  return direction === SORT_DIRECTION.ascending
    ? SORT_DIRECTION.descending
    : SORT_DIRECTION.ascending;
}

export const INITIAL_SORT: MemberSort = { ...DEFAULT_SORT };

/** Whether the chip of the member table keeps a row. */
export function keeps(chip: ChipId, row: MemberRow): boolean {
  switch (chip) {
    case CHIP.properties:
      return PROPERTY_KINDS.includes(row.kind);
    case CHIP.methods:
      return METHOD_KINDS.includes(row.kind);
    case CHIP.public:
      return row.visibility === PUBLIC;
    default:
      return true;
  }
}

/** The rows of the member table: those the chip keeps, in the order of the sort (or of the code). */
export function tableRows(
  rows: readonly MemberRow[],
  chip: ChipId,
  sort: MemberSort | null,
): MemberRow[] {
  return sortMembers(
    rows.filter((row) => keeps(chip, row)),
    sort,
  );
}
