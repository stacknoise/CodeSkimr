import { createSignal } from "solid-js";
import { countKind, readKind } from "./api";
import { BOARD, KIND, STATUS, type BoardState } from "./consts";
import { targetKey } from "./join";
import type { Annotation, Target } from "./types";

/** The filters above the columns (mock 16). */
export interface Filters {
  mine: boolean;
  critical: boolean;
  due: boolean;
}

/** All filters off. */
export const NO_FILTERS: Filters = { mine: false, critical: false, due: false };

const [tasks, setTasks] = createSignal<Annotation[]>([]);
const [notes, setNotes] = createSignal(0);
const [marked, setMarked] = createSignal<ReadonlySet<string>>(new Set());
const [state, setState] = createSignal<BoardState>(BOARD.loading);

/** Counts the reads, so that the answer of an earlier one is dropped. */
let generation = 0;

/** The tasks of the project, the number of notes and what the board is doing. */
export const boardTasks = tasks;
export const boardNotes = notes;
export const boardState = state;

function keyOfTarget(target: Target): string {
  return targetKey(target.kind, target.name);
}

/** Reads what the board shows: the tasks, the number of notes and the critical symbols, in three
 * calls (the tasks and the critical marks page by page), never one call for each task. */
export async function loadBoard(project: number): Promise<void> {
  generation += 1;
  const mine = generation;
  try {
    const [taskList, noteCount, marks] = await Promise.all([
      readKind(project, KIND.task),
      countKind(project, KIND.note),
      readKind(project, KIND.critical),
    ]);
    if (mine === generation) {
      setTasks(taskList);
      setNotes(noteCount);
      setMarked(new Set<string>(marks.map((mark) => keyOfTarget(mark.target))));
      setState(BOARD.ready);
    }
  } catch {
    if (mine === generation) {
      setState(BOARD.failed);
    }
  }
}

/** Whether the symbol of a task is marked as critical. */
export function isMarked(target: Target): boolean {
  return marked().has(keyOfTarget(target));
}

/** The tasks that pass the filters. "Mine" are the tasks assigned to `me`, the name the server gave
 * as author of the annotations made here; "Due" are the tasks that are not done and have a date. */
export function filterTasks(
  all: readonly Annotation[],
  filters: Filters,
  mark: (target: Target) => boolean,
  me: string | null,
): Annotation[] {
  return all.filter(
    (task) =>
      (!filters.mine || (me !== null && task.assignee === me)) &&
      (!filters.critical || mark(task.target)) &&
      (!filters.due || (task.due != null && task.status !== STATUS.done)),
  );
}

/** The tasks of a column: the ones with the nearest due date first, then the others by the time
 * they were made. */
export function column(tasksOf: readonly Annotation[], status: string): Annotation[] {
  return tasksOf
    .filter((task) => (task.status ?? STATUS.open) === status)
    .sort((a, b) => (a.due ?? Infinity) - (b.due ?? Infinity) || a.created - b.created);
}

/** The number of tasks that are not done. */
export function openTasks(all: readonly Annotation[]): number {
  return all.filter((task) => task.status !== STATUS.done).length;
}

/** Forgets what is held; for tests. */
export function resetBoard(): void {
  generation += 1;
  setTasks([]);
  setNotes(0);
  setMarked(new Set<string>());
  setState(BOARD.loading);
}
