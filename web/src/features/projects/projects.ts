import { createSignal } from "solid-js";
import { ApiError } from "../../shared/api/client";
import { runCommand } from "../../shared/api/commands";
import type { ErrorCode } from "../../shared/api/error-codes.generated";
import { COMMAND, POLL_MS } from "./consts";
import type { ProjectRow, Scanned } from "./types";

/** Why the last action failed: a registered code, or `null` for a failure without one (the
 * server could not be reached). */
export type Failure = { code: ErrorCode | null };

const [rows, setRows] = createSignal<ProjectRow[]>([]);
const [loaded, setLoaded] = createSignal(false);
const [listFailure, setListFailure] = createSignal<Failure | null>(null);
const [actionFailure, setActionFailure] = createSignal<Failure | null>(null);
const [scanning, setScanning] = createSignal<ReadonlySet<number>>(new Set());
const [adding, setAdding] = createSignal(false);
const [lastSeconds, setLastSeconds] = createSignal<ReadonlyMap<number, number>>(new Map());

/** The registered projects. */
export const projects = rows;
/** Whether the list has been read at least once. */
export const projectsLoaded = loaded;
/** The failure of the last action, else the one of the last read; `null` if both worked. */
export const projectsFailure = () => actionFailure() ?? listFailure();
/** Whether a new project is being added and scanned. */
export const projectAdding = adding;
/** Ids of the projects that this tab is scanning again. */
export const projectsScanning = scanning;
/** Duration in seconds of the last scan that this tab started, by project id. */
export const scanSeconds = lastSeconds;

function failed(error: unknown): Failure {
  return { code: error instanceof ApiError ? error.code : null };
}

/** Reads the list again. A failure is kept for the screen; the old rows stay. */
export async function refreshProjects(): Promise<void> {
  try {
    setRows(await runCommand<ProjectRow[]>(COMMAND.list));
    setListFailure(null);
  } catch (error) {
    setListFailure(failed(error));
  } finally {
    setLoaded(true);
  }
}

/** Runs `work` and reads the list every `POLL_MS` until it is done, so that the progress of a
 * long scan shows up. */
async function whileReading<T>(work: () => Promise<T>): Promise<T> {
  const timer = setInterval(() => void refreshProjects(), POLL_MS);
  try {
    return await work();
  } finally {
    clearInterval(timer);
  }
}

function remember(scanned: Scanned): void {
  setLastSeconds((previous) => new Map(previous).set(scanned.project, scanned.seconds));
}

/** Registers a folder or a Git address and scans it. Returns whether it worked. */
export async function addProject(source: string, branch: string): Promise<boolean> {
  setAdding(true);
  setActionFailure(null);
  try {
    remember(await whileReading(() => runCommand<Scanned>(COMMAND.add, { source, branch })));
    return true;
  } catch (error) {
    setActionFailure(failed(error));
    return false;
  } finally {
    setAdding(false);
    await refreshProjects();
  }
}

function mark(project: number, on: boolean): void {
  setScanning((previous) => {
    const next = new Set(previous);
    if (on) {
      next.add(project);
    } else {
      next.delete(project);
    }
    return next;
  });
}

/** Scans a project again, incrementally if it was scanned before. */
export async function rescanProject(project: number): Promise<void> {
  if (scanning().has(project)) {
    return;
  }
  mark(project, true);
  setActionFailure(null);
  try {
    remember(await whileReading(() => runCommand<Scanned>(COMMAND.rescan, { project })));
  } catch (error) {
    setActionFailure(failed(error));
  } finally {
    mark(project, false);
    await refreshProjects();
  }
}
