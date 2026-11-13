import { createSignal } from "solid-js";
import { readSetting, writeSetting } from "../settings/storage";
import { DARK_QUERY, THEME_ATTRIBUTE } from "./consts";
import { DARK_THEME, DEFAULT_THEME, THEMES, THEME_STORAGE_KEY, type ThemeName } from "./tokens";

/** Whether `value` names a theme. */
export function isTheme(value: string | null): value is ThemeName {
  return THEMES.some((name) => name === value);
}

const [chosen, setChosen] = createSignal<ThemeName | null>(readChoice());
const [defaultTheme, setDefaultTheme] = createSignal<ThemeName>(DEFAULT_THEME);

function readChoice(): ThemeName | null {
  const stored = readSetting(THEME_STORAGE_KEY);
  return isTheme(stored) ? stored : null;
}

function systemIsDark(): boolean {
  return typeof window.matchMedia === "function" && window.matchMedia(DARK_QUERY).matches;
}

/** The theme the user chose, or `null` while the system setting decides. */
export const chosenTheme = chosen;

/** The theme that is shown: the choice, else the dark theme on a dark system, else the default. */
export function activeTheme(): ThemeName {
  return chosen() ?? (systemIsDark() ? DARK_THEME : defaultTheme());
}

/** Puts the active theme on `<html>`, as the script `theme-init.js` does before the first paint. */
export function applyTheme(): void {
  document.documentElement.setAttribute(THEME_ATTRIBUTE, activeTheme());
}

/** Stores the choice, mirrors it for `theme-init.js` and applies it at once. */
export function selectTheme(name: ThemeName): void {
  writeSetting(THEME_STORAGE_KEY, name);
  setChosen(name);
  applyTheme();
}

/** Forgets the choice: the system setting decides again. */
export function followSystem(): void {
  writeSetting(THEME_STORAGE_KEY, null);
  setChosen(null);
  applyTheme();
}

/** The theme for users who have not chosen one, from the setting `ui.default_theme`. A name
 * that is no theme is ignored. */
export function useDefaultTheme(name: string): void {
  if (isTheme(name)) {
    setDefaultTheme(name);
    applyTheme();
  }
}
