import { createSignal } from "solid-js";
import { SETTINGS_TAB, type SettingsTab } from "./consts";
import { setEffectiveSearch } from "./effective";

const [tab, setTab] = createSignal<SettingsTab>(SETTINGS_TAB.appearance);

/** The tab of the settings screen that is shown. */
export const settingsTab = tab;

/** Changes the tab that is shown; the text of the search belongs to the effective settings, so
 * it is cleared when another tab is chosen. */
export function showSettingsTab(next: SettingsTab): void {
  if (next !== SETTINGS_TAB.effective) {
    setEffectiveSearch("");
  }
  setTab(next);
}

/** The search of the tab bar: it narrows the effective settings and shows that tab, because the
 * list of effective settings is the one place that holds many values (ADR 0098). */
export function searchSettings(text: string): void {
  setEffectiveSearch(text);
  if (text.trim() !== "") {
    setTab(SETTINGS_TAB.effective);
  }
}
