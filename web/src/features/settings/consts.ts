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
export const ORIGIN = { default: "default", user: "user", project: "project", cli: "cli" } as const;

/** The pill of each origin: a value from a file or a flag stands out from a default. */
export const ORIGIN_TEXTS = {
  default: I18N_KEYS.SETTINGS_EFFECTIVE_ORIGIN_DEFAULT,
  user: I18N_KEYS.SETTINGS_EFFECTIVE_ORIGIN_USER,
  project: I18N_KEYS.SETTINGS_EFFECTIVE_ORIGIN_PROJECT,
  cli: I18N_KEYS.SETTINGS_EFFECTIVE_ORIGIN_CLI,
} as const;

/** The columns of the table of effective settings and the key their widths are kept under. */
export const EFFECTIVE_TABLE_KEY = "effective-settings";
export const EFFECTIVE_COLUMNS = [
  { id: "key", label: I18N_KEYS.SETTINGS_EFFECTIVE_COLUMN_KEY },
  { id: "value", label: I18N_KEYS.SETTINGS_EFFECTIVE_COLUMN_VALUE },
  { id: "origin", label: I18N_KEYS.SETTINGS_EFFECTIVE_COLUMN_ORIGIN },
] as const;

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
export const EFFECTIVE_SEARCH_ID = "effective-search";
export const EFFECTIVE_ONLY_CHANGED_ID = "effective-only-changed";
