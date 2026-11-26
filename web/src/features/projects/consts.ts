import type { MessageKey } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";

/** Ids of the commands of the project list (crates/app/src/ui_commands/consts.rs). */
export const COMMAND = {
  list: "projects.list",
  add: "projects.add",
  rescan: "projects.rescan",
  remove: "projects.remove",
  rename: "projects.rename",
  suggestName: "projects.suggest_name",
  tools: "projects.tools",
} as const;

/** Ids of the commands of the scope of a project (crates/app/src/ui_commands/consts.rs). */
export const SCOPE_COMMAND = { get: "scope.get", set: "scope.set" } as const;
/** Separates the parts of the path of a folder in the scope. */
export const SCOPE_PATH_SEPARATOR = "/";
/** Ids of the headings of the scope page, which name their sections. */
export const SCOPE_HEADING_ID = { languages: "scope-languages", folders: "scope-folders" } as const;
/** The custom property that carries the depth of a folder in the tree to the style sheet. */
export const SCOPE_DEPTH_VAR = "--scope-depth";
/** The name of the radio group of the hint in the window "New project". */
export const OVERLAP_GROUP = "overlap";

/** Ids of the folder dialog commands (crates/app/src/ui_commands/consts.rs). */
export const FOLDER_DIALOG_COMMAND = {
  info: "projects.folder_dialog",
  pick: "projects.pick_folder",
} as const;

/** The role of a tool that an indexer needs besides itself. */
export const TOOL_ROLE_TOOLCHAIN = "toolchain";

/** Id of the command that lists the indexers of this machine, without a project. */
export const INDEXERS_COMMAND = "tools.indexers";

/** Ids of the credential commands (crates/app/src/ui_commands/consts.rs). */
export const CREDENTIAL_COMMAND = {
  list: "credentials.list",
  set: "credentials.set",
  remove: "credentials.delete",
} as const;
/** Where this browser keeps the names of the credentials it stored. */
export const CREDENTIAL_NAMES_KEY = "codeskimr.credentials.names";

/** A pasted text that is clearly a path of a folder. */
export const FOLDER_PATH = /^(?:[/~\\]|[a-z]:[\\/])/i;
/** A path on the network (`\\server\share` or `//server/share`): looking it up can wait for a
 * remote machine, so no name is suggested while it is typed. */
export const NETWORK_PATH = /^(?:\\\\|\/\/)/;
/** How long the form waits after the last key before it asks for a suggested name. */
export const SUGGEST_DELAY_MS = 300;
/** A pasted text that is the address of a repository, not a path. */
export const GIT_ADDRESS = /^(?:[a-z][a-z0-9+.-]*:\/\/|git@)/i;

/** What the check found out about one tool, as the server names it. */
export const TOOL_STATE = {
  ready: "ready",
  missing: "missing",
  tooOld: "too_old",
  tooNew: "too_new",
  versionUnknown: "version_unknown",
  notRunnable: "not_runnable",
} as const;

/** Separates the languages in a line about a tool. */
export const TOOL_LANGUAGE_SEPARATOR = ", ";

/** How often the list is read while a scan runs, in milliseconds. */
export const POLL_MS = 1500;
/** How often a relative time ("3 min ago") is renewed, in milliseconds. */
export const NOW_TICK_MS = 5000;

/** Key that sends a field. */
export const ENTER_KEY = "Enter";

/** Branch that the form starts with. */
export const DEFAULT_BRANCH = "main";

/** What a project's source is, as the server names it. */
export const SOURCE_KIND = { folder: "folder", git: "git" } as const;
/** One of the kinds of source. */
export type SourceKind = (typeof SOURCE_KIND)[keyof typeof SOURCE_KIND];

/** How a scan is named by the server. */
export const SCAN_MODE = { full: "full", incremental: "incremental" } as const;

/** Display names of the languages (proper names, not translated). */
export const LANGUAGE_NAMES: Record<string, string> = {
  c: "C",
  cpp: "C++",
  csharp: "C#",
  go: "Go",
  java: "Java",
  javascript: "JavaScript",
  kotlin: "Kotlin",
  php: "PHP",
  python: "Python",
  ruby: "Ruby",
  rust: "Rust",
  tsx: "TSX",
  typescript: "TypeScript",
};

/** The colour class of a language bar segment; TypeScript and TSX share one colour (same family), every
 * other common language of a mixed project has its own; languages without one share the last. */
export const LANGUAGE_TONE: Record<string, string> = {
  csharp: "tone1",
  typescript: "tone2",
  tsx: "tone2",
  javascript: "tone3",
  kotlin: "tone1",
  python: "tone4",
  java: "tone5",
};
export const LANGUAGE_TONE_OTHER = "tone6";

/** Percent of a whole. */
export const PERCENT = 100;
/** Time units in seconds, largest first, for "3 min ago". */
export const TIME_UNITS: readonly [Intl.RelativeTimeFormatUnit, number][] = [
  ["day", 86_400],
  ["hour", 3_600],
  ["minute", 60],
];
/** Milliseconds in a second. */
export const MS_PER_SECOND = 1000;
/** Decimals of a scan duration. */
export const SECONDS_DECIMALS = 1;
/** Microseconds in a second, for the parse time. */
export const MICROS_PER_SECOND = 1_000_000;
/** Decimals of the parse time in seconds. */
export const PARSE_TIME_DECIMALS = 2;

/** Prefix of the address of a dropped file or folder. */
export const FILE_SCHEME = "file://";
/** Types of dropped data that may hold a path or an address, best first. */
export const DROP_TYPES = ["text/uri-list", "text/plain"] as const;
/** A drive letter path inside a file address: `/C:/work`. */
export const DRIVE_PATH = /^\/[A-Za-z]:/;
/** Lines of a `text/uri-list` that are comments. */
export const URI_COMMENT = "#";

/** Keys of the screen. */
export const KEY = {
  newProject: "n",
  showScan: "r",
  open: "Enter",
  down: "ArrowDown",
  up: "ArrowUp",
} as const;

/** What a drop carried. */
export const DROP = { text: "text", opaque: "opaque", empty: "empty" } as const;
/** Options of the relative time: "yesterday" instead of "1 day ago". */
export const RELATIVE_OPTIONS: Intl.RelativeTimeFormatOptions = { numeric: "auto" };
/** The unit of times below a minute. */
export const SECOND_UNIT: Intl.RelativeTimeFormatUnit = "second";
/** Splits dropped text into lines. */
export const LINE_BREAK = /\r?\n/;
/** Separates the languages in the text under the bar. */
export const LANGUAGE_SEPARATOR = " · ";

/** Icon of the button that scans a project again. */
export const RESCAN_ICON = "refresh";
/** One step down and up in the list. */
export const STEP_DOWN = 1;
export const STEP_UP = -1;
/** Index of a project that is not in the list. */
export const NOT_FOUND = -1;

/** Elements that act on `Enter` themselves. */
export const PRESSABLE_TAGS = ["BUTTON", "A"] as const;

/** Ids of the commands of the level 2 switch (ADR 0071, decision 4). */
export const LEVEL2_COMMAND = {
  status: "level2.status",
  revoke: "level2.revoke",
} as const;

/** The answers of a scan about stage 2 (ADR 0072, decision 4). */
export const LEVEL2_START = {
  started: "started",
  notStarted: "not_started",
  notRequested: "not_requested",
} as const;

/** The stage labels of the events of a level 2 run. */
export const LEVEL2_STAGE = { running: "level2_running", importing: "level2_importing" } as const;

/** The texts of the reasons why stage 2 did not start. */
export const NOT_STARTED_TEXT: Record<string, MessageKey> = {
  not_permitted: I18N_KEYS.PROJECTS_LEVEL2_NOT_STARTED_NOT_PERMITTED,
  not_confirmed: I18N_KEYS.PROJECTS_LEVEL2_NOT_STARTED_NOT_CONFIRMED,
  busy: I18N_KEYS.PROJECTS_LEVEL2_NOT_STARTED_BUSY,
  paused: I18N_KEYS.PROJECTS_LEVEL2_NOT_STARTED_PAUSED,
};

/** The route that asks the server for the console window of the confirmation; it is not a command
 * of the registry (ADR 0071, decision 4). */
export const CONFIRM_WINDOW_PATH = "/api/v1/level2/confirm-window";

/** What `level2.status` says about a project (ADR 0071, decision 3). */
export const LEVEL2_STATE = {
  on: "on",
  off: "off",
  notEnabled: "not_enabled",
  locked: "locked",
} as const;

/** How the confirmation can be started on this system: in a console window, or by a command
 * that the user copies. */
export const LAUNCH = { window: "window", copy: "copy" } as const;

/** What the switch of a project is doing besides showing its state. */
export const LEVEL2_PHASE = {
  idle: "idle",
  opening: "opening",
  waiting: "waiting",
  timedOut: "timed_out",
  cancelled: "cancelled",
  askRevoke: "ask_revoke",
  revoking: "revoking",
  revoked: "revoked",
} as const;

/** The command of the terminal that confirms level 2, without the project id. */
export const CONFIRM_COMMAND_PREFIX = "codeskimr level2 confirm ";
/** The two lines that the user puts into `config.toml` to allow level 2 (blueprint 14). */
export const LEVEL2_CONFIG_SNIPPET = "[level2]\nenabled = true";

/** How often the state is asked while the confirmation is open, in seconds, and for how long at
 * most (`ui.level2_poll_seconds`, `ui.level2_poll_max_seconds`, ADR 0071, decision 3). */
export const LEVEL2_POLL_SECONDS = 2;
export const LEVEL2_POLL_MAX_SECONDS = 300;
/** The pause after a window was opened, in seconds (`level2.confirm_window_cooldown_seconds`). */
export const LEVEL2_COOLDOWN_SECONDS = 30;
/** How long "Copied" stays on the copy button, in milliseconds. */
export const COPIED_MS = 2000;

/** Id of the switch of a project, so that focus can return to it. */
export const LEVEL2_SWITCH_ID_PREFIX = "level2-switch-";
/** Id of the level 2 section of a project, to tell whether the focus is inside it. */
export const LEVEL2_PANEL_ID_PREFIX = "level2-panel-";
/** Id of the scan section of a project. */
export const SCAN_PANEL_ID_PREFIX = "scan-panel-";
/** Key that closes the question before switching off. */
export const ESCAPE_KEY = "Escape";

/** The error codes that the switch handles itself (ADR 0071, decision 11). */
export const LEVEL2_ERROR = {
  windowFailed: "APP-604",
  switchOff: "CFG-105",
  rateLimited: "API-504",
} as const;
/** Hours and minutes of a time of day. */
export const CLOCK_OPTIONS: Intl.DateTimeFormatOptions = { hour: "2-digit", minute: "2-digit" };

/** The tabs of the project page, in the order they are shown (ADR 0096). */
export const PROJECT_TAB = {
  overview: "overview",
  scan: "scan",
  scope: "scope",
  level2: "level2",
  tools: "tools",
} as const;
/** One tab of the project page. */
export type ProjectTab = (typeof PROJECT_TAB)[keyof typeof PROJECT_TAB];
/** The tabs with their texts. */
export const PROJECT_TABS = [
  { id: PROJECT_TAB.overview, label: I18N_KEYS.PROJECTS_TAB_OVERVIEW },
  { id: PROJECT_TAB.scan, label: I18N_KEYS.PROJECTS_TAB_SCAN },
  { id: PROJECT_TAB.scope, label: I18N_KEYS.PROJECTS_TAB_SCOPE },
  { id: PROJECT_TAB.level2, label: I18N_KEYS.PROJECTS_TAB_LEVEL2 },
  { id: PROJECT_TAB.tools, label: I18N_KEYS.PROJECTS_TAB_TOOLS },
] as const;
/** Separates the parts of the source line of a project and of the list line. */
export const PART_SEPARATOR = " · ";
/** Length of a short commit id. */
export const COMMIT_LENGTH = 7;
/** Joins branch and commit in the source line. */
export const AT_SIGN = " @ ";
/** Units of a size in bytes, and the step between them. */
export const SIZE_UNITS = ["B", "KB", "MB", "GB", "TB"] as const;
export const SIZE_STEP = 1024;
/** Decimals of a size from megabytes up. */
export const SIZE_DECIMALS = 1;
/** The state of a project database as the server names it. */
export const DB_STATE = {
  ok: "ok",
  missing: "missing",
  removing: "removing",
  unreadable: "unreadable",
} as const;

/** Icon of the button that removes a project. */
export const REMOVE_ICON = "trash";
/** Icon of the button that renames a project, and its size. */
export const RENAME_ICON = "edit";
export const RENAME_ICON_SIZE = 14;
/** Index of "MB" in `SIZE_UNITS`. */
export const MEGABYTE_UNIT = 2;

/** Id prefixes of the tab buttons and the tab panels, for `aria-controls`. */
export const TAB_ID = "project-tab-";
export const PANEL_ID = "project-panel-";
/** How the arrow keys move between the tabs. */
export const TAB_STEP: Record<string, number> = { ArrowRight: 1, ArrowLeft: -1 };
/** `tabindex` of the selected tab and of the others (the arrow keys reach them). */
export const TAB_IN = 0;
export const TAB_OUT = -1;

/** The signs of the dialog "Remove project": what goes, what stays, and the warning. */
export const REMOVE_MARKS = { goes: "−", stays: "✓", warning: "!" } as const;

/** The id of the title of the dialog "Remove project" and of the window "New project". */
export const REMOVE_DIALOG_TITLE_ID = "remove-title";
export const NEW_PROJECT_TITLE_ID = "new-project-title";

/** How project names are compared for the list: without regard to case and accents, digits as numbers. */
export const NAME_ORDER: Intl.CollatorOptions = { sensitivity: "base", numeric: true };

/** The drag events the screen listens to on the window. */
export const DRAG_EVENT = {
  over: "dragover",
  drop: "drop",
  leave: "dragleave",
  start: "dragstart",
  end: "dragend",
} as const;
