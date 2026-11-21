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

/** Milliseconds the browser gets to start the download before its address is released. */
export const EXPORT_REVOKE_DELAY_MS = 10_000;

/** Longest name of a file without the extension; file systems allow 255 characters in all. */
export const EXPORT_NAME_MAX = 100;

/** Names that Windows reserves for devices, with or without an extension. */
export const EXPORT_RESERVED_NAME = /^(con|prn|aux|nul|com[0-9]|lpt[0-9])(\..*)?$/i;
/** What is put in front of a reserved name. */
export const EXPORT_RESERVED_PREFIX = "project-";
