import { createSignal } from "solid-js";
import { FILTER_ALL, VISIBILITY_PUBLIC } from "./consts";

/** What the filter needs to know about a row of either list. */
export interface RowFacts {
  /** What happened: `removed`, `changed`, `moved` or `added`. */
  cause: string;
  /** The kind of the symbol, as the index names it. */
  kind: string;
  /** The qualified name the search looks at. */
  name: string;
  visibility: string | null;
  /** How many usages are not touched; 0 for a row without a checklist. */
  open: number;
}

/** What the user chose in the filter row. */
export interface ChangeFilter {
  query: string;
  cause: string;
  kind: string;
  onlyPublic: boolean;
  onlyOpen: boolean;
}

/** The filter that lets every row pass. */
export const NO_FILTER: ChangeFilter = {
  query: "",
  cause: FILTER_ALL,
  kind: FILTER_ALL,
  onlyPublic: false,
  onlyOpen: false,
};

const [filter, setFilter] = createSignal<ChangeFilter>(NO_FILTER);

/** The filter of the lists; both views share it. */
export const changeFilter = filter;
/** Changes some fields of the filter. */
export function patchFilter(change: Partial<ChangeFilter>): void {
  setFilter({ ...filter(), ...change });
}
/** Lets every row pass again. */
export function resetFilter(): void {
  setFilter(NO_FILTER);
}

/** Whether the filter holds back rows. `withOpen` says whether the view has the open places. */
export function isFiltering(chosen: ChangeFilter, withOpen: boolean): boolean {
  return (
    chosen.query.trim() !== NO_FILTER.query ||
    chosen.cause !== FILTER_ALL ||
    chosen.kind !== FILTER_ALL ||
    chosen.onlyPublic ||
    (withOpen && chosen.onlyOpen)
  );
}

/** Whether a row passes the filter. The search looks for the typed text anywhere in the qualified
 * name, ignoring case; the open places only count in the view that has them (`withOpen`). */
export function passes(facts: RowFacts, chosen: ChangeFilter, withOpen: boolean): boolean {
  const query = chosen.query.trim().toLowerCase();
  return (
    (query === "" || facts.name.toLowerCase().includes(query)) &&
    (chosen.cause === FILTER_ALL || facts.cause === chosen.cause) &&
    (chosen.kind === FILTER_ALL || facts.kind === chosen.kind) &&
    (!chosen.onlyPublic || facts.visibility === VISIBILITY_PUBLIC) &&
    (!withOpen || !chosen.onlyOpen || facts.open > 0)
  );
}

/** What `scans.impact` takes of the filter: only the fields that hold rows back, so the server
 * applies them before it cuts the lists (#826). */
export function filterInput(chosen: ChangeFilter): Record<string, string | boolean> {
  const name = chosen.query.trim();
  return {
    ...(name === "" ? {} : { name }),
    ...(chosen.cause === FILTER_ALL ? {} : { cause: chosen.cause }),
    ...(chosen.kind === FILTER_ALL ? {} : { kind: chosen.kind }),
    ...(chosen.onlyPublic ? { public_only: true } : {}),
    ...(chosen.onlyOpen ? { only_open: true } : {}),
  };
}

/** The kinds the rows have, sorted, for the choice of the filter. */
export function kindsOf(rows: readonly RowFacts[]): string[] {
  return [...new Set(rows.map((row) => row.kind))].sort();
}
