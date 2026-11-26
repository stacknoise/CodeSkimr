import { createSignal } from "solid-js";
import { runCommand } from "../../shared/api/commands";
import { failed, type Failure } from "../projects/projects";
import { RENAME_COMMAND } from "./consts";
import type { Preview } from "./types";

/** The symbol whose rename is being checked. */
export interface RenameTarget {
  project: number;
  symbol: number;
  name: string;
}

const [target, setTarget] = createSignal<RenameTarget | null>(null);
const [preview, setPreview] = createSignal<Preview | null>(null);
const [failure, setFailure] = createSignal<Failure | null>(null);
const [checking, setChecking] = createSignal(false);
let generation = 0;

/** The symbol of the open dialog; `null` while the dialog is closed. */
export const renameTarget = target;
/** The answer for the name that was asked last; `null` before the first check. */
export const renamePreview = preview;
/** Why the last check failed, `null` if it worked or none ran. */
export const renameFailure = failure;
/** True while a check runs. */
export const renameChecking = checking;

/** Opens the dialog for a symbol, with no answer yet. */
export function openRename(next: RenameTarget): void {
  generation += 1;
  setPreview(null);
  setFailure(null);
  setChecking(false);
  setTarget(next);
}

/** Closes the dialog and forgets its answer. */
export function closeRename(): void {
  generation += 1;
  setTarget(null);
  setPreview(null);
  setFailure(null);
  setChecking(false);
}

/** Asks what the new name would hit. Nothing is changed. An answer that comes after the dialog was
 * closed or another check started is dropped. */
export async function checkRename(newName: string): Promise<void> {
  const asked = target();
  if (asked === null) {
    return;
  }
  generation += 1;
  const mine = generation;
  setChecking(true);
  setFailure(null);
  try {
    const answer = await runCommand<Preview>(RENAME_COMMAND, {
      project: asked.project,
      symbol: asked.symbol,
      new_name: newName,
    });
    if (mine === generation) {
      setPreview(answer);
    }
  } catch (error) {
    if (mine === generation) {
      setPreview(null);
      setFailure(failed(error));
    }
  } finally {
    if (mine === generation) {
      setChecking(false);
    }
  }
}
