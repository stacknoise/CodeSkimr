import { createSignal } from "solid-js";
import { runCommand } from "../../shared/api/commands";
import { SET_INDEXER_MEMORY_COMMAND, UI_SETTINGS_COMMAND } from "./consts";

/** What the UI has saved, as `ui_settings.get` answers it. */
export interface UiSettings {
  /** Memory limit in MB per indexer; an indexer without an entry has no saved value. */
  indexer_memory_mb: Record<string, number>;
}

const [saved, setSaved] = createSignal<UiSettings | null>(null);

/** The saved settings as the last call got them; `null` before the first answer. */
export const uiSettings = saved;

/** Asks the server for the saved settings. A failure keeps what the screen shows. */
export async function loadUiSettings(): Promise<void> {
  try {
    setSaved(await runCommand<UiSettings>(UI_SETTINGS_COMMAND));
  } catch {
    // The panel then shows the values in effect only.
  }
}

/** Saves the limit of one indexer, or forgets it with `null`.
 *
 * @throws ApiError if the server refuses the value or cannot write it.
 */
export async function saveIndexerMemory(indexer: string, megabytes: number | null): Promise<void> {
  setSaved(await runCommand<UiSettings>(SET_INDEXER_MEMORY_COMMAND, { indexer, megabytes }));
}
