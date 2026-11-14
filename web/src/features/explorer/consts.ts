import { I18N_KEYS } from "../../shared/i18n/keys";

/** Ids of the commands of the explorer (crates/app/src/ui_commands/consts.rs). */
export const COMMAND = {
  outline: "explorer.outline",
  symbol: "explorer.symbol",
  source: "explorer.source",
} as const;

/** Letter of the badge of a symbol kind, as in the mock. */
export const KIND_LETTER: Record<string, string> = {
  namespace: "N",
  module: "D",
  class: "C",
  interface: "I",
  trait: "T",
  record: "R",
  struct: "S",
  enum: "E",
  union: "U",
  function: "F",
  method: "M",
  constructor: "C",
  property: "P",
  field: "F",
  constant: "K",
  event: "E",
  enum_member: "V",
  parameter: "A",
  type_alias: "T",
};
/** Letter of a kind that is not in the table. */
export const KIND_LETTER_OTHER = "?";

/** The colour class of a badge, by kind; kinds without one are grey. */
export const KIND_TONE: Record<string, string> = {
  class: "class",
  constructor: "struct",
  interface: "interface",
  trait: "interface",
  record: "record",
  struct: "struct",
  enum: "enum",
  union: "enum",
  method: "record",
  function: "record",
  property: "class",
  constant: "interface",
  event: "enum",
  enum_member: "struct",
};
export const KIND_TONE_OTHER = "other";

/** Kinds that the tree groups under "Properties & fields". */
export const FIELD_KINDS: readonly string[] = [
  "property",
  "field",
  "constant",
  "event",
  "enum_member",
];
/** Kinds of types: a type inside a type is listed under "Nested types". */
export const TYPE_KINDS: readonly string[] = [
  "class",
  "interface",
  "trait",
  "record",
  "struct",
  "enum",
  "union",
];
/** Kinds that the tree groups under "Methods". */
export const METHOD_KINDS: readonly string[] = ["method", "constructor", "function"];
/** Kinds that the "Properties" chip of the member table shows. */
export const PROPERTY_KINDS: readonly string[] = ["property"];
/** Visibility that the chip "public only" keeps. */
export const PUBLIC = "public";

/** Keys of the explorer tree. */
export const KEY = {
  down: "ArrowDown",
  up: "ArrowUp",
  right: "ArrowRight",
  left: "ArrowLeft",
  home: "Home",
  end: "End",
  enter: "Enter",
  space: " ",
  reveal: ".",
  usages: "u",
  graph: "g",
} as const;
/** One row down and up. */
export const STEP_DOWN = 1;
export const STEP_UP = -1;
export const NOT_FOUND = -1;

/** What a row of the tree is. */
/** Depth of the rows below a namespace row. */
export const TYPE_DEPTH = 2;

/** Id of the complexity metric, which names the limits of its column. */
export const CC_METRIC = "cc";

export const ROW = {
  layer: "layer",
  namespace: "namespace",
  type: "type",
  group: "group",
  member: "member",
} as const;
export type RowKind = (typeof ROW)[keyof typeof ROW];

/** Keys of expanded rows are built from a prefix and the names, separated by this. */
export const KEY_SEPARATOR = "\u0001";
/** Indent of a tree row per level, in pixels. */
export const INDENT_PX = 14;
/** Indent of the first level, in pixels. */
export const BASE_INDENT_PX = 8;

/** Characters of a collapsed and an expanded row. */
export const CARET = { closed: "▶", open: "▼" } as const;

/** The member groups of the tree. */
export const GROUP = { fields: "fields", methods: "methods", nested: "nested" } as const;

/** The chips of the member table. */
export const CHIP = {
  all: "all",
  properties: "properties",
  methods: "methods",
  public: "public",
} as const;
export type ChipId = (typeof CHIP)[keyof typeof CHIP];

/** Kinds that the tile "Fields etc." counts: the field group without the properties. */
export const FIELD_KINDS_OTHER: readonly string[] = ["field", "constant", "event", "enum_member"];

/** Where the value of a tile comes from: a stored metric, or a count of the member kinds. */
export const TILE_SOURCE = { metric: "metric", members: "members", kinds: "kinds" } as const;

/** The tiles of the metric strip in the order of the mock: source, label, tip and, for counts,
 * the kinds that are counted. `limits` names the thresholds the tip mentions. */
export const TILES = [
  {
    id: "members",
    source: TILE_SOURCE.members,
    label: I18N_KEYS.EXPLORER_METRIC_MEMBERS,
    tip: I18N_KEYS.EXPLORER_METRIC_MEMBERS_TIP,
  },
  {
    id: "loc",
    source: TILE_SOURCE.metric,
    label: I18N_KEYS.EXPLORER_METRIC_LOC,
    tip: I18N_KEYS.EXPLORER_METRIC_LOC_TIP,
  },
  {
    id: "cc",
    source: TILE_SOURCE.metric,
    label: I18N_KEYS.EXPLORER_METRIC_CC,
    tip: I18N_KEYS.EXPLORER_METRIC_CC_TIP,
  },
  {
    id: "mi",
    source: TILE_SOURCE.metric,
    label: I18N_KEYS.EXPLORER_METRIC_MI,
    tip: I18N_KEYS.EXPLORER_METRIC_MI_TIP,
  },
  {
    id: "properties",
    source: TILE_SOURCE.kinds,
    kinds: PROPERTY_KINDS,
    label: I18N_KEYS.EXPLORER_METRIC_PROPERTIES,
    tip: I18N_KEYS.EXPLORER_METRIC_PROPERTIES_TIP,
  },
  {
    id: "fields",
    source: TILE_SOURCE.kinds,
    kinds: FIELD_KINDS_OTHER,
    label: I18N_KEYS.EXPLORER_METRIC_FIELDS,
    tip: I18N_KEYS.EXPLORER_METRIC_FIELDS_TIP,
  },
  {
    id: "methods",
    source: TILE_SOURCE.kinds,
    kinds: METHOD_KINDS,
    label: I18N_KEYS.EXPLORER_METRIC_METHODS,
    tip: I18N_KEYS.EXPLORER_METRIC_METHODS_TIP,
  },
  {
    id: "ce",
    source: TILE_SOURCE.metric,
    label: I18N_KEYS.EXPLORER_METRIC_CE,
    tip: I18N_KEYS.EXPLORER_METRIC_CE_TIP,
  },
  {
    id: "ca",
    source: TILE_SOURCE.metric,
    label: I18N_KEYS.EXPLORER_METRIC_CA,
    tip: I18N_KEYS.EXPLORER_METRIC_CA_TIP,
  },
  {
    id: "instability",
    source: TILE_SOURCE.metric,
    label: I18N_KEYS.EXPLORER_METRIC_INSTABILITY,
    tip: I18N_KEYS.EXPLORER_METRIC_INSTABILITY_TIP,
  },
] as const;

/** Decimals of a metric value with a fraction. */
export const METRIC_DECIMALS = 2;
/** Shown for a missing value. */
export const MISSING = "–";
/** Shown between the first and the last line of a member. */
export const LINE_RANGE_SEPARATOR = "–";

/** Value of `aria-current` and `aria-selected` for the selected row. */
export const TRUE = "true";
/** Role of the tree and of its rows. */
export const ROLE = { tree: "tree", item: "treeitem", group: "group" } as const;
/** How a rating is named on the wire. */
export const RATING = { good: "good", medium: "medium", poor: "poor" } as const;

/** Icons of the two buttons above the tree. */
export const ICON_COLLAPSE = "collapse";
export const ICON_LOCATE = "locate";

/** Separates the parts of the kind pill: `class · public`. */
export const PILL_SEPARATOR = " · ";
/** Separates the symbols of the breadcrumb. */
export const CRUMB_SEPARATOR = "›";

/** Height of a tree row in pixels; the windowing of the tree counts with it. */
export const ROW_HEIGHT_PX = 26;
/** Rows drawn above and below the visible ones, so scrolling shows no gaps. */
export const OVERSCAN_ROWS = 8;
/** Rows assumed for the height of the tree before it has been measured. */
export const FALLBACK_VIEWPORT_ROWS = 40;
/** Prefix of the ids of the rows, for `aria-activedescendant`. */
export const ROW_ID_PREFIX = "explorer-row-";
/** Number of colour dots for the layers (`layer-1` to `layer-6` of the tokens). */
export const LAYER_TONES = 6;

/** Joins the folder of a project and the path of a file in it. */
export const PATH_JOINER = "/";

/** Keyboard hints on the three buttons of the header. */
export const HOTKEY = { usages: "U", graph: "G", editor: "E" } as const;
