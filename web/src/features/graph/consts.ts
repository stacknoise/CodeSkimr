/** Id of the command that places the layers (crates/app/src/ui_commands/consts.rs). */
export const GRAPH_COMMAND = "graph.layers";

/** The levels of the map, as the command names them. */
export const LEVEL = { layers: "layers", namespaces: "namespaces" } as const;

/** The look of an edge. */
export const EDGE = { out: "out", in: "in", violation: "violation", weak: "weak" } as const;

/** Colour variable, width, dash pattern and opacity of each kind of edge. */
export const EDGE_STYLE = {
  out: { color: "var(--outgoing)", width: 2, dash: "", opacity: 1, marker: "graph-arrow-out" },
  in: { color: "var(--accent)", width: 2, dash: "", opacity: 1, marker: "graph-arrow-in" },
  violation: {
    color: "var(--bad)",
    width: 1.6,
    dash: "5 4",
    opacity: 1,
    marker: "graph-arrow-violation",
  },
  weak: { color: "var(--faint)", width: 1, dash: "", opacity: 0.35, marker: "graph-arrow-weak" },
} as const;

/** Ids of the arrow heads in the drawing. */
export const MARKER_IDS = [
  "graph-arrow-out",
  "graph-arrow-in",
  "graph-arrow-violation",
  "graph-arrow-weak",
] as const;
/** Fill of the arrow head of each marker, as colour variables. */
export const MARKER_FILL: Record<(typeof MARKER_IDS)[number], string> = {
  "graph-arrow-out": "var(--outgoing)",
  "graph-arrow-in": "var(--accent)",
  "graph-arrow-violation": "var(--bad)",
  "graph-arrow-weak": "var(--faint)",
};
/** Wraps an id into the reference an SVG attribute needs. */
export const URL_OPEN = "url(#";
export const URL_CLOSE = ")";

/** Height of the loop a dependency between two boxes of one row draws above them. */
export const LOOP_HEIGHT = 34;
/** Space around the picture when it is fitted to the view. */
export const FIT_PADDING = 32;
/** Smallest and largest zoom. */
export const MIN_SCALE = 0.2;
export const MAX_SCALE = 3;
/** Factor of one wheel notch. */
export const ZOOM_STEP = 1.1;
/** Divides the vertical wheel distance into notches. */
export const WHEEL_NOTCH = 100;

/** Keys of the screen. */
export const KEY = { fit: "f", pan: " ", open: "Enter" } as const;

/** Wheel and pointer values. */
export const BUTTON_MIDDLE = 1;

/** How many colours the layers cycle through (`--layer-1` to `--layer-N`). */
export const LAYER_COLORS = 4;
/** Prefix of the colour variables of the layers. */
export const LAYER_VAR = "var(--layer-";

/** The arrow head of an edge (SVG marker attributes). */
export const ARROW = {
  viewBox: "0 0 10 10",
  refX: 9,
  refY: 5,
  size: 7,
  orient: "auto-start-reverse",
  shape: "M0 0L10 5L0 10z",
} as const;
/** Unit of the sizes of the drawing. */
export const PX = "px";
/** An edge is drawn without fill. */
export const NO_FILL = "none";
/** Border colour of the left edge of a box, as the CSS property name. */
export const BOX_COLOR_PROPERTY = "border-left-color";
/** Parts of the CSS `transform` of the picture. */
export const TRANSLATE = "translate(";
export const SCALE = ") scale(";
export const SEPARATOR = ", ";
export const END = ")";
/** Moves of an edge path. */
export const PATH_MOVE = "M";
export const PATH_CURVE = " C";
export const SPACE = " ";

/** Half, for centring. */
export const HALF = 0.5;
/** The padding applies on both sides. */
export const PADDING_SIDES = 2;
