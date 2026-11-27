import { compareValues, type SortState } from "../../shared/table/sort";
import { CYCLE_JOIN, MODULE_COLUMN, NAME_ORDER } from "./consts";
import type {
  LayerLink,
  ModuleCycle,
  ModuleEdge,
  ModuleRisk,
  ModuleRow,
  ModuleViolation,
} from "./types";

/** Compares two texts the way the language sorts them: umlauts with their letter, no regard to
 * case, digits by value. */
export function compareText(a: string, b: string, language: string): number {
  return a.localeCompare(b, language, NAME_ORDER);
}

/** The items by one text each, then by the next one where two are equal. */
function sortedBy<T>(items: readonly T[], language: string, ...keys: ((item: T) => string)[]): T[] {
  return [...items].sort((a, b) => {
    for (const key of keys) {
      const order = compareText(key(a), key(b), language);
      if (order !== 0) {
        return order;
      }
    }
    return 0;
  });
}

/** Links by the name at the other end. */
export function sortLinks(
  links: readonly LayerLink[],
  pick: (link: LayerLink) => string,
  language: string,
): LayerLink[] {
  return sortedBy(links, language, pick);
}

/** Links by their two names, the one that leaves first. */
export function sortLinkPairs(links: readonly LayerLink[], language: string): LayerLink[] {
  return sortedBy(
    links,
    language,
    (link) => link.from,
    (link) => link.to,
  );
}

/** Module edges by their two names, the one that leaves first. */
export function sortEdges(edges: readonly ModuleEdge[], language: string): ModuleEdge[] {
  return sortedBy(
    edges,
    language,
    (edge) => edge.from,
    (edge) => edge.to,
  );
}

/** The value of a row in a column, `null` where the row has none. */
function valueOf(row: ModuleRow, column: string): string | number | null {
  switch (column) {
    case MODULE_COLUMN.incoming:
      return row.incoming;
    case MODULE_COLUMN.outgoing:
      return row.outgoing;
    case MODULE_COLUMN.stability:
      return row.stability;
    case MODULE_COLUMN.types:
      return row.types;
    default:
      return row.name;
  }
}

/** The rows of the table of modules by the sort of the table, by name while it has none; rows that
 * are equal in the column go by name. */
export function sortRows(
  rows: readonly ModuleRow[],
  language: string,
  sort: SortState | null,
): ModuleRow[] {
  const byName = (a: ModuleRow, b: ModuleRow) => compareText(a.name, b.name, language);
  if (sort === null) {
    return [...rows].sort(byName);
  }
  return [...rows].sort(
    (a, b) =>
      compareValues(valueOf(a, sort.column), valueOf(b, sort.column), sort.direction, language) ||
      byName(a, b),
  );
}

/** Violations against the layering by their two modules. */
export function sortViolations(
  found: readonly ModuleViolation[],
  language: string,
): ModuleViolation[] {
  return sortedBy(
    found,
    language,
    (item) => item.edge.from,
    (item) => item.edge.to,
  );
}

/** Modules that stand out by name, then by the module they depend on. */
export function sortRisks(risks: readonly ModuleRisk[], language: string): ModuleRisk[] {
  return sortedBy(
    risks,
    language,
    (risk) => risk.module,
    (risk) => risk.depends_on,
  );
}

/** Cycles by the names of their modules; the order inside one cycle is the way round the ring and
 * stays, but its edges are sorted. */
export function sortCycles(cycles: readonly ModuleCycle[], language: string): ModuleCycle[] {
  return sortedBy(cycles, language, (cycle) => cycle.modules.join(CYCLE_JOIN)).map((cycle) => ({
    ...cycle,
    edges: sortEdges(cycle.edges, language),
    resolution: sortEdges(cycle.resolution, language),
  }));
}
