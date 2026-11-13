import {
  ALT_SHORTCUT_KEY,
  DRIVE_PATTERN,
  FIRST_LINE,
  PATH_SEPARATOR,
  PLACEHOLDER_LINE,
  PLACEHOLDER_PATH,
  SCHEME_PATTERN,
  SCHEME_SEPARATOR,
  SHORTCUT_KEY,
  TEXT_ENTRY_TAGS,
  WINDOWS_SEPARATOR,
} from "./consts";

/** The setting `editor` of the server (ADR 0028). */
export interface EditorConfig {
  url_template: string;
  allowed_schemes: string[];
}

/** The scheme of `url`, in lower case, if it is written as a scheme. */
function schemeOf(url: string): string | null {
  const end = url.indexOf(SCHEME_SEPARATOR);
  const scheme = end > 0 ? url.slice(0, end).toLowerCase() : "";
  return SCHEME_PATTERN.test(scheme) ? scheme : null;
}

/** Percent-encodes a file path for the link: every part on its own, so that the slashes stay,
 * and a drive letter keeps its colon (`vscode://file/C:/src/a.cs`). */
export function encodePath(path: string): string {
  const parts = path.replace(WINDOWS_SEPARATOR, PATH_SEPARATOR).split(PATH_SEPARATOR);
  return parts
    .map((part) => (DRIVE_PATTERN.test(part) ? part : encodeURIComponent(part)))
    .join(PATH_SEPARATOR);
}

/** The path without its leading slash when the template already has a slash before `{path}`, so
 * that `vscode://file/{path}` gives `vscode://file/home/a.rs` and not `vscode://file//home/a.rs`. */
function dropDoubleSlash(template: string, path: string): string {
  const slashBefore = template.includes(`${PATH_SEPARATOR}${PLACEHOLDER_PATH}`);
  return slashBefore && path.startsWith(PATH_SEPARATOR) ? path.slice(PATH_SEPARATOR.length) : path;
}

/** The link that opens `path` at `line` in the editor, or `null` if the template is not usable:
 * a scheme that is not on the allow-list, no scheme, or no place for the path. The server checks
 * the same when it reads the setting; the UI does not trust the answer blindly, because the
 * link leaves the page. */
export function buildEditorUrl(config: EditorConfig, path: string, line?: number): string | null {
  const scheme = schemeOf(config.url_template);
  const allowed = config.allowed_schemes.map((name) => name.toLowerCase());
  if (scheme === null || !allowed.includes(scheme)) {
    return null;
  }
  if (!config.url_template.includes(PLACEHOLDER_PATH)) {
    return null;
  }
  const shown =
    line !== undefined && Number.isInteger(line) && line >= FIRST_LINE ? line : FIRST_LINE;
  return config.url_template
    .split(PLACEHOLDER_PATH)
    .join(encodePath(dropDoubleSlash(config.url_template, path)))
    .split(PLACEHOLDER_LINE)
    .join(String(shown));
}

/** Asks the browser to open `url`, which hands it to the program that handles its scheme. */
export function openLink(url: string, target: Pick<Location, "assign"> = window.location): void {
  target.assign(url);
}

/** Whether a key press asks to open the selection in the editor: `E` or `Alt+Enter`, but not
 * while the user types in a field. */
export function isEditorShortcut(event: KeyboardEvent): boolean {
  const typing =
    event.target instanceof HTMLElement && TEXT_ENTRY_TAGS.includes(event.target.tagName);
  if (typing || event.ctrlKey || event.metaKey) {
    return false;
  }
  const plainKey = event.key.toLowerCase() === SHORTCUT_KEY && !event.altKey && !event.shiftKey;
  return plainKey || (event.altKey && event.key === ALT_SHORTCUT_KEY);
}
