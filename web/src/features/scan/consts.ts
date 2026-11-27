import { I18N_KEYS } from "../../shared/i18n/keys";
import type { MessageKey } from "../../shared/i18n/i18n";

/** The stages of the pipeline in the order they run, as the event stream names them
 * (`StageKind::label` in crates/application/src/stage.rs). */
export const STAGES = ["fetch", "discover", "analyze", "resolve", "measure", "persist"] as const;

/** The stage whose write reports its steps. */
export const STEP_STAGE = "persist";
/** How often the time of a step is renewed while it runs, in milliseconds. */
export const STEP_TICK_MS = 1000;
/** Seconds in a minute, for the time of a step as "4:12". */
export const SECONDS_PER_MINUTE = 60;
/** The width of the seconds of a time as "4:12". */
export const CLOCK_SECONDS_WIDTH = 2;

/** The names of the steps of writing a snapshot (`PersistStep` in crates/application). */
export const STEP_TEXT: Record<string, MessageKey> = {
  header: I18N_KEYS.SCAN_STEP_HEADER,
  prune_before_load: I18N_KEYS.SCAN_STEP_PRUNE_BEFORE_LOAD,
  facts: I18N_KEYS.SCAN_STEP_FACTS,
  references: I18N_KEYS.SCAN_STEP_REFERENCES,
  reference_indexes: I18N_KEYS.SCAN_STEP_REFERENCE_INDEXES,
  fingerprints: I18N_KEYS.SCAN_STEP_FINGERPRINTS,
  details: I18N_KEYS.SCAN_STEP_DETAILS,
  integrity: I18N_KEYS.SCAN_STEP_INTEGRITY,
  read_models: I18N_KEYS.SCAN_STEP_READ_MODELS,
  prune: I18N_KEYS.SCAN_STEP_PRUNE,
  commit: I18N_KEYS.SCAN_STEP_COMMIT,
};

/** The stage whose units are files: its progress is the progress of the whole scan. */
export const FILE_STAGE = "analyze";
/** The stage that compares every file with the newest snapshot. */
export const CHECK_STAGE = "discover";

/** The kinds of events on the stream (`WireEvent` in crates/server/src/events.rs). */
export const EVENT = {
  progressed: "scan_progressed",
  created: "snapshot_created",
  stopped: "scan_stopped",
  changed: "files_changed",
} as const;

/** The events of a level 2 pass (ADR 0072, decision 6). */
export const LEVEL2_EVENT = {
  progressed: "level2_progressed",
  finished: "level2_finished",
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

/** The command that cancels a running scan, and the key that triggers it. */
export const CANCEL_COMMAND = "projects.cancel";
export const CANCEL_KEY = "Escape";

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

/** Joins branch and commit in the header of the screen, and how many characters of the commit
 * are shown. */
export const BRANCH_COMMIT_JOIN = " @ ";
export const SHORT_COMMIT_LENGTH = 7;

/** The state of a screen that has no run to show. */
export const IDLE = "idle";

/** The most a scan that still runs says of itself: the stages after the files are not counted. */
export const LAST_RUNNING_PERCENT = 99;

/** Start of the id of the note under a stage that cannot say how far it has come. */
export const NOTE_ID_PREFIX = "scan-stage-note-";

/** The columns of the language table by id, with their head texts, and the key the widths are kept under. */
export const LANGUAGE_TABLE_KEY = "languages";
export const LANGUAGE_COLUMNS = [
  { id: "language", label: I18N_KEYS.SCAN_COLUMN_LANGUAGE, number: false },
  { id: "tier", label: I18N_KEYS.SCAN_COLUMN_TIER, number: false },
  { id: "files", label: I18N_KEYS.SCAN_COLUMN_FILES, number: true },
  { id: "lines", label: I18N_KEYS.SCAN_COLUMN_LINES, number: true },
  { id: "symbols", label: I18N_KEYS.SCAN_COLUMN_SYMBOLS, number: true },
  { id: "parseTime", label: I18N_KEYS.SCAN_COLUMN_PARSE_TIME, number: true },
] as const;

/** Id of the command that lists the last scans of a project (ADR 0097). */
export const COMMAND_SCANS_LIST = "scans.list";

/** The columns of the table of the last scans by id, with their head texts, and the key the widths
 * are kept under. */
export const HISTORY_TABLE_KEY = "scan-history";
export const HISTORY_COLUMNS = [
  { id: "time", label: I18N_KEYS.SCAN_HISTORY_COLUMN_TIME, number: false },
  { id: "mode", label: I18N_KEYS.SCAN_HISTORY_COLUMN_MODE, number: false },
  { id: "duration", label: I18N_KEYS.SCAN_HISTORY_COLUMN_DURATION, number: true },
  { id: "files", label: I18N_KEYS.SCAN_HISTORY_COLUMN_FILES, number: true },
  { id: "changed", label: I18N_KEYS.SCAN_HISTORY_COLUMN_CHANGED, number: true },
] as const;

/** How the day and the time of a scan are written in the list of the last scans. */
export const HISTORY_MOMENT: Intl.DateTimeFormatOptions = {
  dateStyle: "short",
  timeStyle: "short",
};

/** The unit after a duration in seconds. */
export const SECONDS_UNIT = " s";

/** Between the parts of the note under a figure. */
export const NOTE_SEPARATOR = " · ";
