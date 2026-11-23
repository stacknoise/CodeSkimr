import { I18N_KEYS } from "../../shared/i18n/keys";

/** Ids of the commands of the statistics (crates/app/src/ui_commands/consts.rs). */
export const COMMAND = { overview: "stats.overview", ranking: "stats.ranking" } as const;

/** The views of the screen: the overview and one tab per ranking (ADR 0081). */
export const TAB = {
  overview: "overview",
  mostUsed: "most_used",
  unused: "unused",
  complexity: "complexity",
  size: "size",
  coupling: "coupling",
} as const;

/** The tabs in their order, with their texts. */
export const TABS = [
  { id: TAB.overview, label: I18N_KEYS.STATS_TAB_OVERVIEW },
  { id: TAB.mostUsed, label: I18N_KEYS.STATS_TAB_MOST_USED },
  { id: TAB.unused, label: I18N_KEYS.STATS_TAB_UNUSED },
  { id: TAB.complexity, label: I18N_KEYS.STATS_TAB_COMPLEXITY },
  { id: TAB.size, label: I18N_KEYS.STATS_TAB_SIZE },
  { id: TAB.coupling, label: I18N_KEYS.STATS_TAB_COUPLING },
] as const;

/** The kinds of symbols a ranking holds, as the command names them. */
export const KINDS = [
  { id: "all", label: I18N_KEYS.STATS_KINDS_ALL },
  { id: "types", label: I18N_KEYS.STATS_KINDS_TYPES },
  { id: "functions", label: I18N_KEYS.STATS_KINDS_FUNCTIONS },
] as const;
export const KINDS_ALL = "all";
export const KINDS_TYPES = "types";

/** Scrolls a selected row into view without moving the page more than needed. */
export const SCROLL_NEAREST: ScrollIntoViewOptions = { block: "nearest" };

/** Which count the most used symbols are sorted by. */
export const ORDERS = [
  { id: "certain", label: I18N_KEYS.STATS_ORDER_CERTAIN },
  { id: "all", label: I18N_KEYS.STATS_ORDER_ALL },
] as const;
export const ORDER_CERTAIN = "certain";

/** The assessments of a symbol without usages, with their texts and the tone of their pill. */
export const ASSESSMENTS = [
  {
    id: "likely",
    label: I18N_KEYS.STATS_ASSESSMENT_LIKELY,
    short: I18N_KEYS.STATS_ASSESSMENT_LIKELY_SHORT,
    tone: "good",
  },
  {
    id: "public",
    label: I18N_KEYS.STATS_ASSESSMENT_PUBLIC,
    short: I18N_KEYS.STATS_ASSESSMENT_PUBLIC_SHORT,
    tone: "warn",
  },
  {
    id: "approximate",
    label: I18N_KEYS.STATS_ASSESSMENT_APPROXIMATE,
    short: I18N_KEYS.STATS_ASSESSMENT_APPROXIMATE_SHORT,
    tone: "bad",
  },
] as const;

/** Texts of the visibilities. */
export const VISIBILITY_TEXT: Record<string, keyof typeof VISIBILITY_KEYS> = {
  public: "public",
  protected: "protected",
  internal: "internal",
  private: "private",
};
export const VISIBILITY_KEYS = {
  public: I18N_KEYS.STATS_VISIBILITY_PUBLIC,
  protected: I18N_KEYS.STATS_VISIBILITY_PROTECTED,
  internal: I18N_KEYS.STATS_VISIBILITY_INTERNAL,
  private: I18N_KEYS.STATS_VISIBILITY_PRIVATE,
} as const;

/** Rows a ranking asks for. */
export const ROW_LIMIT = 200;

/** Rating of a complexity above the limit (crates/app/src/ui_commands/consts.rs). */
export const RATING_POOR = "poor";

/** Keys of the screen: open the selected row in the explorer, show its usages, move. */
export const KEY = { open: "Enter", usages: "u", up: "ArrowUp", down: "ArrowDown" } as const;
/** The text of the keys on the buttons. */
export const KEY_TEXT = { open: "Enter", usages: "U" } as const;

/** ARIA roles and values the screen sets. */
export const ROLE = {
  tablist: "tablist",
  tab: "tab",
  tabpanel: "tabpanel",
  group: "group",
  img: "img",
  status: "status",
} as const;
/** Id of the panel the tabs control. */
export const PANEL_ID = "stats-panel";

/** Separates the parts of a line of details. */
/** The arrow of a row that opens the explorer. */
export const OPEN_ARROW = "↗";
/** Separator of the parts of a path; a row shows the last two. */
export const PATH_SEPARATOR = "/";
export const PATH_TAIL = 2;

/** The bar of the usages of a row, in the units of its drawing. */
export const USAGE_BAR = { width: 150, height: 8, radius: 4, viewBox: "0 0 150 8" } as const;
/** Id of the hatch pattern of the probable part of a bar, and its drawing. */
export const HATCH = {
  id: "stats-hatch",
  fill: "url(#stats-hatch)",
  size: 5,
  units: "userSpaceOnUse",
  turn: "rotate(45)",
  line: 2,
} as const;

/** The histogram of the complexity, in the units of its drawing. */
export const HISTOGRAM = {
  width: 520,
  height: 200,
  viewBox: "0 0 520 200",
  top: 18,
  bottom: 22,
  gap: 4,
  minBar: 3,
  radius: 2,
  labelGap: 4,
  axisGap: 14,
} as const;

/** Lower limits of the complexity classes of the histogram; the class above the limit of the
 * project starts at its own value so that a bar never mixes functions above and below it. */
export const COMPLEXITY_CLASS_STARTS = [1, 2, 5, 10, 20, 50, 100] as const;
/** The dash of a class range ("5–9") and the plus of the open last class ("100+"). */
export const RANGE_TEXT = { dash: "–", open: "+" } as const;

/** The picture of the hotspots, in the units of its drawing. */
export const HOTSPOTS = {
  width: 520,
  height: 220,
  viewBox: "0 0 520 220",
  left: 44,
  right: 12,
  top: 28,
  bottom: 34,
  axisTitle: 10,
  dot: 4.5,
  dotSelected: 7,
  ticksX: 5,
  tickGap: 14,
  labelGap: 6,
  titleGap: 30,
  minMaxX: 10,
  minMaxY: 10,
  whisker: 2,
  selectedStroke: 2,
} as const;
/** Base of the scale of the usages, and its steps on the axis. */
export const LOG_BASE = 10;
export const TEXT_ANCHOR = { end: "end", middle: "middle" } as const;
/** Dash pattern of the line of the limit. */
export const LIMIT_DASH = "4 3";
/** Prefix of the ids of the tab buttons. */
export const TAB_ID_PREFIX = "stats-tab-";
/** How far the arrow keys move the selection. */
export const STEP = { up: -1, down: 1 } as const;
/** Value of the language field that stands for all languages. */
export const LANGUAGE_ALL = "";

/** Divides a width into its two halves. */
export const HALVES = 2;

/** The tab stop of the tab that is shown and of the others (roving tabindex). */
export const TAB_STOP = { on: 0, off: -1 } as const;

/** The tones of the counters above the unused symbols; each names a class of the screen. */
export const COUNTER_TONE = { good: "good", warn: "warn", bad: "bad", plain: "plain" } as const;
