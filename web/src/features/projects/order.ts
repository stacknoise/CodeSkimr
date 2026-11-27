import { TEXT_COLLATION } from "../../shared/table/consts";
import type { ProjectRow } from "./types";

/** The projects in the order of the list: by shown name, without regard to case; projects with the
 * same name keep a stable order by id. `locale` is the language of the app. */
export function byName(rows: readonly ProjectRow[], locale?: string): ProjectRow[] {
  return [...rows].sort(
    (a, b) => a.name.localeCompare(b.name, locale, TEXT_COLLATION) || a.id - b.id,
  );
}
