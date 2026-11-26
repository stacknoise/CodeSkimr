/** One placed layer, in layout units (`BoxOut` in crates/app/src/ui_commands/graph.rs). */
export interface LayerBox {
  /** The key of the box in the links and the selection. */
  name: string;
  /** The text of the box: the short name; the path is in `context`. */
  label: string;
  /** The namespaces in front of a type, as a breadcrumb; empty where there is none. */
  context: string;
  x: number;
  y: number;
  width: number;
  height: number;
  rank: number;
  order: number;
  /** Name of the layer that owns the box; for a layer box its own name. */
  layer: string | null;
  /** Number of types in a namespace box; `null` on the layer level. */
  types: number | null;
  /** Id of the type of a box on the type level; `null` on the other levels. */
  symbol: number | null;
}

/** The stripe of one layer behind the rows of its boxes (`BandOut`). */
export interface LayerBand {
  name: string;
  rank: number;
  y: number;
  height: number;
}

/** What a box stands for. */
export type Level = "layers" | "namespaces" | "types" | "modules";

/** One dependency between two layers. */
export interface LayerLink {
  from: string;
  to: string;
  count: number;
  violates_layering: boolean;
  in_cycle: boolean;
}

/** The type in the middle of the map on the type level, and what the picture leaves out. */
export interface FocusInfo {
  /** Id of the type in the middle; `null` for a namespace. */
  symbol: number | null;
  name: string;
  users_more: number;
  used_more: number;
}

/** The answer of `graph.layers`. */
export interface GraphData {
  snapshot: number | null;
  width: number;
  height: number;
  boxes: LayerBox[];
  bands: LayerBand[];
  links: LayerLink[];
  /** The type in the middle; `null` on the other levels. */
  focus: FocusInfo | null;
}

/** Zoom and shift of the picture: a point of the layout appears at `x + scale * point`. */
export interface View {
  scale: number;
  x: number;
  y: number;
}

/** A width and a height. */
export interface Size {
  width: number;
  height: number;
}

/** One row of the table of modules (`ModuleRow` in crates/app/src/ui_commands/graph_modules.rs). */
export interface ModuleRow {
  name: string;
  layer: string | null;
  incoming: number;
  outgoing: number;
  /** Stability in thousandths, `null` for a module without any dependency. */
  stability: number | null;
  types: number;
  in_cycle: boolean;
  violations: number;
}

/** A dependency between two modules. */
export interface ModuleEdge {
  from: string;
  to: string;
  count: number;
}

/** A ring of modules and the edges that end it. */
export interface ModuleCycle {
  modules: string[];
  edges: ModuleEdge[];
  resolution: ModuleEdge[];
}

/** A dependency between two modules that breaks the layering. */
export interface ModuleViolation {
  edge: ModuleEdge;
  from_layer: string;
  to_layer: string;
}

/** A module that stands out. */
export interface ModuleRisk {
  module: string;
  incoming: number;
  stability: number;
  depends_on: string;
  depends_on_stability: number;
}

/** What the tab "Modules" shows beside the picture. */
export interface ModulesInfo {
  modules: ModuleRow[];
  modules_more: number;
  cycles: ModuleCycle[];
  violations: ModuleViolation[];
  risks: ModuleRisk[];
}

/** The answer of `graph.modules`: the picture as of `graph.layers`, and the numbers. */
export type ModulesData = GraphData & ModulesInfo;
