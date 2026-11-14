/** One placed layer, in layout units (`BoxOut` in crates/app/src/ui_commands/graph.rs). */
export interface LayerBox {
  name: string;
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
}

/** The stripe of one layer behind the rows of its boxes (`BandOut`). */
export interface LayerBand {
  name: string;
  rank: number;
  y: number;
  height: number;
}

/** What a box stands for. */
export type Level = "layers" | "namespaces";

/** One dependency between two layers. */
export interface LayerLink {
  from: string;
  to: string;
  count: number;
  violates_layering: boolean;
  in_cycle: boolean;
}

/** The answer of `graph.layers`. */
export interface GraphData {
  snapshot: number | null;
  width: number;
  height: number;
  boxes: LayerBox[];
  bands: LayerBand[];
  links: LayerLink[];
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
