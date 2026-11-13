import { I18N_KEYS } from "../../shared/i18n/keys";
import type { MessageKey } from "../../shared/i18n/i18n";

/** The stages of the pipeline in the order they run, as the event stream names them
 * (`StageKind::label` in crates/application/src/stage.rs). */
export const STAGES = ["fetch", "discover", "analyze", "resolve", "measure", "persist"] as const;

/** The stage whose units are files: its progress is the progress of the whole scan. */
export const FILE_STAGE = "analyze";

/** The kinds of events on the stream (`WireEvent` in crates/server/src/events.rs). */
export const EVENT = {
  progressed: "scan_progressed",
  created: "snapshot_created",
  stopped: "scan_stopped",
  changed: "files_changed",
} as const;

/** Why a scan stopped without a snapshot. */
export const STOP = { cancelled: "cancelled", failed: "failed" } as const;

/** How a scan ends. */
export const STATE = {
  running: "running",
  done: "done",
  cancelled: "cancelled",
  failed: "failed",
} as const;

/** Names and details of the stages. */
export const STAGE_TEXT: Record<(typeof STAGES)[number], { name: MessageKey; detail: MessageKey }> =
  {
    fetch: { name: I18N_KEYS.SCAN_STAGE_FETCH, detail: I18N_KEYS.SCAN_STAGE_FETCH_DETAIL },
    discover: {
      name: I18N_KEYS.SCAN_STAGE_DISCOVER,
      detail: I18N_KEYS.SCAN_STAGE_DISCOVER_DETAIL,
    },
    analyze: { name: I18N_KEYS.SCAN_STAGE_ANALYZE, detail: I18N_KEYS.SCAN_STAGE_ANALYZE_DETAIL },
    resolve: { name: I18N_KEYS.SCAN_STAGE_RESOLVE, detail: I18N_KEYS.SCAN_STAGE_RESOLVE_DETAIL },
    measure: { name: I18N_KEYS.SCAN_STAGE_MEASURE, detail: I18N_KEYS.SCAN_STAGE_MEASURE_DETAIL },
    persist: { name: I18N_KEYS.SCAN_STAGE_PERSIST, detail: I18N_KEYS.SCAN_STAGE_PERSIST_DETAIL },
  };

/** The time budgets shown on the screen, in the order of the mock: the key in the output of
 * `status.info` and the name of the budget. */
export const BUDGET_ROWS = [
  { key: "cold_start", name: I18N_KEYS.SCAN_BUDGET_COLD_START },
  { key: "search_keystroke", name: I18N_KEYS.SCAN_BUDGET_SEARCH_KEYSTROKE },
  { key: "explorer_switch", name: I18N_KEYS.SCAN_BUDGET_EXPLORER_SWITCH },
  { key: "usages_query", name: I18N_KEYS.SCAN_BUDGET_USAGES_QUERY },
  { key: "graph_layout", name: I18N_KEYS.SCAN_BUDGET_GRAPH_LAYOUT },
  { key: "full_scan_100k_loc", name: I18N_KEYS.SCAN_BUDGET_FULL_SCAN },
  { key: "incremental_rescan", name: I18N_KEYS.SCAN_BUDGET_INCREMENTAL },
] as const;

/** The route of the event stream. */
export const EVENTS_PATH = "/api/v1/events";
/** Schemes of the stream for a page loaded over `http:` and over `https:`. */
export const SOCKET_SCHEME: Record<string, string> = { "http:": "ws:", "https:": "wss:" };
/** Scheme used when the page has none of the two. */
export const SOCKET_SCHEME_DEFAULT = "ws:";
/** Separates the scheme from the host. */
export const SCHEME_SEPARATOR = "//";
/** Wait before the stream is opened again after it was lost, in milliseconds. */
export const RECONNECT_MS = 2000;
/** Runs that are remembered, newest last. */
export const MAX_RUNS = 20;

/** Percent of a whole. */
export const PERCENT = 100;
/** Milliseconds in a second. */
export const MS_PER_SECOND = 1000;
/** Fewest files a rate needs before the remaining time is estimated. */
export const MIN_FILES_FOR_ESTIMATE = 20;

/** How many recent scans are listed. */
export const RECENT_LIMIT = 6;

/** Unit of the width of the progress bar. */
export const PERCENT_SIGN = "%";
/** Range of the progress bar for assistive technology. */
export const PROGRESS_MIN = 0;
export const PROGRESS_MAX = PERCENT;

/** How a stage of a run stands. */
export const STAGE_STATE = {
  waiting: "waiting",
  running: "running",
  done: "done",
  stopped: "stopped",
} as const;

/** The mark in front of a stage: a check once done, a dot while it runs, a bang when stopped. */
export const STAGE_MARKS = {
  waiting: "",
  running: "•",
  done: "✓",
  stopped: "!",
} as const;

/** The state of a run that counts as good, for the colour of a measured budget. */
export const GOOD = "good";
/** Attribute that carries the state of a row for the styles. */
export const STATE_ATTRIBUTE = "data-state";

/** Index of a run that is not in the list. */
export const NOT_FOUND = -1;

/** The budget that is defined for a project of a given size, and that size in lines of code. */
export const FULL_SCAN_BUDGET = "full_scan_100k_loc";
export const FULL_SCAN_LOC = 100_000;
