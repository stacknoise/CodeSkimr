import { createPaneSize } from "../../shared/layout/paneSize";
import {
  COLUMN_JOIN,
  HIT_COLUMN_KEY,
  HIT_COLUMNS,
  LEFTOVER_COLUMN_WIDTH,
  PIXEL_UNIT,
} from "./consts";

/** The widths of the columns of the hit list by column id. */
export const columnWidths = Object.fromEntries(
  HIT_COLUMNS.map((column) => [
    column.id,
    createPaneSize(HIT_COLUMN_KEY + column.id, column.bounds),
  ]),
);

/** The `grid-template-columns` of the head and of every row of the list: a width for every column and
 * a last track that takes what is left, so that the last column can be widened too. */
export function gridColumns(): string {
  return [
    ...HIT_COLUMNS.map((column) => `${columnWidths[column.id]?.size() ?? 0}${PIXEL_UNIT}`),
    LEFTOVER_COLUMN_WIDTH,
  ].join(COLUMN_JOIN);
}
