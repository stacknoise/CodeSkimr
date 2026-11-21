/** A layer as `layers.get` and `layers.set` carry it (crates/app/src/ui_commands/layers.rs). */
export interface LayerEntry {
  name: string;
  namespaces: string[];
}

/** A namespace of the newest scan and the layer that owns it now. */
export interface NamespaceEntry {
  name: string;
  types: number;
  layer: string | null;
}

/** The answer of `layers.get`. */
export interface LayersData {
  editable: boolean;
  scanned: boolean;
  layers: LayerEntry[];
  namespaces: NamespaceEntry[];
}

/** One row of the namespace tree: a prefix of the names and everything below it. */
export interface PrefixNode {
  prefix: string;
  label: string;
  depth: number;
  /** Types in the namespaces at or below the prefix. */
  types: number;
  children: PrefixNode[];
}
