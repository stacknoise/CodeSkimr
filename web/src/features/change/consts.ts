import { I18N_KEYS } from "../../shared/i18n/keys";
import type { MessageKey } from "../../shared/i18n/i18n";
import { TONE } from "../../shared/ui/consts";
import type { PillTone } from "../../shared/ui/Pill";

/** Ids of the commands of the screen (crates/app/src/ui_commands/consts.rs). */
export const COMMAND = {
  list: "scans.list",
  impact: "scans.impact",
} as const;

/** The two views of the screen (ADR 0097). */
export const TAB = { impact: "impact", compare: "compare" } as const;

/** The tabs in their order, with their texts. */
export const TABS = [
  { id: TAB.impact, label: I18N_KEYS.CHANGE_VIEW_TAB_IMPACT },
  { id: TAB.compare, label: I18N_KEYS.CHANGE_VIEW_TAB_COMPARE },
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

/** The columns of the table of the symbols by id, with their head texts, and the key the widths are
 * kept under. */
export const SYMBOL_TABLE_KEY = "change-symbols";
export const SYMBOL_COLUMNS = [
  { id: "symbol", label: I18N_KEYS.CHANGE_VIEW_COL_SYMBOL, number: false },
  { id: "change", label: I18N_KEYS.CHANGE_VIEW_COL_CHANGE, number: false },
  { id: "visibility", label: I18N_KEYS.CHANGE_VIEW_COL_VISIBILITY, number: false },
  { id: "usages", label: I18N_KEYS.CHANGE_VIEW_COL_USAGES, number: true },
  { id: "state", label: I18N_KEYS.CHANGE_VIEW_COL_STATE, number: true },
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
