import { createPaneSize } from "../../shared/layout/paneSize";
import { CODE_COLUMN_WIDTH, COLUMN_JOIN, HIT_COLUMN_KEY, HIT_COLUMNS, PIXEL_UNIT } from "./consts";

/** The widths of the columns of the hit list by column id; the code column has none. */
export const columnWidths = Object.fromEntries(
  HIT_COLUMNS.flatMap((column) =>
    column.bounds === null
      ? []
      : [[column.id, createPaneSize(HIT_COLUMN_KEY + column.id, column.bounds)]],
  ),
);

/** The `grid-template-columns` of the head and of every row of the list. */
export function gridColumns(): string {
  return HIT_COLUMNS.map((column) => {
    const width = columnWidths[column.id];
    return width === undefined ? CODE_COLUMN_WIDTH : `${width.size()}${PIXEL_UNIT}`;
  }).join(COLUMN_JOIN);
}
