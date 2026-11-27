import { I18N_KEYS } from "../../shared/i18n/keys";
import type { MessageKey } from "../../shared/i18n/i18n";
import { TONE } from "../../shared/ui/consts";
import type { PillTone } from "../../shared/ui/Pill";

/** Ids of the commands of the screen (crates/app/src/ui_commands/consts.rs). */
export const COMMAND = {
  list: "scans.list",
  impact: "scans.impact",
  markdown: "scans.impact_markdown",
  metrics: "scans.metrics",
  metricsMarkdown: "scans.metrics_markdown",
} as const;

/** The two views of the screen (ADR 0097). */
export const TAB = { impact: "impact", compare: "compare", metrics: "metrics" } as const;

/** The tabs in their order, with their texts. */
export const TABS = [
  { id: TAB.impact, label: I18N_KEYS.CHANGE_VIEW_TAB_IMPACT },
  { id: TAB.compare, label: I18N_KEYS.CHANGE_VIEW_TAB_COMPARE },
  { id: TAB.metrics, label: I18N_KEYS.CHANGE_VIEW_TAB_METRICS },
] as const;

/** Start of the ids of the tab buttons, and the id of the panel they control. */
export const TAB_ID_PREFIX = "change-tab-";
export const PANEL_ID = "change-panel";
/** The ARIA roles of the tab list. */
export const ROLE = { tablist: "tablist", tab: "tab", tabpanel: "tabpanel", grid: "grid" } as const;
/** The `tabindex` of the selected tab and of the others. */
export const TAB_STOP = { on: 0, off: -1 } as const;

/** What happened to a symbol, as the answer names it. */
export const CAUSE = { removed: "removed", changed: "changed", added: "added" } as const;
/** The cause of a symbol that has the same key in another file. */
export const CAUSE_MOVED = "moved";
/** Why a symbol counts as changed. */
export const REASON = {
  signature: "signature",
  visibility: "visibility",
  content: "content",
} as const;
/** The status and the confidence of a usage. */
export const STATUS = { adapted: "adapted", notTouched: "not_touched" } as const;
export const CONFIDENCE = { certain: "certain", probable: "probable" } as const;
/** The visibility that counts as public in the table. */
export const PUBLIC_VISIBILITY = "public";

/** The text of a visibility, by the word the answer uses. */
export const VISIBILITY_TEXT: Record<string, MessageKey> = {
  public: I18N_KEYS.CHANGE_VIEW_VISIBILITY_PUBLIC,
  protected: I18N_KEYS.CHANGE_VIEW_VISIBILITY_PROTECTED,
  internal: I18N_KEYS.CHANGE_VIEW_VISIBILITY_INTERNAL,
  private: I18N_KEYS.CHANGE_VIEW_VISIBILITY_PRIVATE,
};

/** A pill of the table: its text and its tone as in the mocks. */
export interface PillSpec {
  label: MessageKey;
  tone: PillTone;
}
/** The pill that names a reason of a change, by the word the answer uses. */
export const REASON_PILL: Record<string, PillSpec> = {
  signature: { label: I18N_KEYS.CHANGE_VIEW_KIND_SIGNATURE, tone: TONE.warn },
  visibility: { label: I18N_KEYS.CHANGE_VIEW_KIND_VISIBILITY, tone: TONE.warn },
  content: { label: I18N_KEYS.CHANGE_VIEW_KIND_CONTENT, tone: TONE.neutral },
};
/** The pill that names what happened to a symbol that did not change in place. */
export const CAUSE_PILL: Record<string, PillSpec> = {
  removed: { label: I18N_KEYS.CHANGE_VIEW_KIND_REMOVED, tone: TONE.bad },
  added: { label: I18N_KEYS.CHANGE_VIEW_KIND_ADDED, tone: TONE.good },
  moved: { label: I18N_KEYS.CHANGE_VIEW_KIND_MOVED, tone: TONE.neutral },
};

/** The mark in front of a row of the table, as in the mock: plus, tilde, minus. */
export const MARK: Record<string, string> = { added: "+", changed: "~", removed: "−", moved: "→" };
/** The sign in front of a line of a changed signature. */
export const SIGN = { before: "−", after: "+" } as const;
/** Joins the parts of a short line, such as the numbers under a figure. */
export const PART_SEPARATOR = " · ";
/** Joins the places of a violation. */
export const EDGE_SEPARATOR = "; ";
/** Joins the old and the new place of a symbol that moved. */
export const MOVE_ARROW = " → ";
/** Separates a file from the line in it. */
export const LINE_SEPARATOR = ":";
/** Joins the two chips of the compared scans and the commit of one. */
export const COMMIT_JOIN = " @ ";
/** How many characters of a commit the chips show. */
export const SHORT_COMMIT = 7;
/** The arrow between the two scans of a comparison. */
export const SCAN_ARROW = "→";

/** The ids of the columns of the table of the impact (the sort and the widths go by them). */
export const SYMBOL_COLUMN = {
  symbol: "symbol",
  change: "change",
  visibility: "visibility",
  usages: "usages",
  state: "state",
} as const;

/** The columns of the table of the symbols by id, with their head texts, and the key the widths are
 * kept under. */
export const SYMBOL_TABLE_KEY = "change-symbols";
export const SYMBOL_COLUMNS = [
  { id: SYMBOL_COLUMN.symbol, label: I18N_KEYS.CHANGE_VIEW_COL_SYMBOL, number: false },
  { id: SYMBOL_COLUMN.change, label: I18N_KEYS.CHANGE_VIEW_COL_CHANGE, number: false },
  { id: SYMBOL_COLUMN.visibility, label: I18N_KEYS.CHANGE_VIEW_COL_VISIBILITY, number: false },
  { id: SYMBOL_COLUMN.usages, label: I18N_KEYS.CHANGE_VIEW_COL_USAGES, number: true },
  { id: SYMBOL_COLUMN.state, label: I18N_KEYS.CHANGE_VIEW_COL_STATE, number: true },
] as const;

/** Keys that move the selection of the table, and how far each moves; Home and End go to the ends. */
export const ROW_KEY = {
  next: "ArrowDown",
  previous: "ArrowUp",
  first: "Home",
  last: "End",
} as const;
/** Start of the ids of the rows, so that the table can name the selected one. */
export const ROW_ID_PREFIX = "change-row-";
/** Scrolls a selected row into view without moving the page more than needed. */
export const SCROLL_NEAREST: ScrollIntoViewOptions = { block: "nearest" };

/** The ids of the two selectors of the scans. */
export const BASE_FIELD_ID = "change-base";
export const TARGET_FIELD_ID = "change-target";

/** The word for what a change did to a critical symbol, by the word the answer uses. */
export const OUTCOME_TEXT: Record<string, MessageKey> = {
  removed: I18N_KEYS.CHANGE_VIEW_OUTCOME_REMOVED,
  moved: I18N_KEYS.CHANGE_VIEW_OUTCOME_MOVED,
  changed: I18N_KEYS.CHANGE_VIEW_OUTCOME_CHANGED,
};

/** The fewest scans a comparison needs. */
export const MIN_SCANS = 2;
/** The index `findIndex` answers when nothing matches. */
export const NOT_FOUND = -1;

/** The visibility the filter "only public" keeps. */
export const VISIBILITY_PUBLIC = "public";
/** The value of a choice of the filter that lets everything pass. */
export const FILTER_ALL = "all";
/** The kinds of change the filter offers, in the order of the table, with their texts. */
export const CAUSE_CHOICES = [
  { id: CAUSE.removed, label: I18N_KEYS.CHANGE_VIEW_FILTER_CAUSE_REMOVED },
  { id: CAUSE.changed, label: I18N_KEYS.CHANGE_VIEW_FILTER_CAUSE_CHANGED },
  { id: CAUSE_MOVED, label: I18N_KEYS.CHANGE_VIEW_FILTER_CAUSE_MOVED },
  { id: CAUSE.added, label: I18N_KEYS.CHANGE_VIEW_FILTER_CAUSE_ADDED },
] as const;
/** Joins the letter of a symbol kind and its name in the choice of the filter. */
export const KIND_JOIN = " · ";
/** The id of the search field of the filter. */
export const FILTER_FIELD_ID = "change-filter-query";
/** Joins the name of a symbol and its signature in a tooltip. */
export const TITLE_JOIN = " ";

/** The columns of the list of the comparison by id, with their head texts, and the key the widths
 * are kept under. */
export const COMPARE_TABLE_KEY = "change-compare";
export const COMPARE_COLUMN = { change: "change", symbol: "symbol", file: "file" } as const;
export const COMPARE_COLUMNS = [
  { id: COMPARE_COLUMN.change, label: I18N_KEYS.CHANGE_VIEW_COL_CHANGE, number: false },
  { id: COMPARE_COLUMN.symbol, label: I18N_KEYS.CHANGE_VIEW_COL_SYMBOL, number: false },
  { id: COMPARE_COLUMN.file, label: I18N_KEYS.CHANGE_VIEW_COL_FILE, number: false },
] as const;

/** The levels of the view "Metrics" by the word the answer uses, in the order of the choice, with
 * their texts and the letter of the tag in front of an object. */
export const METRICS_LEVEL = { member: "member", type: "type", module: "module" } as const;
export const METRICS_LEVELS = [
  { id: METRICS_LEVEL.member, label: I18N_KEYS.CHANGE_METRICS_LEVEL_MEMBER, tag: "M" },
  { id: METRICS_LEVEL.type, label: I18N_KEYS.CHANGE_METRICS_LEVEL_TYPE, tag: "T" },
  { id: METRICS_LEVEL.module, label: I18N_KEYS.CHANGE_METRICS_LEVEL_MODULE, tag: "N" },
] as const;
/** The id of the choice of the level. */
export const METRICS_LEVEL_FIELD_ID = "change-metrics-level";

/** The metrics by the word the answer uses (`MetricKind::label`), in the order of the filter. */
export const METRIC = {
  cc: "cc",
  mi: "mi",
  loc: "loc",
  ce: "ce",
  instability: "instability",
} as const;
/** The value of the filter that shows every metric. */
export const METRIC_ALL = "all";
/** The choices of the filter by metric, in the order of the mock, with their texts. */
export const METRIC_CHOICES = [
  { id: METRIC_ALL, label: I18N_KEYS.CHANGE_METRICS_FILTER_ALL },
  { id: METRIC.cc, label: I18N_KEYS.CHANGE_METRICS_METRIC_CC },
  { id: METRIC.mi, label: I18N_KEYS.CHANGE_METRICS_METRIC_MI },
  { id: METRIC.loc, label: I18N_KEYS.CHANGE_METRICS_METRIC_LOC },
  { id: METRIC.ce, label: I18N_KEYS.CHANGE_METRICS_METRIC_CE },
  { id: METRIC.instability, label: I18N_KEYS.CHANGE_METRICS_METRIC_INSTABILITY },
] as const;
/** The name of a metric in a row, by the word the answer uses. */
export const METRIC_TEXT: Record<string, MessageKey> = {
  cc: I18N_KEYS.CHANGE_METRICS_METRIC_CC,
  mi: I18N_KEYS.CHANGE_METRICS_METRIC_MI,
  loc: I18N_KEYS.CHANGE_METRICS_METRIC_LOC,
  ce: I18N_KEYS.CHANGE_METRICS_METRIC_CE,
  instability: I18N_KEYS.CHANGE_METRICS_METRIC_INSTABILITY,
};
/** The tone of the pill of a move, by what the move did as the answer names it. */
export const DIRECTION_TONE: Record<string, PillTone> = {
  worse: TONE.bad,
  better: TONE.good,
  grew: TONE.warn,
};
/** How a move crossed the bound of the rating, as the answer names it, and its text. */
export const CROSSING_TEXT: Record<string, MessageKey> = {
  into_poor: I18N_KEYS.CHANGE_METRICS_CROSSING_INTO,
  out_of_poor: I18N_KEYS.CHANGE_METRICS_CROSSING_OUT,
};
/** The sign of a change, shown in front of its size. */
export const DELTA_SIGN = { up: "+", down: "−" } as const;
/** Values are shown with at most this many decimals. */
export const METRIC_DECIMALS = 2;
/** The id of the box of the list of what got worse and of what got better. */
export const METRICS_LIST = { worse: "worse", better: "better" } as const;
/** The prefix of the key under which the widths of the columns of a list are kept. */
export const METRICS_TABLE_KEY = "change-metrics-";
/** The columns of the two lists of the view "Metrics", by id. */
export const METRICS_COLUMN = {
  object: "object",
  file: "file",
  metric: "metric",
  values: "values",
  change: "change",
} as const;
export const METRICS_COLUMNS = [
  { id: METRICS_COLUMN.object, label: I18N_KEYS.CHANGE_METRICS_COL_OBJECT, number: false },
  { id: METRICS_COLUMN.file, label: I18N_KEYS.CHANGE_VIEW_COL_FILE, number: false },
  { id: METRICS_COLUMN.metric, label: I18N_KEYS.CHANGE_METRICS_FILTER_LABEL, number: false },
  { id: METRICS_COLUMN.values, label: I18N_KEYS.CHANGE_METRICS_COL_VALUES, number: true },
  { id: METRICS_COLUMN.change, label: I18N_KEYS.CHANGE_VIEW_COL_CHANGE, number: true },
] as const;
/** The metrics that a level does not have (ADR 0114): members have no coupling and no stability. */
export const METRICS_NOT_AT_MEMBER: readonly string[] = [METRIC.ce, METRIC.instability];
/** Joins the parts of the key that tells one object from another. */
export const OBJECT_KEY_JOIN = "\u0000";
/** What a figure shows when its number is not known. */
export const UNKNOWN_FIGURE = "–";
/** The tag of an object whose level the screen does not know. */
export const NO_TAG = "";
