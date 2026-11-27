import { I18N_KEYS } from "../../shared/i18n/keys";

/** Ids of the commands of the annotations (ADR 0068, crates/app/src/ui_commands). */
export const COMMAND = {
  list: "annotations.list",
  summary: "annotations.summary",
  create: "annotations.create",
  markCritical: "annotations.mark_critical",
  addNote: "annotations.add_note",
  addTask: "annotations.add_task",
  update: "annotations.update",
  remove: "annotations.delete",
} as const;

/** The kinds of an annotation. */
export const KIND = { critical: "critical", note: "note", task: "task" } as const;
export type AnnotationKind = (typeof KIND)[keyof typeof KIND];

/** The states of a task. */
export const STATUS = { open: "open", inProgress: "in_progress", done: "done" } as const;
export type TaskStatus = (typeof STATUS)[keyof typeof STATUS];

/** Kind of the target of an annotation on a namespace; types and members carry their own kind. */
export const NAMESPACE_KIND = "namespace";

/** Kind of a module, which the explorer tree shows as a row like a namespace. */
export const MODULE_KIND = "module";

/** Rows per call. The summary and the project-wide list are read page by page until the server
 * names no next page. */
export const PAGE_SIZE = 500;
/** Most pages that are read for one list, so a runaway answer cannot loop for ever. */
export const MAX_PAGES = 100;

/** Separates the kind and the name in the key of a symbol. */
export const TARGET_SEPARATOR = "\u0001";
/** Characters that follow the name of a container in the qualified name of a symbol inside it. */
export const NAME_BOUNDARY = /^[.:]/;
/** Separates the parts of a qualified name. */
export const NAME_SEPARATOR = ".";

/** The tabs of the side panel. */
export const TAB = { info: "info", critical: "critical", notes: "notes", tasks: "tasks" } as const;
export type TabId = (typeof TAB)[keyof typeof TAB];
/** The tabs in the order of the mock. */
export const TABS: readonly TabId[] = [TAB.info, TAB.critical, TAB.notes, TAB.tasks];

/** `localStorage` key and values of the switch "Show annotations" (a per-viewer convenience). */
export const SHOW_TEXT_KEY = "codeskimr.annotations.show-text";
export const FLAG_ON = "1";
export const FLAG_OFF = "0";
/** `localStorage` keys of the two switches of the dependency map. */
export const HIGHLIGHT_KEY = "codeskimr.annotations.highlight";
export const NEIGHBOURS_KEY = "codeskimr.annotations.only-critical";

/** What the board is doing. */
export const BOARD = { loading: "loading", ready: "ready", failed: "failed" } as const;
export type BoardState = (typeof BOARD)[keyof typeof BOARD];
/** What the board shows: the tasks in columns, or the notes in a list. */
export const VIEW = { tasks: "tasks", notes: "notes" } as const;
export type BoardView = (typeof VIEW)[keyof typeof VIEW];
/** The filters of the board. */
export const FILTER = { mine: "mine", critical: "critical", due: "due" } as const;
/** The columns of the board, left to right (mock 16). */
export const COLUMNS: readonly TaskStatus[] = [STATUS.open, STATUS.inProgress, STATUS.done];

/** `localStorage` key of the name of the author of the last annotation made in this browser. */
export const ME_KEY = "codeskimr.annotations.me";

/** The keys of the actions (blueprint 15.4). */
export const HOTKEY = { critical: "K", note: "N", task: "T" } as const;
export const KEY_OF: Readonly<Record<string, keyof typeof HOTKEY>> = {
  k: "critical",
  n: "note",
  t: "task",
};

/** Size of the shield in the tree, in the details and on a diagram node. */
export const SHIELD = { tree: 13, label: 11, map: 10, tab: 14 } as const;
/** Size of the icon of a counter chip. */
export const CHIP_ICON_SIZE = 11;

/** Drawing of the shield with its tick (as in the mock). */
export const SHIELD_PATH = "M12 2l8 3v6c0 5-3.5 9-8 11-4.5-2-8-6-8-11V5z";
export const SHIELD_TICK = "M8.5 12l2.5 2.5 4.5-5";
export const SHIELD_TICK_WIDTH = 2.2;
/** Drawing of the note and the task icon. */
export const NOTE_PATH = "M5 3h10l4 4v14H5zM14 3v5h5M8 12h8M8 16h5";
export const TASK_RECT = { x: 4, y: 4, width: 16, height: 16, rx: 3 } as const;
export const TASK_TICK = "M8 12l3 3 5-6";
export const ICON_BOX = "0 0 24 24";
export const ICON_STROKE_WIDTH = 2;

/** Labels of a status in the order of the cycle of the task button. */
export const STATUS_LABEL: Readonly<
  Record<TaskStatus, (typeof I18N_KEYS)[keyof typeof I18N_KEYS]>
> = {
  open: I18N_KEYS.ANNOTATIONS_STATUS_OPEN,
  in_progress: I18N_KEYS.ANNOTATIONS_STATUS_IN_PROGRESS,
  done: I18N_KEYS.ANNOTATIONS_STATUS_DONE,
};
/** The states a task can take, in the order of the select. */
export const STATUSES: readonly TaskStatus[] = [STATUS.open, STATUS.inProgress, STATUS.done];

/** Milliseconds in a second: the server counts a due date in seconds since 1970. */
export const MS_PER_SECOND = 1000;
/** Format of a date field (`type="date"`): the first ten characters of an ISO date. */
export const DATE_LENGTH = 10;
/** A due date is stored at midday UTC, so that every time zone shows the same day. */
export const DUE_OFFSET_SECONDS = 43_200;
/** The start of a day in UTC, appended to the value of a date field. */
export const DAY_START_UTC = "T00:00:00Z";
/** How a date is written: day, month and year in the order of the language. */
export const DATE_FORMAT = { day: "2-digit", month: "2-digit", year: "numeric" } as const;
/** A due date is a day, not a moment: it is shown in UTC, where it was stored. */
export const DUE_FORMAT = { ...DATE_FORMAT, timeZone: "UTC" } as const;
/** Whether a due date lies in the past is judged by this many seconds in a day. */
export const SECONDS_PER_DAY = 86_400;

/** The kinds of a piece of text in the Markdown subset (ADR 0068, decision 8). */
export const PIECE = {
  text: "text",
  bold: "bold",
  italic: "italic",
  code: "code",
  link: "link",
} as const;
export type PieceKind = (typeof PIECE)[keyof typeof PIECE];
/** Where a part of the subset is recognised: sticky patterns, tried one after the other at each
 * position. A link names its address in the second group; a bare address is its own text. */
export const MARKUP_RULES: readonly { kind: PieceKind; pattern: RegExp }[] = [
  { kind: PIECE.code, pattern: /`([^`\n]+)`/y },
  { kind: PIECE.link, pattern: /\[([^\]\n]+)\]\((https?:\/\/[^)\s]+)\)/y },
  { kind: PIECE.bold, pattern: /\*\*([^*\n]+)\*\*/y },
  { kind: PIECE.italic, pattern: /\*([^*\n]+)\*/y },
  { kind: PIECE.link, pattern: /(https?:\/\/[^\s<>"')]+)/y },
];
/** The schemes that the address of a link may have. */
export const LINK_SCHEMES: readonly string[] = ["http:", "https:"];

/** Ids of the tab buttons (their prefix) and of the panel they control. */
export const TAB_ID_PREFIX = "annotation-tab-";
export const PANEL_ID = "annotation-panel";
/** The roles of the line under the tabs: a failure is announced at once, a hint politely. */
export const ROLE = { alert: "alert", status: "status" } as const;
/** The sign of the button that closes the line under the tabs. */
export const CLOSE_MARK = "\u00d7";

/** The check boxes of a task in the text under a row. */
export const BOX_OPEN = "\u2610";
export const BOX_DONE = "\u2611";

/** Rows of the text fields of the forms. */
export const TEXT_ROWS = 3;
/** The tab stops of the tab list: only the selected tab is reached with Tab, the arrows do the rest. */
export const TAB_STOP = { reachable: 0, skipped: -1 } as const;
