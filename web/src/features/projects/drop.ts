import { DRIVE_PATH, DROP, DROP_TYPES, FILE_SCHEME, LINE_BREAK, URI_COMMENT } from "./consts";

/** What a drop on the window carried. */
export type Dropped =
  /** A path or an address that can go into the source field. */
  | { kind: typeof DROP.text; text: string }
  /** Files or folders, which the browser shows without a path. */
  | { kind: typeof DROP.opaque }
  | { kind: typeof DROP.empty };

/** The path of a `file://` address; other text stays as it is. */
export function pathOf(text: string): string {
  if (!text.startsWith(FILE_SCHEME)) {
    return text;
  }
  const path = decodeURIComponent(text.slice(FILE_SCHEME.length));
  return DRIVE_PATH.test(path) ? path.slice(1) : path;
}

function firstLine(text: string): string {
  const lines = text.split(LINE_BREAK).map((line) => line.trim());
  return lines.find((line) => line !== "" && !line.startsWith(URI_COMMENT)) ?? "";
}

/** Reads a drop: a folder dragged from a file manager into the browser carries a `file://`
 * address in some systems and only an opaque file entry in others. */
export function readDrop(data: DataTransfer): Dropped {
  for (const type of DROP_TYPES) {
    const line = firstLine(data.getData(type));
    if (line !== "") {
      return { kind: DROP.text, text: pathOf(line) };
    }
  }
  return data.files.length > 0 || data.items.length > 0
    ? { kind: DROP.opaque }
    : { kind: DROP.empty };
}
