/** Drawing of an icon: a 24 by 24 grid, strokes only (blueprint 15.2). */
export const ICON_VIEW_BOX = "0 0 24 24";
export const ICON_SIZE = 20;
/** Icons that the top bar names. */
export const ICON_SEARCH = "search";
export const ICON_BRANCH = "branch";
export const ICON_BELL = "bell";
export const ICON_DOWNLOAD = "download";
/** Sizes of the small icons of the top bar. */
export const ICON_SIZE_SMALL = 13;
export const ICON_SIZE_MEDIUM = 16;
export const ICON_STROKE = "currentColor";
export const ICON_STROKE_WIDTH = 1.8;
export const ICON_NONE = "none";
export const ICON_ROUND = "round";

/** Paths of the icons of the rail. */
export const ICON_PATHS = {
  projects: ["M3 7h6l2 2h10v10H3z"],
  explorer: ["M4 5h7M8 10h8M8 15h8M4 20h7M4 5v15"],
  collapse: ["M7 15l5-5 5 5", "M7 9l5-5 5 5"],
  locate: ["M12 9a3 3 0 1 0 0 6 3 3 0 0 0 0-6z", "M12 2v4M12 18v4M2 12h4M18 12h4"],
  refresh: ["M20 11a8 8 0 1 0-2.3 5.7", "M20 4v7h-7"],
  graph: [
    "M3.5 6a2.5 2.5 0 1 0 5 0 2.5 2.5 0 1 0-5 0",
    "M15.5 8a2.5 2.5 0 1 0 5 0 2.5 2.5 0 1 0-5 0",
    "M7.5 18a2.5 2.5 0 1 0 5 0 2.5 2.5 0 1 0-5 0",
    "M8.4 6.4l7.2 1.2M7 8.3l2.2 7.4M16.6 10.2l-5 6",
  ],
  scan: ["M13 3L5 14h6l-1 7 8-11h-6z"],
  rules: ["M12 3l8 3v6c0 4.5-3.2 7.8-8 9-4.8-1.2-8-4.5-8-9V6z", "M8.5 12l2.5 2.5 4.5-5"],
  stats: ["M4 20V10", "M10 20V4", "M16 20v-8", "M2 20h20"],
  search: ["M11 5a6 6 0 1 0 0 12 6 6 0 0 0 0-12z", "M20 20l-4.5-4.5"],
  branch: [
    "M4 5a2 2 0 1 0 4 0 2 2 0 1 0-4 0",
    "M4 19a2 2 0 1 0 4 0 2 2 0 1 0-4 0",
    "M16 7a2 2 0 1 0 4 0 2 2 0 1 0-4 0",
    "M6 7v10M18 9c0 5-12 3-12 8",
  ],
  download: ["M12 4v11M7 10l5 5 5-5", "M5 20h14"],
  copy: ["M9 9h11v11H9z", "M5 15H4V4h11v1"],
  check: ["M5 12.5l4.5 4.5L19 7"],
  cross: ["M6 6l12 12M18 6L6 18"],
  bell: ["M6 8a6 6 0 0 1 12 0c0 7 3 8 3 8H3s3-1 3-8", "M10 20a2 2 0 0 0 4 0"],
  annotations: [
    "M9 6h11M9 12h11M9 18h11",
    "M3.5 6l1.2 1.2L7 5M3.5 12l1.2 1.2L7 11M3.5 18l1.2 1.2L7 17",
  ],
  edit: ["M4 20l1-4L16.5 4.5a2 2 0 0 1 3 3L8 19z", "M14 7l3 3"],
  trash: ["M4 7h16", "M9 7V4h6v3", "M6 7l1 13h10l1-13", "M10 11v6M14 11v6"],
  settings: [
    "M12 9a3 3 0 1 0 0 6 3 3 0 0 0 0-6z",
    "M12 3v3M12 18v3M3 12h3M18 12h3M5.6 5.6l2.1 2.1M16.3 16.3l2.1 2.1M18.4 5.6l-2.1 2.1M7.7 16.3l-2.1 2.1",
  ],
} as const;

/** Separates class names in the `class` attribute. */
export const CLASS_SEPARATOR = " ";

/** Tone of a pill that has none. */
export const NEUTRAL_TONE = "neutral";

/** The tones of a pill. */
export const TONE = {
  neutral: "neutral",
  accent: "accent",
  good: "good",
  warn: "warn",
  bad: "bad",
} as const;
/** Where a name may break: right after `.`, `_` or `:`. */
export const BREAK_AFTER = /(?<=[._:])/;

/** The ARIA role of a note that screen readers announce at once. */
export const ALERT_ROLE = "alert";

/** How long the button that copies details says that it did. */
export const COPIED_MS = 2000;
/** Height of the box that shows the details when they cannot be copied. */
export const DETAILS_ROWS = 4;
/** What stands between the code and the message in the details of an error note. */
export const DETAILS_SEPARATOR = ": ";
/** How long an answer may take before the screen says that it is still loading, in milliseconds. */
export const SLOW_LOAD_DELAY_MS = 1000;

/** The states of a try to copy to the clipboard. */
export const COPY = { copied: "copied", failed: "failed" } as const;

/** Tab, and what a dialog lets the Tab key reach. */
export const TAB_KEY = "Tab";
export const FOCUSABLE =
  "button:not(:disabled):not([tabindex='-1']), input:not(:disabled):not([tabindex='-1']), select:not(:disabled):not([tabindex='-1'])";
