import { NAME_BOUNDARY, TARGET_SEPARATOR } from "./consts";
import type { Counters, SummaryRow, Target } from "./types";

/** The counters of the project, joined to the symbols the UI draws. */
export interface Index {
  /** By kind and qualified name. */
  exact: ReadonlyMap<string, Counters>;
  /** By qualified name alone, for a place that does not know the kind (the dependency map). */
  byName: ReadonlyMap<string, Counters>;
  /** The qualified names of the critical symbols. */
  critical: readonly string[];
}

/** The index of nothing: no symbol is annotated. */
export const EMPTY_INDEX: Index = { exact: new Map(), byName: new Map(), critical: [] };

/** The key of a symbol in the index. */
export function targetKey(kind: string, name: string): string {
  return [kind, name].join(TARGET_SEPARATOR);
}

function merge(known: Counters | undefined, row: SummaryRow): Counters {
  return {
    critical: (known?.critical ?? false) || row.critical,
    notes: (known?.notes ?? 0) + row.notes,
    tasksDone: (known?.tasksDone ?? 0) + row.tasks_done,
    tasksTotal: (known?.tasksTotal ?? 0) + row.tasks_total,
  };
}

/** Joins the rows of `annotations.summary` by kind and qualified name. Two rows for one key
 * (overloads with different parameters) are added up. A symbol that is gone from the snapshot
 * does not count: nothing in the UI shows it. */
export function buildIndex(rows: readonly SummaryRow[]): Index {
  const exact = new Map<string, Counters>();
  const byName = new Map<string, Counters>();
  const critical = new Set<string>();
  for (const row of rows.filter((one) => !one.without_target)) {
    const key = targetKey(row.target.kind, row.target.name);
    exact.set(key, merge(exact.get(key), row));
    byName.set(row.target.name, merge(byName.get(row.target.name), row));
    if (row.critical) {
      critical.add(row.target.name);
    }
  }
  return { exact, byName, critical: [...critical] };
}

/** The counters of a symbol, `undefined` if it has no annotation. */
export function countersOf(index: Index, target: Target | null): Counters | undefined {
  return target === null ? undefined : index.exact.get(targetKey(target.kind, target.name));
}

/** How many critical symbols stand below `name` (a namespace or a type), the symbol itself not
 * counted: their qualified names continue the name after a `.` or `::`. */
export function criticalBelow(index: Index, name: string): number {
  return index.critical.filter(
    (other) => other.startsWith(name) && NAME_BOUNDARY.test(other.slice(name.length)),
  ).length;
}

/** Whether a symbol has a note or a task, so that it has something to show next to its name. */
export function hasCounters(counters: Counters | undefined): boolean {
  return counters !== undefined && (counters.notes > 0 || counters.tasksTotal > 0);
}

/** The number of critical symbols a collapsed row hides: those below its `scope` (a namespace or a
 * type), or, for a group, those among the `names` of its members. */
export function criticalHidden(
  index: Index,
  scope: string | null,
  names: readonly string[],
): number {
  if (scope !== null) {
    return criticalBelow(index, scope);
  }
  return names.filter((name) => index.byName.get(name)?.critical === true).length;
}
