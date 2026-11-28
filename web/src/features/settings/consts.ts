import type { MessageKey } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import type { ThemeName } from "../../shared/theme/tokens";

/** Name and one-line description of every theme, as message keys. A theme that is added to
 * `design/tokens.toml` needs an entry here and texts in `locales/`; the compiler asks for both. */
export const THEME_TEXTS: Record<ThemeName, { name: MessageKey; note: MessageKey }> = {
  hell: {
    name: I18N_KEYS.SETTINGS_THEME_NAME_HELL,
    note: I18N_KEYS.SETTINGS_THEME_NOTE_HELL,
  },
  petrol: {
    name: I18N_KEYS.SETTINGS_THEME_NAME_PETROL,
    note: I18N_KEYS.SETTINGS_THEME_NOTE_PETROL,
  },
  dunkel: {
    name: I18N_KEYS.SETTINGS_THEME_NAME_DUNKEL,
    note: I18N_KEYS.SETTINGS_THEME_NOTE_DUNKEL,
  },
};

/** Tokens shown as swatches under each preview, in the order of the mock. */
export const SWATCH_TOKENS = ["bg", "surface", "ink", "accent", "good", "warn", "bad"] as const;

/** Language names are shown in their own language. */
export const LANGUAGE_NAME_KEYS = {
  de: I18N_KEYS.SETTINGS_LANGUAGE_NAME_DE,
  en: I18N_KEYS.SETTINGS_LANGUAGE_NAME_EN,
} as const;

/** The projects of the miniature in the preview (sample data, as in the mock). */
export const PREVIEW_NEW_FILES = 12;

/** Ids that tie a label to its field. */
export const FOLLOW_SYSTEM_ID = "follow-system";
export const LANGUAGE_FIELD_ID = "language";
/** Name of the second sample project in the preview. */
export const PREVIEW_OTHER_PROJECT = "haac-bridge";

/** What the table of indexers shows where a value is missing. */
export const NO_VALUE = "–";
/** How long "Copied" stays on the button, in milliseconds. */
export const COPIED_MS = 2000;

/** The columns of the indexer table by id, with their head texts, and the key the widths are kept under. */
export const INDEXER_TABLE_KEY = "indexers";
export const INDEXER_COLUMNS = [
  { id: "tool", label: I18N_KEYS.SETTINGS_INDEXERS_COLUMN_TOOL },
  { id: "state", label: I18N_KEYS.SETTINGS_INDEXERS_COLUMN_STATE },
  { id: "version", label: I18N_KEYS.SETTINGS_INDEXERS_COLUMN_VERSION },
  { id: "path", label: I18N_KEYS.SETTINGS_INDEXERS_COLUMN_PATH },
] as const;

/** Id of the command that lists the settings in effect. */
export const EFFECTIVE_COMMAND = "settings.effective";

/** Where a setting got its value; the names are those of the answer of `settings.effective`. */
export const ORIGIN = {
  default: "default",
  user: "user",
  ui: "ui",
  project: "project",
  cli: "cli",
} as const;

/** The pill of each origin: a value from a file or a flag stands out from a default. */
export const ORIGIN_TEXTS = {
  default: I18N_KEYS.SETTINGS_EFFECTIVE_ORIGIN_DEFAULT,
  user: I18N_KEYS.SETTINGS_EFFECTIVE_ORIGIN_USER,
  ui: I18N_KEYS.SETTINGS_EFFECTIVE_ORIGIN_UI,
  project: I18N_KEYS.SETTINGS_EFFECTIVE_ORIGIN_PROJECT,
  cli: I18N_KEYS.SETTINGS_EFFECTIVE_ORIGIN_CLI,
} as const;

/** The columns of the table of effective settings and the key their widths are kept under. */
export const EFFECTIVE_TABLE_KEY = "effective-settings";
export const EFFECTIVE_COLUMNS = [
  { id: "key", label: I18N_KEYS.SETTINGS_EFFECTIVE_COLUMN_KEY },
  { id: "value", label: I18N_KEYS.SETTINGS_EFFECTIVE_COLUMN_VALUE },
  { id: "origin", label: I18N_KEYS.SETTINGS_EFFECTIVE_COLUMN_ORIGIN },
  { id: "actions", label: I18N_KEYS.SETTINGS_EFFECTIVE_COLUMN_ACTIONS },
] as const;

/** The command that saves or forgets one number or switch (ADR 0118). */
export const SET_VALUE_COMMAND = "ui_settings.set_value";

/** Whether and how a setting can be changed, as `settings.effective` says: `scan` and `start` can
 * be changed (the value counts from the next scan or the next start), `cli` is set with a flag
 * and `locked` is for the configuration file only. */
export const EDIT = {
  scan: "scan",
  start: "start",
  cli: "cli",
  locked: "locked",
} as const;

/** The kinds of value the dialog edits. */
export const VALUE_KIND = { switch: "switch", int: "int", float: "float" } as const;
/** The words of a switch value in the answer. */
export const SWITCH_ON = "true";
export const SWITCH_OFF = "false";
/** A whole number, and a number with an optional fraction, as the field may hold them. */
export const INT_PATTERN = /^-?\d+$/;
export const FLOAT_PATTERN = /^-?\d+(\.\d+)?$/;
/** Ids that tie the dialog to its title and its field. */
export const SETTING_DIALOG_TITLE_ID = "setting-dialog-title";
export const SETTING_DIALOG_FIELD_ID = "setting-dialog-field";
/** The key that confirms the field and the key that closes the dialog. */
export const SETTING_SUBMIT_KEY = "Enter";
export const SETTING_CANCEL_KEY = "Escape";

/** The tables of `config/default.toml` in the order the screen shows them; a table that is not
 * listed here comes after them under its own name. */
export const GROUP_TEXTS: Record<string, MessageKey> = {
  scan: I18N_KEYS.SETTINGS_EFFECTIVE_GROUP_SCAN,
  level2: I18N_KEYS.SETTINGS_EFFECTIVE_GROUP_LEVEL2,
  server: I18N_KEYS.SETTINGS_EFFECTIVE_GROUP_SERVER,
  ui: I18N_KEYS.SETTINGS_EFFECTIVE_GROUP_UI,
  editor: I18N_KEYS.SETTINGS_EFFECTIVE_GROUP_EDITOR,
  export: I18N_KEYS.SETTINGS_EFFECTIVE_GROUP_EXPORT,
  layers: I18N_KEYS.SETTINGS_EFFECTIVE_GROUP_LAYERS,
  metrics: I18N_KEYS.SETTINGS_EFFECTIVE_GROUP_METRICS,
  budgets: I18N_KEYS.SETTINGS_EFFECTIVE_GROUP_BUDGETS,
  conventions: I18N_KEYS.SETTINGS_EFFECTIVE_GROUP_CONVENTIONS,
  similarity: I18N_KEYS.SETTINGS_EFFECTIVE_GROUP_SIMILARITY,
  keys: I18N_KEYS.SETTINGS_EFFECTIVE_GROUP_KEYS,
  mcp: I18N_KEYS.SETTINGS_EFFECTIVE_GROUP_MCP,
  snapshots: I18N_KEYS.SETTINGS_EFFECTIVE_GROUP_SNAPSHOTS,
  watcher: I18N_KEYS.SETTINGS_EFFECTIVE_GROUP_WATCHER,
  annotations: I18N_KEYS.SETTINGS_EFFECTIVE_GROUP_ANNOTATIONS,
  graph: I18N_KEYS.SETTINGS_EFFECTIVE_GROUP_GRAPH,
};

/** The unit that the end of a key name tells, with the text that puts the value in front of it. */
export const UNIT_SUFFIXES: [string, MessageKey][] = [
  ["_ms", I18N_KEYS.SETTINGS_EFFECTIVE_UNIT_MS],
  ["_seconds", I18N_KEYS.SETTINGS_EFFECTIVE_UNIT_SECONDS],
  ["_days", I18N_KEYS.SETTINGS_EFFECTIVE_UNIT_DAYS],
  ["_mb", I18N_KEYS.SETTINGS_EFFECTIVE_UNIT_MB],
  ["_kb", I18N_KEYS.SETTINGS_EFFECTIVE_UNIT_KB],
  ["_bytes", I18N_KEYS.SETTINGS_EFFECTIVE_UNIT_BYTES],
  ["_percent", I18N_KEYS.SETTINGS_EFFECTIVE_UNIT_PERCENT],
  ["_chars", I18N_KEYS.SETTINGS_EFFECTIVE_UNIT_CHARS],
];

/** How a list is written in the value column. */
export const LIST_SEPARATOR = ", ";
/** The separator of a dotted key path. */
export const KEY_SEPARATOR = ".";

/** Ids that tie a label to its field in the table of effective settings. */
export const EFFECTIVE_ONLY_CHANGED_ID = "effective-only-changed";

/** The tabs of the settings screen (ADR 0098), in the order of the tab bar. */
export const SETTINGS_TAB = {
  appearance: "appearance",
  analysis: "analysis",
  access: "access",
  effective: "effective",
  info: "info",
  diagnosis: "diagnosis",
} as const;
/** One tab of the settings screen. */
export type SettingsTab = (typeof SETTINGS_TAB)[keyof typeof SETTINGS_TAB];
/** The tabs with their texts. */
export const SETTINGS_TABS = [
  { id: SETTINGS_TAB.appearance, label: I18N_KEYS.SETTINGS_TAB_APPEARANCE },
  { id: SETTINGS_TAB.analysis, label: I18N_KEYS.SETTINGS_TAB_ANALYSIS },
  { id: SETTINGS_TAB.access, label: I18N_KEYS.SETTINGS_TAB_ACCESS },
  { id: SETTINGS_TAB.effective, label: I18N_KEYS.SETTINGS_TAB_EFFECTIVE },
  { id: SETTINGS_TAB.info, label: I18N_KEYS.SETTINGS_TAB_INFO },
  { id: SETTINGS_TAB.diagnosis, label: I18N_KEYS.SETTINGS_TAB_DIAGNOSIS },
] as const;
/** Id prefix of the tab buttons and the id of the tab panel. */
export const SETTINGS_TAB_ID_PREFIX = "settings-tab-";
export const SETTINGS_PANEL_ID = "settings-panel";
/** Id of the search field in the tab bar. */
export const SETTINGS_SEARCH_ID = "settings-search";
/** `tabindex` of the selected tab and of the others (the arrow keys reach them). */
export const SETTINGS_TAB_STOP = { on: 0, off: -1 } as const;

/** The commands of the saved settings (ADR 0103). */
export const UI_SETTINGS_COMMAND = "ui_settings.get";
export const SET_INDEXER_MEMORY_COMMAND = "ui_settings.set_indexer_memory";
/** The key of the memory limit of one indexer in the effective settings, before its name. */
export const INDEXER_MEMORY_PREFIX = "level2.indexer_memory_mb.";
/** The general memory limit of an indexer in MB, which `0` follows. */
export const GENERAL_MEMORY_KEY = "level2.max_memory_mb";
/** The most MB the server accepts for one indexer (`MAX_INDEXER_MEMORY_MB`). */
export const INDEXER_MEMORY_MAX = 1_048_576;
/** Ids that tie the input of an indexer to its note. */
export const INDEXER_MEMORY_ID_PREFIX = "indexer-memory-";

/** The commands of the log (ADR 0115). */
export const LOG_STATUS_COMMAND = "log.status";
export const LOG_SET_LEVEL_COMMAND = "log.set_level";
export const LOG_TAIL_COMMAND = "log.tail";
export const LOG_CLEAR_COMMAND = "log.clear";
export const LOG_OPEN_FOLDER_COMMAND = "log.open_folder";
/** How many of the last lines the panel shows, and how many "Copy log" copies. */
export const LOG_SHOWN_LINES = 8;
export const LOG_COPIED_LINES = 2000;
/** How often the last lines are read again while the tab is open, in milliseconds. */
export const LOG_REFRESH_MS = 3000;
/** The texts of the levels by their wire word. */
export const LOG_LEVEL_TEXTS: Record<string, MessageKey> = {
  off: I18N_KEYS.SETTINGS_LOG_LEVEL_OFF,
  warn: I18N_KEYS.SETTINGS_LOG_LEVEL_WARN,
  info: I18N_KEYS.SETTINGS_LOG_LEVEL_INFO,
  debug: I18N_KEYS.SETTINGS_LOG_LEVEL_DEBUG,
  trace: I18N_KEYS.SETTINGS_LOG_LEVEL_TRACE,
};
/** Seconds in a minute and in an hour, to say how long a level lasts. */
export const SECONDS_PER_MINUTE = 60;
export const MINUTES_PER_HOUR = 60;
/** The line break of the copied log. */
export const LOG_LINE_BREAK = "\n";

/** The code of a refused value (a key or kind the server does not accept). */
export const VALUE_REFUSED_CODE = "API-101";
/** What stands between the kind of a value and its default in the hint under the field. */
export const HINT_SEPARATOR = " · ";
