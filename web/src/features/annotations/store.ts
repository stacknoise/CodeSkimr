import { createSignal } from "solid-js";
import { ApiError } from "../../shared/api/client";
import type { ErrorCode } from "../../shared/api/error-codes.generated";
import { readSummary, readTexts } from "./api";
import { KIND, TARGET_SEPARATOR } from "./consts";
import { buildIndex, countersOf, EMPTY_INDEX, targetKey, type Index } from "./join";
import type { Annotation, Counters, Target } from "./types";

/** Why a change failed: a registered code, or `null` if the server could not be reached. */
export type ChangeFailure = { code: ErrorCode | null };

const [index, setIndex] = createSignal<Index>(EMPTY_INDEX);
const [texts, setTexts] = createSignal<ReadonlyMap<string, Annotation[]>>(new Map());
const [changes, setChanges] = createSignal(0);
const [available, setAvailable] = createSignal<boolean | null>(null);

/** Which project and snapshot the summary and the texts were read for. */
let summaryFor: string | null = null;
/** The project the counters in the index belong to. */
let summaryProject: number | null = null;
let textsFor: string | null = null;
/** The snapshot the counters were read for, kept so that a change can read them again. */
let currentSnapshot: number | null = null;
/** Counts the reads, so that the answer of an earlier one is dropped. */
let generation = 0;

/** The counters of the project, read once for each snapshot. */
export const annotationIndex = index;
/** The notes and tasks of the project by symbol; empty until `loadTexts` has run. */
export const annotationTexts = texts;
/** Whether the server has annotations: `null` until the first read answered, `false` for a server
 * without the commands and for the static export, which holds no user data. The screens then show
 * no tabs, no column and no switch. */
export const annotationsAvailable = available;
/** Counts the changes made in this browser; what shows annotations reads it to read them again. */
export const annotationChanges = changes;

/** The counters of a symbol, `undefined` if it has no annotation. Reads the index, so a place that
 * calls it in its markup follows the changes. */
export function countersFor(target: Target | null): Counters | undefined {
  return countersOf(index(), target);
}

function keyOf(project: number, snapshot: number): string {
  return [project, snapshot].join(TARGET_SEPARATOR);
}

/** Reads the counters of the project unless they are held for this snapshot already. One call
 * for each page of the summary, whatever the number of symbols that are drawn. */
export async function loadSummary(project: number, snapshot: number | null): Promise<void> {
  if (summaryProject !== project) {
    // Another project: nothing of the old one may show while the new one is read.
    summaryProject = project;
    summaryFor = null;
    textsFor = null;
    setIndex(EMPTY_INDEX);
    setTexts(new Map());
  }
  if (snapshot === null) {
    return;
  }
  currentSnapshot = snapshot;
  const key = keyOf(project, snapshot);
  if (summaryFor === key) {
    return;
  }
  summaryFor = key;
  generation += 1;
  const mine = generation;
  try {
    const rows = await readSummary(project);
    if (mine === generation) {
      setIndex(buildIndex(rows));
      setAvailable(true);
    }
  } catch {
    // An older server without the commands, or none at all: no marks, the screen still works.
    if (mine === generation) {
      summaryFor = null;
      setAvailable(false);
    }
  }
}

/** Groups notes and tasks by the symbol they belong to. */
function byTarget(items: readonly Annotation[]): Map<string, Annotation[]> {
  const grouped = new Map<string, Annotation[]>();
  for (const item of items.filter((one) => one.kind !== KIND.critical)) {
    const key = targetKey(item.target.kind, item.target.name, item.target.parameters);
    grouped.set(key, [...(grouped.get(key) ?? []), item]);
  }
  return grouped;
}

/** Reads the notes and tasks of the whole project for the text under the rows (the switch "Show
 * annotations"), once for each snapshot. */
export async function loadTexts(project: number, snapshot: number | null): Promise<void> {
  if (snapshot === null) {
    return;
  }
  const key = keyOf(project, snapshot);
  if (textsFor === key) {
    return;
  }
  textsFor = key;
  try {
    setTexts(byTarget(await readTexts(project)));
  } catch {
    textsFor = null;
  }
}

/** Reads what is held again, after a change: the counters, and the texts if they were read. */
async function reload(project: number, snapshot: number | null): Promise<void> {
  const hadTexts = textsFor !== null;
  summaryFor = null;
  textsFor = null;
  await Promise.all([
    loadSummary(project, snapshot),
    hadTexts ? loadTexts(project, snapshot) : Promise.resolve(),
  ]);
}

/** Forgets what is held, so that the next screen reads it again (a project that was left). */
export function forgetAnnotations(): void {
  summaryFor = null;
  summaryProject = null;
  textsFor = null;
  setAvailable(null);
  generation += 1;
  setIndex(EMPTY_INDEX);
  setTexts(new Map());
}

/** Makes a change on the server, then reads the counters (and texts) again and tells the panels
 * to read theirs. Returns `null` on success, else why it failed. */
export async function change(
  project: number,
  work: () => Promise<unknown>,
): Promise<ChangeFailure | null> {
  try {
    await work();
  } catch (error) {
    // What the screen shows may be out of date (a mark that was removed elsewhere): read it again.
    await reload(project, currentSnapshot);
    return { code: error instanceof ApiError ? error.code : null };
  }
  await reload(project, currentSnapshot);
  setChanges((count) => count + 1);
  return null;
}
