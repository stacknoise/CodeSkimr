/** One hit of `search.query` (crates/app/src/ui_commands/search.rs). */
export interface Hit {
  id: number;
  name: string;
  qualified_name: string;
  kind: string;
  path: string;
  parent: number | null;
}

/** The answer of `search.query`. */
export interface SearchResult {
  snapshot: number | null;
  hits: Hit[];
}
