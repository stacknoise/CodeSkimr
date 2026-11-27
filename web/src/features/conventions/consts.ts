import type { SortState } from "../../shared/table/sort";

/** Id of the command of the tab (crates/app/src/ui_commands/consts.rs). */
export const COMMAND = { check: "conventions.check" } as const;

/** The key under which the column widths of the table of findings are kept. */
export const CONVENTIONS_TABLE_KEY = "conventions";

/** The columns of the table of findings. */
export const COLUMN = {
  rule: "rule",
  place: "place",
  symbol: "symbol",
  measure: "measure",
} as const;
export type Column = (typeof COLUMN)[keyof typeof COLUMN];
/** The columns in their order. */
export const COLUMNS: readonly Column[] = [
  COLUMN.rule,
  COLUMN.place,
  COLUMN.symbol,
  COLUMN.measure,
];
/** The columns whose values are numbers: they start with the largest. */
export const NUMBER_COLUMNS: readonly Column[] = [COLUMN.measure];
/** The table keeps the order of the server (by file, then by place) until a column is pressed. */
export const DEFAULT_SORT: SortState | null = null;

/** The codes of the rules whose measure has its own words (docs/conventions.md). */
export const CODE = {
  literals: "CHK-101",
  duplicate: "CHK-102",
  lines: "CHK-104",
  complexity: "CHK-105",
  parameters: "CHK-106",
} as const;

/** Separates the file from the line in a place. */
export const PLACE_SEPARATOR = ":";
/** How long the word on the copy button stays, in milliseconds. */
export const COPY_NOTE_MS = 2000;
/** The words the copy button can show instead of its label. */
export const COPY_NOTE = { done: "done", failed: "failed" } as const;
/** The text of the copy: its line break, the bullet of a finding and what joins its parts. */
export const MARKDOWN = { lineBreak: "\n", bullet: "- ", join: " · ", code: "`" } as const;
