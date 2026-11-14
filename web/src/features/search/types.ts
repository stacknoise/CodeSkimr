/** One hit of `search.query` (crates/app/src/ui_commands/search.rs). */
export interface Hit {
  id: number;
  name: string;
  qualified_name: string;
  kind: string;
  path: string;
  parent: number | null;
  /** The project the symbol belongs to; a search over all projects mixes them. */
  project: number;
  project_name: string;
}

/** One file of `search.query`: a file that declares at least one type. */
export interface FileHit {
  path: string;
  loc: number;
  /** The first type of the file, where the explorer opens it. */
  first_type: number;
  project: number;
  project_name: string;
}

/** The answer of `search.query`. */
export interface SearchResult {
  snapshot: number | null;
  /** Symbols of the snapshot, which the search runs over. */
  entries: number | null;
  hits: Hit[];
  files: FileHit[];
}
