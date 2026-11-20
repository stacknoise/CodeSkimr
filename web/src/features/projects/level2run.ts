import { createSignal } from "solid-js";
import type { Level2Start } from "./types";

/** What a running level 2 pass is doing, from the event stream (ADR 0072, decision 6). */
export interface Level2Run {
  language: string;
  /** `level2_running` or `level2_importing`. */
  stage: string;
}

/** How a level 2 pass ended: counts of languages. */
export type Level2Done = { checked: number; skipped: number; failed: number };

const [runs, setRuns] = createSignal<ReadonlyMap<number, Level2Run>>(new Map());
const [starts, setStarts] = createSignal<ReadonlyMap<number, Level2Start>>(new Map());
const [ended, setEnded] = createSignal<ReadonlyMap<number, Level2Done>>(new Map());

function put<V>(previous: ReadonlyMap<number, V>, project: number, value: V): Map<number, V> {
  return new Map(previous).set(project, value);
}

function without<V>(previous: ReadonlyMap<number, V>, project: number): Map<number, V> {
  const next = new Map(previous);
  next.delete(project);
  return next;
}

/** The level 2 pass that is running for the project, `null` if none is known to run. */
export function level2RunOf(project: number): Level2Run | null {
  return runs().get(project) ?? null;
}

/** What the last scan from this tab said about stage 2, `null` if nothing is worth saying. */
export function level2StartOf(project: number): Level2Start | null {
  return starts().get(project) ?? null;
}

/** How the last pass that this tab saw ended. */
export function level2DoneOf(project: number): Level2Done | null {
  return ended().get(project) ?? null;
}

/** Remembers the answer of a scan: a new scan clears the old messages. */
export function noteLevel2Start(project: number, start: Level2Start | undefined): void {
  setEnded((previous) => without(previous, project));
  setStarts((previous) =>
    start === undefined ? without(previous, project) : put(previous, project, start),
  );
}

/** An event `level2_progressed`. */
export function noteLevel2Progress(project: number, run: Level2Run): void {
  setRuns((previous) => put(previous, project, run));
  setEnded((previous) => without(previous, project));
}

/** An event `level2_finished`. */
export function noteLevel2Finished(project: number, done: Level2Done): void {
  setRuns((previous) => without(previous, project));
  setEnded((previous) => put(previous, project, done));
}

/** Drops the runs of projects that the list no longer calls running: a missed event must not leave
 * a banner on the screen. */
export function keepRunningOnly(running: ReadonlySet<number>): void {
  setRuns((previous) => {
    const stale = [...previous.keys()].filter((project) => !running.has(project));
    return stale.length === 0 ? previous : stale.reduce(without, previous);
  });
}

/** Forgets everything about a project, or about all (for tests). */
export function resetLevel2Runs(project?: number): void {
  if (project === undefined) {
    setRuns(new Map());
    setStarts(new Map());
    setEnded(new Map());
    return;
  }
  setRuns((previous) => without(previous, project));
  setStarts((previous) => without(previous, project));
  setEnded((previous) => without(previous, project));
}
