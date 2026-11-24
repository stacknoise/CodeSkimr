import { I18N_KEYS } from "../../shared/i18n/keys";

/** Ids of the commands of the explorer (crates/app/src/ui_commands/consts.rs). */
export const COMMAND = {
  outline: "explorer.outline",
  symbol: "explorer.symbol",
  source: "explorer.source",
} as const;

/** The icons of the button to copy: ready, done and failed. */
export const COPY_ICON = { idle: "copy", copied: "check", failed: "cross" } as const;

/** The key of `COPY_ICON` while nothing was tried lately. */
export const COPY_IDLE = "idle";

/** The text of a status that has nothing to say. */
export const NO_TEXT = "";

/** The line break of copied code. */
export const LINE_BREAK = "\n";

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
/** The short name of a language in the tree; a language without one shows its id. */
export const LANGUAGE_SHORT: Record<string, string> = {
  c: "C",
  cpp: "C++",
  csharp: "C#",
  go: "Go",
  java: "Java",
  javascript: "JS",
  kotlin: "Kt",
  php: "PHP",
  python: "Py",
  ruby: "Rb",
  rust: "Rs",
  tsx: "TSX",
  typescript: "TS",
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
/** Kinds that the tree groups under "Type aliases". */
export const ALIAS_KINDS: readonly string[] = ["type_alias"];
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
} as const;
/** The letters that act on the selection: the action each one runs, by the name of its key in
 * `ExplorerKeys` (keys.ts). */
export const PLAIN_KEYS: Readonly<
  Record<string, "usages" | "graph" | "critical" | "note" | "task" | undefined>
> = { u: "usages", g: "graph", k: "critical", n: "note", t: "task" };
/** One row down and up. */
export const STEP_DOWN = 1;
export const STEP_UP = -1;
/** `Alt+Down` and `Alt+Up` step to the next and previous member of the source. */
export const MEMBER_STEP: Readonly<Record<string, number>> = {
  [KEY.down]: STEP_DOWN,
  [KEY.up]: STEP_UP,
};
export const NOT_FOUND = -1;

/** Separates the file and the lines in a location. */
export const LOCATION_SEPARATOR = ":";

/** Columns of the member table: name, annotations, signature, access, CC, uses, lines. */
export const COLUMN_COUNT = 7;
/** The columns of the member table by id (the ids of the sortable ones are the sort columns), the id of
 * the notes column and the key the widths are kept under. */
export const NOTES_COLUMN = "notes";
export const MEMBER_TABLE_KEY = "members";

/** Id of the complexity metric, which names the limits of its column. */
export const CC_METRIC = "cc";

/** What a row of the tree is. */
export const ROW = {
  layer: "layer",
  namespace: "namespace",
  type: "type",
  group: "group",
  member: "member",
} as const;
export type RowKind = (typeof ROW)[keyof typeof ROW];

/** Separates the folders of the path of a file. */
export const PATH_SEPARATOR = "/";

/** One part of a namespace name; the parts are separated by `.` or `::`. */
export const NAME_PART = /[^.:]+/g;

/** Keys of expanded rows are built from a prefix and the names, separated by this. */
export const KEY_SEPARATOR = "\u0001";
/** Indent of a tree row per level, in pixels. */
export const INDENT_PX = 14;
/** Indent of the first level, in pixels. */
export const BASE_INDENT_PX = 8;

/** Characters of a collapsed and an expanded row. */
export const CARET = { closed: "▶", open: "▼" } as const;

/** The member groups of the tree. */
export const GROUP = {
  fields: "fields",
  methods: "methods",
  nested: "nested",
  aliases: "aliases",
} as const;

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

/** The tiles of the coupling counts (ADR 0037); while level 2 has not confirmed the language of the
 * symbol they say "upper bound". */
export const BOUND_TILES: readonly string[] = ["ce", "ca", "instability"];

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
/** Keys under which the sizes of the columns are kept, and their limits in pixels (mock 2 shows
 * the first values). */
export const PANE_KEY = {
  tree: "explorer.tree",
  source: "explorer.source",
  panel: "explorer.panel",
} as const;
export const PANE_BOUNDS = {
  tree: { initial: 290, min: 200, max: 640 },
  source: { initial: 560, min: 280, max: 1100 },
  panel: { initial: 400, min: 280, max: 640 },
} as const;

/** The columns of the member table that sort it. */
export const SORT_COLUMN = {
  name: "name",
  signature: "signature",
  access: "access",
  complexity: "complexity",
  usages: "usages",
  lines: "lines",
} as const;
export type SortColumn = (typeof SORT_COLUMN)[keyof typeof SORT_COLUMN];
export const MEMBER_COLUMNS = Object.values(SORT_COLUMN);
export const MEMBER_COLUMNS_WITH_NOTES = [
  SORT_COLUMN.name,
  NOTES_COLUMN,
  SORT_COLUMN.signature,
  SORT_COLUMN.access,
  SORT_COLUMN.complexity,
  SORT_COLUMN.usages,
  SORT_COLUMN.lines,
];

/** The directions of a sort, as `aria-sort` names them. */
export const SORT_DIRECTION = { ascending: "ascending", descending: "descending" } as const;
export type SortDirection = (typeof SORT_DIRECTION)[keyof typeof SORT_DIRECTION];

/** The member table is sorted by name until a column head is pressed. */
export const DEFAULT_SORT = {
  column: SORT_COLUMN.name,
  direction: SORT_DIRECTION.ascending,
} as const;

/** The columns of numbers sort the largest first when they are chosen. */
export const NUMBER_COLUMNS: readonly SortColumn[] = [
  SORT_COLUMN.complexity,
  SORT_COLUMN.usages,
  SORT_COLUMN.lines,
];

/** The marks next to the name of the column that sorts the table. */
export const SORT_MARK = { ascending: "▲", descending: "▼" } as const;

/** Compares names without regard to case and accents. */
export const NAME_COLLATION: Intl.CollatorOptions = { sensitivity: "base", numeric: true };

/** The sign that turns the order of a comparison for the direction of a sort. */
export const SORT_SIGN = { ascending: 1, descending: -1 } as const;

/** The separator between a namespace and a name in the qualified name of a type
 * (crates/app/src/ui_commands/consts.rs). */
export const QUALIFIED_SEPARATOR = ".";

/** The language of a type of an answer that names none. */
export const NO_LANGUAGE = "";
