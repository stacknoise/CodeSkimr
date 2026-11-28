import { createSignal } from "solid-js";
import { ApiError } from "../../shared/api/client";
import { runCommand } from "../../shared/api/commands";
import type { ErrorCode } from "../../shared/api/error-codes.generated";
import { COMMAND, type DIALOG } from "./consts";
import { loadRules } from "./rules";
import type { LayersOut, RuleOut, RulesInput, RulesOut, SavedOut } from "./types";

/** What the open dialog changes: a kind of rule, the entry it starts from (`null` for a new
 * one) and, for deleting, the rule itself. */
export interface DialogTarget {
  kind: (typeof DIALOG)[keyof typeof DIALOG];
  index: number | null;
  rule: RuleOut | null;
}

const [target, setTarget] = createSignal<DialogTarget | null>(null);
const [namesIn, setNames] = createSignal<LayersOut>({ layers: [], namespaces: [] });
const [saveFailure, setSaveFailure] = createSignal<ErrorCode | null | undefined>(undefined);

/** The dialog that is open, `null` if none is. */
export const dialogTarget = target;
/** The layers and namespaces of the project the dialogs offer. */
export const editorNames = namesIn;
/** The code of the failed save; `null` for a failure without a code, `undefined` if it did not
 * fail. */
export const saveFailed = saveFailure;

/** Opens a dialog once the layers and namespaces are read for it; a failed read leaves the lists
 * empty, because the fields take typed prefixes as well. */
export async function openDialog(project: number, next: DialogTarget): Promise<void> {
  setSaveFailure(undefined);
  try {
    setNames(await runCommand<LayersOut>(COMMAND.layers, { project }));
  } catch {
    setNames({ layers: [], namespaces: [] });
  }
  setTarget(next);
}

/** Closes the dialog and forgets its failure. */
export function closeDialog(): void {
  setTarget(null);
  setSaveFailure(undefined);
}

/** Judges `input` over the newest scan without writing it; `null` if the rules are not valid. */
export async function previewRules(input: RulesInput): Promise<RulesOut | null> {
  try {
    return await runCommand<RulesOut>(COMMAND.preview, input);
  } catch {
    return null;
  }
}

/** Writes `input` into the project file and reads the rules again. Returns whether it worked;
 * the dialog stays open with the failure if it did not. */
export async function saveRules(input: RulesInput): Promise<boolean> {
  setSaveFailure(undefined);
  try {
    await runCommand<SavedOut>(COMMAND.set, input);
  } catch (error) {
    setSaveFailure(error instanceof ApiError ? error.code : null);
    return false;
  }
  closeDialog();
  await loadRules(input.project);
  return true;
}
