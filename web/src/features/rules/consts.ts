import { I18N_KEYS } from "../../shared/i18n/keys";

/** Id of the command of the screen (crates/app/src/ui_commands/consts.rs). */
export const COMMAND = {
  show: "rules.show",
  hold: "rules.hold",
  openFile: "project_file.open",
  set: "rules.set",
  preview: "rules.preview",
  layers: "layers.get",
} as const;

/** The kinds of dialog the rules screen opens to change the rules (ADR 0112). */
export const DIALOG = {
  forbid: "forbid",
  publicApi: "public_api",
  layers: "layers",
  remove: "remove",
} as const;

/** How long after the last key the preview of a rule is asked for, in milliseconds. */
export const PREVIEW_DELAY_MS = 350;
/** Separates the lines of a list of prefixes in its field. */
export const LINE_BREAK = "\n";
/** The id of the title of the dialog, for `aria-labelledby`. */
export const DIALOG_TITLE_ID = "rule-dialog-title";
/** The id of the list of namespaces the fields of the dialog offer. */
export const NAMESPACES_LIST_ID = "rule-namespaces";
/** The key that closes the dialog. */
export const CLOSE_KEY = "Escape";
/** The head of an entry of `[[rules.forbid]]` as the file gets it. */
export const TOML_FORBID_HEAD = "[[rules.forbid]]";
/** The head of an entry of `[[rules.public_api]]` as the file gets it. */
export const TOML_API_HEAD = "[[rules.public_api]]";
/** The mark of a layer that may use another one. */
export const TICK = "✓";
/** The mark of a layer in its own column: it never uses itself. */
export const SELF = "–";

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

/** The icons of the buttons of a rule card and their size. */
export const ICON = { pen: "pen", bin: "bin" } as const;
export const ICON_SIZE = 16;
/** How many lines the field for the inside of a module shows. */
export const INSIDE_ROWS = 3;
