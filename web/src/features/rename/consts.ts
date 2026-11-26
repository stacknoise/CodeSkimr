/** Id of the command that previews a rename (crates/app/src/ui_commands/consts.rs). */
export const RENAME_COMMAND = "rename.preview";

/** Id of the heading of the dialog, for `aria-labelledby`. */
export const RENAME_TITLE_ID = "rename-dialog-title";

/** The key that runs the check from the field of the name. */
export const RENAME_SUBMIT_KEY = "Enter";
/** The key that closes the dialog. */
export const RENAME_CANCEL_KEY = "Escape";

/** Most rows of each list that the dialog shows; the counts above them stay whole. */
export const RENAME_SHOWN_ROWS = 12;

/** The kinds of a place of the code. */
export const SITE_KIND = { declaration: "declaration", usage: "usage" } as const;

/** The kinds of a text line. */
export const TEXT_KIND = {
  documentation: "documentation",
  configuration: "configuration",
  string: "string",
  nameof: "nameof",
  comment: "comment",
  code: "code",
} as const;

/** How sure the resolver was about a usage. */
export const CONFIDENCE_PROBABLE = "probable";

/** What stands between the parts of a line of the copied list. */
export const LIST_SEPARATOR = "\t";
/** What stands between the lines of the copied list. */
export const LIST_LINE_END = "\n";
/** What stands between the path and the line number of a place. */
export const PLACE_SEPARATOR = ":";
/** The arrow between the old and the new name. */
export const ARROW = "→";
/** Spaces of the indentation of the copied JSON. */
export const JSON_INDENT = 2;

/** Code of a request that failed validation: here a new name that is no name or is the old one. */
export const REQUEST_INVALID = "API-101";

/** The badges of a place of the code. */
export const BADGE = {
  declaration: "declaration",
  sure: "sure",
  probable: "probable",
  unchecked: "unchecked",
} as const;
