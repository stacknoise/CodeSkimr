/** Ids of the commands of the project list (crates/app/src/ui_commands/consts.rs). */
export const COMMAND = {
  list: "projects.list",
  add: "projects.add",
  rescan: "projects.rescan",
} as const;

/** How often the list is read while a scan runs, in milliseconds. */
export const POLL_MS = 1500;

/** Branch that the form starts with. */
export const DEFAULT_BRANCH = "main";

/** What a project's source is, as the server names it. */
export const SOURCE_KIND = { folder: "folder", git: "git" } as const;

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

/** The colour class of a language bar segment; languages without one share the last. */
export const LANGUAGE_TONE: Record<string, string> = {
  csharp: "tone1",
  typescript: "tone2",
  tsx: "tone2",
  javascript: "tone2",
  kotlin: "tone3",
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
  rescan: "r",
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
