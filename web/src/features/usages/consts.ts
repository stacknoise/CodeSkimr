import { I18N_KEYS } from "../../shared/i18n/keys";

/** Id of the command of the screen (crates/app/src/ui_commands/consts.rs). */
export const USAGES_COMMAND = "usages.query";
/** Command that reads the code line of each hit; it needs the right to read source. */
export const USAGES_CODE_COMMAND = "usages.code";
/** Separates the parts of a qualified name: `.` and `::`. */
export const NAME_SEPARATOR = /[.:]+/;

/** The directions of the toolbar, named as the command names them. */
export const WAY = { both: "both", incoming: "incoming", outgoing: "outgoing" } as const;
export type Way = (typeof WAY)[keyof typeof WAY];

/** The buttons of the direction group with their texts. */
export const WAYS = [
  { id: WAY.both, label: I18N_KEYS.USAGES_MODE_BOTH },
  { id: WAY.incoming, label: I18N_KEYS.USAGES_MODE_CALLERS },
  { id: WAY.outgoing, label: I18N_KEYS.USAGES_MODE_CALLEES },
] as const;

/** Depth that asks for as many steps as the configuration allows. */
export const DEPTH_ALL = 0;
/** Depth shown at first: only direct usages, never those that reach a symbol through another one
 * (product owner, 2026-11-20). The select offers deeper walks on request. */
export const DEFAULT_DEPTH = 1;
/** The depths of the select, ascending, "all" last. */
export const DEPTHS: readonly number[] = [DEFAULT_DEPTH, 2, 3, DEPTH_ALL];

/** Tests are shown at first (mock 6). */
export const DEFAULT_SHOW_TESTS = true;

/** How the command names a kind of use and a confidence. */
export const REF = { test: "test" } as const;
export const CONFIDENCE = { probable: "probable" } as const;

/** Texts of the kinds of use; a kind that is not listed shows its own name. */
export const REF_LABEL: Record<string, (typeof I18N_KEYS)[keyof typeof I18N_KEYS]> = {
  call: I18N_KEYS.USAGES_REF_CALL,
  read: I18N_KEYS.USAGES_REF_READ,
  write: I18N_KEYS.USAGES_REF_WRITE,
  type_use: I18N_KEYS.USAGES_REF_TYPE_USE,
  inherit: I18N_KEYS.USAGES_REF_INHERIT,
  implement: I18N_KEYS.USAGES_REF_IMPLEMENT,
  override: I18N_KEYS.USAGES_REF_OVERRIDE,
  import: I18N_KEYS.USAGES_REF_IMPORT,
  test: I18N_KEYS.USAGES_REF_TEST,
};

/** The first step away from the focus. */
export const FIRST_LEVEL = 1;

/** How the graph is drawn, in pixels (mock 6: four columns of 236 wide cards). */
export const GRAPH = {
  left: 24,
  top: 40,
  bottom: 56,
  columnWidth: 236,
  columnGap: 90,
  cardHeight: 50,
  focusHeight: 64,
  rowGap: 20,
  /** Cards drawn per column; the rest is counted in a note. */
  maxCards: 30,
  /** Divisor for centring. */
  half: 2,
} as const;

/** Colour classes of an edge, from most to least important. */
export const TONE = {
  incoming: "incoming",
  outgoing: "outgoing",
  test: "test",
  probable: "probable",
} as const;
export type Tone = (typeof TONE)[keyof typeof TONE];

/** The three kinds of column of the graph. */
export const COLUMN = { callers: "callers", focus: "focus", callees: "callees" } as const;
export type ColumnKind = (typeof COLUMN)[keyof typeof COLUMN];

/** Where a qualified name is cut into its parts, and how the last parts are joined. */
export const NAME_SEPARATORS = ["::", "."] as const;
export const NAME_JOIN = ".";
/** Parts of a qualified name that a card shows: the type and the member. */
export const NAME_PARTS = 2;

/** Keys of the screen. */
export const KEY = { next: "F8", open: "Enter" } as const;
/** Directions of the hit list. */
export const STEP_NEXT = 1;
export const STEP_PREVIOUS = -1;
/** `indexOf` found nothing. */
export const NOT_FOUND = -1;
/** The depth of a direct reference; deeper hits reach the symbol only through others. */
export const DIRECT_DEPTH = 1;
/** No hit is selected. */
export const NO_HIT = -1;

/** Shown for a missing line or layer. */
export const MISSING = "–";
/** Joins file and line of a hit: `src/Cart.cs:12`. */
export const LOCATION_JOIN = ":";

/** Roles of the groups of the toolbar. */
export const ROLE = { group: "group", region: "region" } as const;
/** Rating of the verification of a hit, named as the styles name it. */
export const STATE = {
  verified: "verified",
  unchecked: "unchecked",
  presumed: "presumed",
} as const;
export type State = (typeof STATE)[keyof typeof STATE];
/** Number of layer colours (`layer-1` to `layer-6` of the tokens). */
export const LAYER_TONES = 6;
/** Prefix of the class of a layer colour. */
export const LAYER_CLASS = "layer";
/** Value of `aria-current` for the selected hit. */
export const TRUE = "true";
/** State of level 2 while no indexer exists for the project (ADR 0033). */
export const LEVEL2_UNAVAILABLE = "unavailable";
/** Separates kind and layer under the name of the symbol. */
export const KIND_SEPARATOR = " · ";
/** The two ends of a hit: the symbol that contains the occurrence and the symbol it refers to. */
export const END = { from: "from", to: "to" } as const;
export type End = (typeof END)[keyof typeof END];
/** Keeps the selected hit in view without moving the list more than needed. */
export const SCROLL_NEAREST: ScrollIntoViewOptions = { block: "nearest" };
/** Commands and separators of the SVG path of an edge, and of the key of a pair of symbols. */
export const PATH = { move: "M", curve: "C", space: " " } as const;
export const PAIR_SEPARATOR = ">";
export const JOIN_NONE = "";

/** Zoom of the graph: the notch of a zoom button, the first scale, and the key and limits of the
 * height of the hit list (mock 6 shows 250 px). */
export const ZOOM_BUTTON_NOTCHES = 2;
export const HITS_PANE_KEY = "usages.hits";
export const HITS_PANE_BOUNDS = { initial: 250, min: 80, max: 700 } as const;
/** Symbols on the buttons that zoom, and the text of the one that fits the graph. */
export const ZOOM_SYMBOL = { in: "+", out: "−", fit: "⤢" } as const;
/** The smallest scale a new graph opens with; a larger graph is centred on the focus instead. */
export const START_MIN_SCALE = 0.6;

/** The width of the side panel, and the key under which it is remembered. */
export const PANEL_PANE_KEY = "usages.panel";
export const PANEL_PANE_BOUNDS = { initial: 300, min: 240, max: 720 } as const;

/** A character that can be part of an identifier: a word is only a whole word next to others. */
export const IDENTIFIER_CHAR = /[\p{L}\p{N}_]/u;

/** The columns of the hit list; every one can be widened with its handle except the place column, which takes what is left. */
export const HIT_COLUMNS = [
  { id: "kind", label: I18N_KEYS.USAGES_COL_KIND, bounds: { initial: 84, min: 50, max: 240 } },
  { id: "where", label: I18N_KEYS.USAGES_COL_WHERE, bounds: { initial: 220, min: 60, max: 600 } },
  { id: "code", label: I18N_KEYS.USAGES_COL_CODE, bounds: { initial: 520, min: 120, max: 1600 } },
  { id: "place", label: I18N_KEYS.USAGES_COL_PLACE, bounds: null },
  { id: "state", label: I18N_KEYS.USAGES_COL_STATE, bounds: { initial: 84, min: 50, max: 240 } },
] as const;
/** Storage key prefix of the widths of the columns. */
export const HIT_COLUMN_KEY = "usages.hits.col.";
/** Pixels a column grows or shrinks with one press of an arrow key. */
export const COLUMN_KEY_STEP_PX = 12;

/** Separates the words of a filter text. */
export const WORD_SEPARATOR = /\s+/;
/** The width of the column that takes what is left, the unit of the others and what joins the widths. */
export const LEFTOVER_COLUMN_WIDTH = "minmax(0, 1fr)";
export const PIXEL_UNIT = "px";
export const COLUMN_JOIN = " ";

/** The keys that move a column handle, and by how many steps (a key not listed moves nothing). */
export const COLUMN_KEY_DELTA: Record<string, number> = { ArrowRight: 1, ArrowLeft: -1 };
/** No movement. */
export const NO_DELTA = 0;
