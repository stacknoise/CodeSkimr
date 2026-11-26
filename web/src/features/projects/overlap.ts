import { SCOPE_PATH_SEPARATOR, SOURCE_KIND } from "./consts";
import type { ProjectRow } from "./types";

const BACKSLASH = /\\/g;
const TRAILING_SEPARATORS = /\/+$/;

/** A path with `/`, without trailing separators and in lower case: enough to tell whether one
 * folder lies inside another (a hint, never a block). */
function plain(path: string): string {
  return path
    .trim()
    .replace(BACKSLASH, SCOPE_PATH_SEPARATOR)
    .replace(TRAILING_SEPARATORS, "")
    .toLowerCase();
}

/** What the typed folder is to a project: the project that holds it and the folder in it. */
export interface Overlap {
  project: ProjectRow;
  /** The folder relative to the project folder, as typed (with `/`). */
  folder: string;
}

/** The project whose folder holds the typed folder, the innermost one if several do; `null` for
 * a folder that is no part of a project, or is the folder of one (adding it again is known). */
export function overlapOf(source: string, rows: readonly ProjectRow[]): Overlap | null {
  const typed = plain(source);
  let best: Overlap | null = null;
  for (const project of rows) {
    const root = plain(project.location);
    const inside =
      project.kind === SOURCE_KIND.folder &&
      root !== "" &&
      typed.startsWith(`${root}${SCOPE_PATH_SEPARATOR}`);
    if (inside && (best === null || root.length > plain(best.project.location).length)) {
      best = {
        project,
        folder: source
          .trim()
          .replace(BACKSLASH, SCOPE_PATH_SEPARATOR)
          .replace(TRAILING_SEPARATORS, "")
          .slice(root.length + 1),
      };
    }
  }
  return best;
}
