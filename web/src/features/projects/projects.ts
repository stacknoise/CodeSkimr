import { createMemo, createRoot, createSignal } from "solid-js";
import { currentLanguage } from "../../shared/i18n/i18n";
import { ApiError } from "../../shared/api/client";
import { runCommand } from "../../shared/api/commands";
import type { ErrorCode } from "../../shared/api/error-codes.generated";
import { navigate } from "../../app/router";
import { PROJECTS_ROUTE } from "../../app/consts";
import { closeProject } from "../../shared/project/active";
import { pollUnanswered } from "../../shared/api/poll";
import { COMMAND, POLL_MS } from "./consts";
import { byName } from "./order";
import { showScan } from "./page";
import { forgetLevel2 } from "./level2";
import { keepRunningOnly, noteLevel2Start, resetLevel2Runs } from "./level2run";
import type { ProjectRow, Scanned, ToolsInfo } from "./types";

/** Why the last action failed: a registered code, or `null` for a failure without one (the
 * server could not be reached). */
export type Failure = { code: ErrorCode | null; retryAfter?: number | null };

const [rows, setRows] = createSignal<ProjectRow[]>([]);
const [loaded, setLoaded] = createSignal(false);
const [listFailure, setListFailure] = createSignal<Failure | null>(null);
const [actionFailure, setActionFailure] = createSignal<Failure | null>(null);
const [scanning, setScanning] = createSignal<ReadonlySet<number>>(new Set());
const [requested, setRequested] = createSignal<ReadonlyMap<number, number>>(new Map());
const [adding, setAdding] = createSignal(false);
const [checkedTools, setCheckedTools] = createSignal<ReadonlyMap<number, ToolsInfo>>(new Map());
const [checking, setChecking] = createSignal<ReadonlySet<number>>(new Set());
const [lastSeconds, setLastSeconds] = createSignal<ReadonlyMap<number, number>>(new Map());

/** The project whose name is being edited and what has been typed so far. It lives here, not in
 * the row: the list is read again while a scan runs and would drop the input of a row. */
const [renaming, setRenaming] = createSignal<{ id: number; text: string } | null>(null);

/** Whether the name is being saved: the store may be busy with a removal for a moment. */
const [savingName, setSavingName] = createSignal(false);

/** The name edit that is open, or `null`. */
export const renamingProject = renaming;
/** Whether the typed name is being saved. */
export const renameSaving = savingName;
/** Opens the name edit of a project with its current name. */
export function startRename(id: number, current: string): void {
  setRenaming({ id, text: current });
}
/** Keeps what the user typed. */
export function typeRename(text: string): void {
  setRenaming((open) => (open === null ? null : { ...open, text }));
}
/** Closes the name edit without saving. */
export function cancelRename(): void {
  setRenaming(null);
}

/** Closes the name edit if it is the one of project `id`. */
export function cancelRenameOf(id: number): void {
  setRenaming((open) => (open?.id === id ? null : open));
}

/** Saves the typed name of the open edit; an empty name goes back to the suggested one. Returns
 * whether it worked; a failure stays on the screen and the edit stays open. */
export async function saveRename(): Promise<boolean> {
  const open = renaming();
  if (open === null || savingName()) {
    return false;
  }
  setActionFailure(null);
  setSavingName(true);
  try {
    await runCommand(COMMAND.rename, { project: open.id, name: open.text });
    setRenaming(null);
    return true;
  } catch (error) {
    setActionFailure(failed(error));
    return false;
  } finally {
    setSavingName(false);
    await refreshProjects();
  }
}

/** The name that a source would get, or `null` if the text is no source (yet) or unanswered. */
export async function suggestName(source: string): Promise<string | null> {
  try {
    const answer = await runCommand<{ name: string | null }>(COMMAND.suggestName, { source });
    return answer.name;
  } catch {
    return null;
  }
}

/** Ids of the projects that existed when the user added a new one; `null` while none is added. */
let knownAtAdd: ReadonlySet<number> | null = null;

/** The registered projects in the order of the list: alphabetical by shown name. */
export const projects = createRoot(() => createMemo(() => byName(rows(), currentLanguage())));
/** Whether the list has been read at least once. */
export const projectsLoaded = loaded;
/** The failure of the last action, else the one of the last read; `null` if both worked. */
export const projectsFailure = () => actionFailure() ?? listFailure();
/** Whether a new project is being added and scanned. */
export const projectAdding = adding;
/** When this tab asked for the scan that it runs for the project, in milliseconds; `null` if it
 * runs none. */
export function scanRequestedAt(project: number): number | null {
  return scanning().has(project) ? (requested().get(project) ?? null) : null;
}
/** Ids of the projects that this tab is scanning again. */
export const projectsScanning = scanning;
/** Ids of the projects whose tools are being checked right now. */
export const toolsChecking = checking;
/** The tools of a project: what this tab checked last, else what the list says; `null` if the
 * project was never checked. */
export function toolsOf(row: ProjectRow): ToolsInfo | null {
  return checkedTools().get(row.id) ?? row.tools ?? null;
}
/** Duration in seconds of the last scan that this tab started, by project id. */
export const scanSeconds = lastSeconds;

/** The failure of a caught error, in the shape the screen shows. */
export function failed(error: unknown): Failure {
  return error instanceof ApiError
    ? { code: error.code, retryAfter: error.retryAfter }
    : { code: null };
}

/** Reads the list again. A failure is kept for the screen; the old rows stay. A `signal` lets
 * the caller give up waiting. */
export async function refreshProjects(signal?: AbortSignal): Promise<void> {
  try {
    const list = await runCommand<ProjectRow[]>(COMMAND.list, undefined, signal);
    setRows(list);
    keepRunningOnly(new Set(list.filter((row) => row.running).map((row) => row.id)));
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
  const stop = pollUnanswered(refreshProjects, POLL_MS);
  try {
    return await work();
  } finally {
    stop();
  }
}

function remember(scanned: Scanned): void {
  noteLevel2Start(scanned.project, scanned.level2);
  setLastSeconds((previous) => new Map(previous).set(scanned.project, scanned.seconds));
  if (scanned.tools.checked_at !== null) {
    // An incremental scan does not check the tools: its empty view must not hide a warning.
    setCheckedTools((previous) => new Map(previous).set(scanned.project, scanned.tools));
  }
}

/** Checks which tools level 2 needs on this machine for the project, so that the screen shows what
 * is there now. A failure keeps what the list says: a missing answer is not a missing tool. */
export async function checkTools(project: number): Promise<void> {
  if (checking().has(project)) {
    return;
  }
  setChecking((previous) => new Set(previous).add(project));
  try {
    const found = await runCommand<ToolsInfo>(COMMAND.tools, { project });
    setCheckedTools((previous) => new Map(previous).set(project, found));
  } catch {
    // The stored result stays on the screen.
  } finally {
    setChecking((previous) => {
      const next = new Set(previous);
      next.delete(project);
      return next;
    });
  }
}

/** Registers a folder or a Git address and scans it. Returns whether it worked. */
export async function addProject(
  source: string,
  branch: string,
  credential?: string,
  name?: string,
): Promise<boolean> {
  setAdding(true);
  setActionFailure(null);
  knownAtAdd = new Set(rows().map((row) => row.id));
  const input = {
    source,
    branch,
    ...(credential ? { credential } : {}),
    ...(name ? { name } : {}),
  };
  try {
    remember(await whileReading(() => runCommand<Scanned>(COMMAND.add, input)));
    return true;
  } catch (error) {
    setActionFailure(failed(error));
    return false;
  } finally {
    setAdding(false);
    knownAtAdd = null;
    await refreshProjects();
  }
}

/** The first report of a scan of a project that did not exist before shows the scan screen: it is
 * the scan that the user just started by adding the project. The project has no id before this. */
export function noteScanReport(project: number): void {
  if (knownAtAdd !== null && !knownAtAdd.has(project)) {
    knownAtAdd = null;
    showScanOf(project);
  }
}

/** Opens the project and shows the tab "Scan" of its page. */
function showScanOf(project: number): void {
  showScan(project);
  navigate(PROJECTS_ROUTE.path);
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

/** Scans a project again, incrementally if it was scanned before. A scan from the browser runs
 * stage 2 after stage 1 where it is confirmed; `level2: false` makes this one scan stage 1 only
 * (ADR 0072, decision 1). `full: true` reads every file again instead of the changes only, as
 * `codeskimr scan` does. With `language`, stage 2 checks only that language and leaves what the
 * report says about the others as it is. */
export async function rescanProject(
  project: number,
  level2 = true,
  full = false,
  language?: string,
): Promise<void> {
  if (scanning().has(project)) {
    return;
  }
  mark(project, true);
  setRequested((previous) => new Map(previous).set(project, Date.now()));
  setActionFailure(null);
  showScanOf(project);
  try {
    remember(
      await whileReading(() =>
        runCommand<Scanned>(COMMAND.rescan, {
          project,
          ...(level2 ? {} : { level2 }),
          ...(full ? { full } : {}),
          ...(language === undefined ? {} : { language }),
        }),
      ),
    );
  } catch (error) {
    setActionFailure(failed(error));
  } finally {
    mark(project, false);
    await refreshProjects();
  }
}

/** Forgets what this tab keeps per project, so a project that gets the id later starts clean. */
function forget(project: number): void {
  const without = <V>(previous: ReadonlyMap<number, V>) => {
    const next = new Map(previous);
    next.delete(project);
    return next;
  };
  setCheckedTools(without);
  setLastSeconds(without);
  forgetLevel2(project);
  resetLevel2Runs(project);
  if (renaming()?.id === project) {
    setRenaming(null);
  }
  closeProject(project);
}

/** Removes a project from the app: its scans and stored results go, its files do not. Returns
 * whether it worked; a failure is kept for the screen. */
export async function removeProject(project: number): Promise<boolean> {
  setActionFailure(null);
  try {
    await runCommand(COMMAND.remove, { project });
    forget(project);
    return true;
  } catch (error) {
    setActionFailure(failed(error));
    return false;
  } finally {
    await refreshProjects();
  }
}
