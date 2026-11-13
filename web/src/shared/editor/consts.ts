/** Placeholders of the editor link template (ADR 0028). */
export const PLACEHOLDER_PATH = "{path}";
export const PLACEHOLDER_LINE = "{line}";
/** Separates the scheme of a URL from the rest. */
export const SCHEME_SEPARATOR = ":";
/** A URL scheme as RFC 3986 writes it. */
export const SCHEME_PATTERN = /^[a-z][a-z0-9+.-]*$/;
/** A drive letter such as `C:` at the start of a Windows path. */
export const DRIVE_PATTERN = /^[A-Za-z]:$/;
/** Separators of the parts of a path. */
export const PATH_SEPARATOR = "/";
export const WINDOWS_SEPARATOR = /\\/g;
/** Line the editor shows when none is known. */
export const FIRST_LINE = 1;
/** Keys of the shortcut "Open in editor" (blueprint 15.4). */
export const SHORTCUT_KEY = "e";
export const ALT_SHORTCUT_KEY = "Enter";
/** Elements in which typing an `e` is not a shortcut. */
export const TEXT_ENTRY_TAGS = ["INPUT", "TEXTAREA", "SELECT"];
