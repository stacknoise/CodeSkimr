/** Id of the command that colours short pieces of code. */
export const HIGHLIGHT_COMMAND = "code.highlight";

/** Most pieces one call asks for (the server colours at most this many). */
export const HIGHLIGHT_BATCH = 200;

/** Joins the extension and the text into the key of the cache. */
export const KEY_SEPARATOR = "\u0000";

/** The end of a line in a piece of code. */
export const LINE_END = "\n";

/** Splits a path at its dots to find the extension. */
export const EXTENSION_SEPARATOR = ".";

/** The text of a piece that has no colours. */
export const NO_TEXT = "";

/** The place of the first line and of the start of a line. */
export const FIRST = 0;

/** How many answers the cache keeps; the oldest go first. */
export const CACHE_LIMIT = 5000;
