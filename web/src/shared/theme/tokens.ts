// Generated from design/tokens.toml by tools/codegen. Do not edit.

/** The themes, in the order they are offered. */
export const THEMES = ["hell", "petrol", "dunkel"] as const;

/** Name of a theme. */
export type ThemeName = (typeof THEMES)[number];

/** Theme until the user chooses one. */
export const DEFAULT_THEME: ThemeName = "petrol";

/** Theme until the user chooses one, on a dark system. */
export const DARK_THEME: ThemeName = "dunkel";

/** Key of the mirror of the choice in `localStorage`. */
export const THEME_STORAGE_KEY = "codeskimr.theme";

/** The colour tokens every theme defines. */
export const COLOR_TOKENS = [
  "bg",
  "surface",
  "line",
  "line-soft",
  "ink",
  "ink-2",
  "muted",
  "faint",
  "accent",
  "accent-bg",
  "accent-hover",
  "accent-fg",
  "good",
  "good-bg",
  "warn",
  "warn-bg",
  "bad",
  "bad-bg",
  "outgoing",
  "highlight",
  "mark",
  "kind-class",
  "kind-interface",
  "kind-record",
  "kind-struct",
  "kind-enum",
  "layer-1",
  "layer-2",
  "layer-3",
  "layer-4",
  "layer-5",
  "layer-6",
] as const;
