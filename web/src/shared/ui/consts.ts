/** Drawing of an icon: a 24 by 24 grid, strokes only (blueprint 15.2). */
export const ICON_VIEW_BOX = "0 0 24 24";
export const ICON_SIZE = 20;
/** Icons that the top bar names. */
export const ICON_SEARCH = "search";
export const ICON_BRANCH = "branch";
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
  search: ["M11 5a6 6 0 1 0 0 12 6 6 0 0 0 0-12z", "M20 20l-4.5-4.5"],
  branch: [
    "M4 5a2 2 0 1 0 4 0 2 2 0 1 0-4 0",
    "M4 19a2 2 0 1 0 4 0 2 2 0 1 0-4 0",
    "M16 7a2 2 0 1 0 4 0 2 2 0 1 0-4 0",
    "M6 7v10M18 9c0 5-12 3-12 8",
  ],
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
