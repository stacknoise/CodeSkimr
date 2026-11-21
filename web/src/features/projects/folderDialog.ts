import { createSignal } from "solid-js";
import { runCommand } from "../../shared/api/commands";
import { FOLDER_DIALOG_COMMAND } from "./consts";

const [available, setAvailable] = createSignal(false);
const [waiting, setWaiting] = createSignal(false);

/** Counts the dialogs asked; the answer of one that was given up is dropped. */
let generation = 0;

/** Whether the app can show the folder dialog of the operating system on its machine. */
export const folderDialogAvailable = available;
/** Whether a dialog is open and the form waits for its answer. */
export const folderDialogWaiting = waiting;

/** Asks the app once whether a folder dialog exists here. A failed call means no button, never an
 * error message: typing the path always works. */
export async function readFolderDialog(): Promise<void> {
  try {
    const info = await runCommand<{ available: boolean }>(FOLDER_DIALOG_COMMAND.info);
    setAvailable(info.available);
  } catch {
    setAvailable(false);
  }
}

/** Opens the dialog and waits; the chosen path, or `null` if the user cancelled, gave up waiting
 * or the dialog could not open. */
export async function pickFolder(): Promise<string | null> {
  generation += 1;
  const mine = generation;
  setWaiting(true);
  try {
    const picked = await runCommand<{ path: string | null }>(FOLDER_DIALOG_COMMAND.pick);
    return mine === generation ? picked.path : null;
  } catch {
    return null;
  } finally {
    if (mine === generation) {
      setWaiting(false);
    }
  }
}

/** Stops waiting for the dialog. The window of the system stays until it is closed; its answer is
 * dropped. */
export function giveUpWaiting(): void {
  generation += 1;
  setWaiting(false);
}
