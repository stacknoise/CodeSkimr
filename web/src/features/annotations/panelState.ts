import { createEffect, createSignal } from "solid-js";
import { errorText, t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { deleteAnnotation, markCritical, readSymbol } from "./api";
import { KIND, TAB, type TabId } from "./consts";
import { countersOf } from "./join";
import { annotationIndex, change, type ChangeFailure } from "./store";
import type { Target } from "./types";

/** What the line under the tabs says: a hint that an action worked, or why it did not. */
export interface Notice {
  text: string;
  failed: boolean;
}

/** Which form of the panel a key asks to open. */
export type Composer = typeof KIND.note | typeof KIND.task;

const [tab, setTab] = createSignal<TabId>(TAB.info);
const [composer, setComposer] = createSignal<Composer | null>(null);
const [notice, setNotice] = createSignal<Notice | null>(null);

/** The tab of the side panel that is shown. It stays when another symbol is selected. */
export const activeTab = tab;
export const selectTab = setTab;
/** The line under the tabs. */
export const panelNotice = notice;
export const clearNotice = () => setNotice(null);

/** The text of a failed change: the message of the registered code, else a general one. */
export function failureText(failure: ChangeFailure): string {
  return failure.code === null ? t(I18N_KEYS.ANNOTATIONS_SAVE_FAILED) : errorText(failure.code);
}

/** Shows the result of a change: the failure, or the hint `done`. */
export function report(failure: ChangeFailure | null, done: string | null): void {
  if (failure !== null) {
    setNotice({ text: failureText(failure), failed: true });
  } else {
    setNotice(done === null ? null : { text: done, failed: false });
  }
}

/** Marks the symbol as critical, with a reason if one is given. */
export async function markAsCritical(
  project: number,
  target: Target,
  reason: string,
): Promise<ChangeFailure | null> {
  return change(project, () => markCritical(project, target, reason));
}

/** Removes the critical mark: finds the annotation of the symbol and deletes it. */
export async function removeCriticalMark(
  project: number,
  target: Target,
): Promise<ChangeFailure | null> {
  let mark: number | undefined;
  try {
    mark = (await readSymbol(project, target)).find((one) => one.kind === KIND.critical)?.id;
  } catch {
    return { code: null };
  }
  return mark === undefined ? null : change(project, () => deleteAnnotation(project, mark));
}

/** The key `K`: marks the symbol, or takes the mark off if it has one. */
export async function toggleCritical(project: number, target: Target): Promise<void> {
  const marked = countersOf(annotationIndex(), target)?.critical ?? false;
  if (marked) {
    report(await removeCriticalMark(project, target), t(I18N_KEYS.ANNOTATIONS_NOTICE_UNMARKED));
  } else {
    report(await markAsCritical(project, target, ""), t(I18N_KEYS.ANNOTATIONS_NOTICE_MARKED));
  }
}

/** The keys `N` and `T`: opens the tab and its form. */
export function compose(kind: Composer): void {
  setNotice(null);
  setTab(kind === KIND.note ? TAB.notes : TAB.tasks);
  setComposer(kind);
}

/** Runs `open` when a key asked for the form of this `kind`, and takes the request. */
export function useRequestedForm(kind: Composer, open: () => void): void {
  createEffect(() => {
    if (composer() === kind) {
      open();
      setComposer(null);
    }
  });
}
