import { EXPLORER_ROUTE } from "../app/consts";

/** Id of the element that holds the data of the export as JSON. */
export const DATA_ELEMENT_ID = "codeskimr-data";

/** Version of the data block that this page reads (`EXPORT_FORMAT` in
 * crates/app/src/ui_commands/consts.rs). */
export const DATA_FORMAT = 1;

/** Start of the address part that names the screen; the export is one file, so the screens are
 * told apart by the fragment. */
export const HASH_PREFIX = "#";

/** Screen shown while the address names none. */
export const START_PATH = EXPLORER_ROUTE.path;

/** Id of the command that lists the projects. The export knows exactly one. */
export const PROJECTS_COMMAND = "projects.list";

/** Status of an answer for a command or a symbol that the export does not hold. */
export const STATUS_NOT_FOUND = 404;

/** Lowest rank of a search hit: the name starts with the text. */
export const RANK_PREFIX = 0;
/** The name contains the text. */
export const RANK_NAME = 1;
/** Only the qualified name contains the text. */
export const RANK_QUALIFIED = 2;
/** The letters of the text appear in the name in this order, with others between them. */
export const RANK_FUZZY = 3;
/** Most hits of one search, as in the server (`SEARCH_LIMIT`). */
export const SEARCH_LIMIT = 50;

/** Seconds are counted by the server; dates are made from milliseconds. */
export const MS_PER_SECOND = 1000;

/** The lines of a declaration: the export never holds source text. */
export const NO_LINES: string[] = [];

/** Between the name of the project and the name of the program in the title of the page. */
export const TITLE_SEPARATOR = " – ";

/** Between the parts of the first line of the banner. */
export const BANNER_SEPARATOR = " · ";
/** Characters of a commit that the banner shows. */
export const SHORT_COMMIT = 8;
