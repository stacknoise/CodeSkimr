import { I18N_KEYS } from "../../shared/i18n/keys";
import type { MessageKey } from "../../shared/i18n/i18n";

/** Ids of the commands of the alerts (crates/app/src/ui_commands/alerts.rs). */
export const COMMAND = {
  list: "alerts.list",
  dismiss: "alerts.dismiss",
  dismissAll: "alerts.dismiss_all",
} as const;

/** The type of the event that tells the number of open alerts changed. */
export const ALERTS_EVENT = "alerts_changed";

/** The states of an alert. */
export const STATE = { open: "open", dismissed: "dismissed" } as const;
export type AlertState = (typeof STATE)[keyof typeof STATE];

/** The kinds of an alert. */
export const KIND = { changed: "changed", removed: "removed", orphaned: "orphaned" } as const;
export type AlertKind = (typeof KIND)[keyof typeof KIND];

/** What changed about a symbol. */
export type ChangeKind = "content" | "signature" | "cc" | "loc";
/** The names of the changes. */
export const CHANGE_TEXT: Record<ChangeKind, MessageKey> = {
  content: I18N_KEYS.ALERTS_CHANGE_CONTENT,
  signature: I18N_KEYS.ALERTS_CHANGE_SIGNATURE,
  cc: I18N_KEYS.ALERTS_CHANGE_CC,
  loc: I18N_KEYS.ALERTS_CHANGE_LOC,
};
/** Separates the names of the changes in a line. */
export const CHANGE_SEPARATOR = ", ";

/** What raised an alert. */
export const TRIGGER_TEXT: Record<string, MessageKey> = {
  scan: I18N_KEYS.ALERTS_TRIGGER_SCAN,
  watcher: I18N_KEYS.ALERTS_TRIGGER_WATCHER,
  fetch: I18N_KEYS.ALERTS_TRIGGER_FETCH,
};

/** The tabs of the drawer, in the order of the mock. */
export const TAB = { open: "open", changed: "changed", removed: "removed", done: "done" } as const;
export type TabId = (typeof TAB)[keyof typeof TAB];
export const TABS: readonly TabId[] = [TAB.open, TAB.changed, TAB.removed, TAB.done];
export const TAB_NAME: Record<TabId, MessageKey> = {
  open: I18N_KEYS.ALERTS_TAB_OPEN,
  changed: I18N_KEYS.ALERTS_TAB_CHANGED,
  removed: I18N_KEYS.ALERTS_TAB_REMOVED,
  done: I18N_KEYS.ALERTS_TAB_DONE,
};
/** Id prefix of the tab buttons and id of the tab panel. */
export const TAB_ID_PREFIX = "alerts-tab-";
export const PANEL_ID = "alerts-panel";
export const DRAWER_ID = "alerts-drawer";
export const BELL_ID = "alerts-bell";

/** The kinds the tab "Removed" holds: orphaned targets are listed with them. */
export const REMOVED_KINDS: readonly AlertKind[] = [KIND.removed, KIND.orphaned];

/** The kinds "dismiss all" of a tab sends; an empty list means every kind. */
export const TAB_KINDS: Record<TabId, readonly AlertKind[]> = {
  open: [],
  changed: [KIND.changed],
  removed: REMOVED_KINDS,
  done: [],
};

/** The text of the label of a kind. */
export const KIND_TEXT: Record<AlertKind, MessageKey> = {
  changed: I18N_KEYS.ALERTS_KIND_CHANGED,
  removed: I18N_KEYS.ALERTS_KIND_REMOVED,
  orphaned: I18N_KEYS.ALERTS_KIND_ORPHANED,
};

/** Rows per call and most pages read for one list. */
export const PAGE_SIZE = 500;
export const MAX_PAGES = 100;
/** Rows of the call that only wants the number of open alerts. */
export const COUNT_ONLY_LIMIT = 1;

/** Keys of the drawer and of the tab list. */
export const KEY = { close: "Escape" } as const;
export const TAB_STOP = { reachable: 0, skipped: -1 } as const;
/** The mark of the button that dismisses one alert, and the badge of a count above the limit. */
export const CLOSE_MARK = "✕";
/** Most open alerts the bell writes out; more show as `99+`. */
export const BELL_MAX = 99;
export const BELL_OVER = "99+";

/** Seconds in a minute, an hour and a day, and the limits that choose the unit of a time. */
export const SECONDS = { minute: 60, hour: 3600, day: 86_400 } as const;
export const MS_PER_SECOND = 1000;
/** Written out as a date after this many days. */
export const RELATIVE_DAYS = 7;
/** Units of a relative time. */
export const UNIT = { second: "second", minute: "minute", hour: "hour", day: "day" } as const;
/** Characters of a commit that are shown, and what separates the parts of the line under an alert. */
export const COMMIT_SHORT = 7;
export const META_SEPARATOR = " · ";
/** What the lists of the drawer are doing. */
export const LOADING = {
  idle: "idle",
  loading: "loading",
  failed: "failed",
  ready: "ready",
} as const;
export type Loading = (typeof LOADING)[keyof typeof LOADING];
/** How a relative time is written: "yesterday" and "now" instead of "1 day ago" and "0 seconds ago". */
export const RELATIVE_STYLE = { numeric: "auto" } as const;
