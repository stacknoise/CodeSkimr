import { I18N_KEYS } from "../../shared/i18n/keys";

/** Ids of the commands of the explorer (crates/app/src/ui_commands/consts.rs). */
export const COMMAND = {
  outline: "explorer.outline",
  symbol: "explorer.symbol",
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
  reveal: ".",
} as const;
/** One row down and up. */
export const STEP_DOWN = 1;
export const STEP_UP = -1;
export const NOT_FOUND = -1;

/** What a row of the tree is. */
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
export const GROUP = { fields: "fields", methods: "methods" } as const;

/** The chips of the member table. */
export const CHIP = {
  all: "all",
  properties: "properties",
  methods: "methods",
  public: "public",
} as const;
export type ChipId = (typeof CHIP)[keyof typeof CHIP];

/** Metrics of the strip: stored kind, label and tip. */
export const METRICS = [
  { kind: "loc", label: I18N_KEYS.EXPLORER_METRIC_LOC, tip: I18N_KEYS.EXPLORER_METRIC_LOC_TIP },
  { kind: "cc", label: I18N_KEYS.EXPLORER_METRIC_CC, tip: I18N_KEYS.EXPLORER_METRIC_CC_TIP },
  { kind: "mi", label: I18N_KEYS.EXPLORER_METRIC_MI, tip: I18N_KEYS.EXPLORER_METRIC_MI_TIP },
  { kind: "ce", label: I18N_KEYS.EXPLORER_METRIC_CE, tip: I18N_KEYS.EXPLORER_METRIC_CE_TIP },
  { kind: "ca", label: I18N_KEYS.EXPLORER_METRIC_CA, tip: I18N_KEYS.EXPLORER_METRIC_CA_TIP },
  {
    kind: "instability",
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
