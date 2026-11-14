import { I18N_KEYS } from "../../shared/i18n/keys";

/** Id of the command of the screen (crates/app/src/ui_commands/consts.rs). */
export const USAGES_COMMAND = "usages.query";

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
/** Depth shown at first (mock 6). */
export const DEFAULT_DEPTH = 2;
/** The depths of the select, in the order of the mock. */
export const DEPTHS: readonly number[] = [DEFAULT_DEPTH, 1, 3, DEPTH_ALL];

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
  columnGap: 30,
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
/** No hit is selected. */
export const NO_HIT = -1;

/** Shown for a missing line or layer. */
export const MISSING = "–";
/** Joins file and line of a hit: `src/Cart.cs:12`. */
export const LOCATION_JOIN = ":";

/** Roles of the groups of the toolbar. */
export const ROLE = { group: "group", region: "region", alert: "alert" } as const;
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
