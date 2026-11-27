import { createSignal } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { compareValues, type SortState } from "../../shared/table/sort";
import { COMPARE_COLUMN, SYMBOL_COLUMN, VISIBILITY_TEXT } from "./consts";
import { openCount, pillsOf, type CompareRow } from "./rows";
import type { ImpactRow } from "./types";

const [impactSort, setImpactSort] = createSignal<SortState | null>(null);
const [compareSort, setCompareSort] = createSignal<SortState | null>(null);

/** How the table of the impact is sorted; `null` keeps the order of the answer. */
export const impactOrder = impactSort;
/** Sorts the table of the impact. */
export const sortImpactBy = setImpactSort;
/** How the list of the comparison is sorted; `null` keeps the order of the answer. */
export const compareOrder = compareSort;
/** Sorts the list of the comparison. */
export const sortCompareBy = setCompareSort;

/** Sorts rows by the value of the column of the sort; rows that are equal, and all rows without a
 * sort, keep the order they have. */
export function sortedBy<Row>(
  rows: readonly Row[],
  sort: SortState | null,
  valueOf: (row: Row, column: string) => string | number | null,
  language: string,
): Row[] {
  if (sort === null) {
    return [...rows];
  }
  return [...rows].sort((a, b) =>
    compareValues(valueOf(a, sort.column), valueOf(b, sort.column), sort.direction, language),
  );
}

/** The text of the first pill of a row: what its column "Change" shows. */
function pillText(pills: ReturnType<typeof pillsOf>): string | null {
  const first = pills[0];
  return first === undefined ? null : t(first.label);
}

/** What a column of the table of the impact holds for a row, as it is shown. */
export function impactValue(row: ImpactRow, column: string): string | number | null {
  const values: Record<string, () => string | number | null> = {
    [SYMBOL_COLUMN.symbol]: () => row.symbol.qualified_name,
    [SYMBOL_COLUMN.change]: () => pillText(pillsOf(row)),
    [SYMBOL_COLUMN.visibility]: () => {
      const key = VISIBILITY_TEXT[row.symbol.visibility ?? ""];
      return key === undefined ? null : t(key);
    },
    [SYMBOL_COLUMN.usages]: () => row.impact?.usages.total ?? null,
    [SYMBOL_COLUMN.state]: () => (row.impact === null ? null : openCount(row)),
  };
  return values[column]?.() ?? null;
}

/** What a column of the list of the comparison holds for a row, as it is shown. */
export function compareValue(row: CompareRow, column: string): string | number | null {
  const values: Record<string, string | null> = {
    [COMPARE_COLUMN.change]: pillText(row.pills),
    [COMPARE_COLUMN.symbol]: row.name,
    [COMPARE_COLUMN.file]: row.where,
  };
  return values[column] ?? null;
}
