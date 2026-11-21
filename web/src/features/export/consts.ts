/** Id of the command that writes the project as one HTML page (crates/app/src/ui_commands/consts.rs). */
export const EXPORT_COMMAND = "export.html";

/** The media type of the downloaded page. */
export const EXPORT_MEDIA_TYPE = "text/html;charset=utf-8";

/** Extension of the downloaded file. */
export const EXPORT_EXTENSION = ".html";

/** Name of the file if the project name leaves nothing usable after cleaning. */
export const EXPORT_FALLBACK_NAME = "codeskimr-export";

/** What a file name may keep of the project name; everything else becomes `EXPORT_NAME_FILL`. */
export const EXPORT_NAME_UNSAFE = /[^\p{L}\p{N}._-]+/gu;
export const EXPORT_NAME_FILL = "-";
/** Dashes and dots at the ends of a cleaned name are dropped. */
export const EXPORT_NAME_EDGES = /^[-.]+|[-.]+$/g;

/** The element that carries the download. */
export const EXPORT_LINK_TAG = "a";
