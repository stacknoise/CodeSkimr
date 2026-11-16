/** Prefix of the keys under which the sizes of the panes are kept in `localStorage`. */
export const LAYOUT_STORAGE_PREFIX = "codeskimr.layout.";

/** Pixels that an arrow key moves a separator, and that a key with Shift moves it. */
export const KEY_STEP_PX = 16;
export const KEY_STEP_LARGE_PX = 64;

/** The way a separator lies: a `vertical` one changes a width, a `horizontal` one a height. */
export const ORIENTATION = { vertical: "vertical", horizontal: "horizontal" } as const;
export type Orientation = (typeof ORIENTATION)[keyof typeof ORIENTATION];

/** The keys a separator answers to. */
export const SEPARATOR_KEY = {
  increase: ["ArrowRight", "ArrowDown"],
  decrease: ["ArrowLeft", "ArrowUp"],
  smallest: "Home",
  largest: "End",
} as const;

/** The number base of a stored size. */
export const DECIMAL = 10;

/** Which side of the handle the pane lies on: `before` (left or above) grows when the handle moves
 * right or down, `after` (right or below) shrinks. */
export const SIDE = { before: 1, after: -1 } as const;
export type Side = (typeof SIDE)[keyof typeof SIDE];
