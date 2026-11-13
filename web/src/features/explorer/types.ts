/** A type in the tree (crates/app/src/ui_commands/explorer.rs). */
export interface TypeNode {
  id: number;
  name: string;
  kind: string;
  members: number;
}

/** A namespace with its types. */
export interface NamespaceNode {
  name: string;
  types: TypeNode[];
}

/** A layer with its namespaces; the name is empty for types that belong to no layer. */
export interface LayerNode {
  name: string;
  namespaces: NamespaceNode[];
}

/** The answer of `explorer.outline`. */
export interface Outline {
  snapshot: number | null;
  layers: LayerNode[];
}

/** A symbol in the breadcrumb. */
export interface Crumb {
  id: number;
  name: string;
  kind: string;
}

/** One measured value with its rating. */
export interface MetricRow {
  kind: string;
  value: number;
  rating: string | null;
}

/** One row of the member table. */
export interface MemberRow {
  id: number;
  name: string;
  kind: string;
  visibility: string | null;
  signature: string | null;
  complexity: number | null;
  complexity_rating: string | null;
  usages: number;
  first_line: number;
  last_line: number;
}

/** The answer of `explorer.symbol`. */
export interface SymbolInfo {
  snapshot: number;
  id: number;
  name: string;
  qualified_name: string;
  kind: string;
  visibility: string | null;
  is_static: boolean;
  path: string;
  first_line: number;
  last_line: number;
  signature: string | null;
  doc: string | null;
  breadcrumb: Crumb[];
  metrics: MetricRow[];
  members: MemberRow[];
}
