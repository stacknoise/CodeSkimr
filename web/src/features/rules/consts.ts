/** Id of the command of the screen (crates/app/src/ui_commands/consts.rs). */
export const COMMAND = {
  show: "rules.show",
  hold: "rules.hold",
} as const;

/** The kinds of rules the command names. */
export const KIND = {
  layers: "layers",
  cycles: "cycles",
  forbid: "forbid",
  publicApi: "public_api",
} as const;

/** How many violations of a rule are shown before the rest is folded away. */
export const VISIBLE_VIOLATIONS = 3;
/** How many places of a violation are named on its row before the rest is cut. */
export const VISIBLE_PLACES = 2;

/** Separates path, line and column of a place. */
export const PLACE_SEPARATOR = ":";
/** Separates the places on a row. */
export const PLACES_JOIN = ", ";
/** Separates the prefixes of the inside of a module. */
export const INTERNAL_JOIN = ", ";
/** The arrow between the symbol that refers and the one it must not reach. */
export const ARROW = "→";
/** The marks of the state of a rule. */
export const MARK = { kept: "✓", broken: "×" } as const;
/** What follows the prefix of an inside: every name below it. */
export const INTERNAL_SUFFIX = ".*";
/** Joins the text of "more" and the button that shows everything. */
export const MORE_JOIN = " · ";
/** Ends the places of a violation that has more of them than the row names. */
export const PLACES_MORE = ", …";

/** Separates the layers in the line of the layer rule. */
export const LAYERS_JOIN = "; ";
/** Separates the layers one layer may use. */
export const USES_JOIN = ", ";
