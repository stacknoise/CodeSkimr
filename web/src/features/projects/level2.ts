import { createSignal } from "solid-js";
import { api } from "../../shared/api/client";
import { runCommand } from "../../shared/api/commands";
import { METHOD_POST } from "../../shared/api/consts";
import { uiConfig } from "../../shared/settings/ui-config";
import {
  CONFIRM_COMMAND_PREFIX,
  CONFIRM_WINDOW_PATH,
  ESCAPE_KEY,
  LAUNCH,
  LEVEL2_COMMAND,
  LEVEL2_PANEL_ID_PREFIX,
  LEVEL2_ERROR,
  LEVEL2_PHASE,
  LEVEL2_POLL_MAX_SECONDS,
  LEVEL2_POLL_SECONDS,
  LEVEL2_STATE,
  LEVEL2_SWITCH_ID_PREFIX,
  MS_PER_SECOND,
} from "./consts";
import { failed, type Failure } from "./projects";
import type { ConfirmWindow, Level2Status } from "./types";

/** Everything the screen knows about the level 2 switch of one project. */
export interface Level2View {
  status: Level2Status | null;
  phase: (typeof LEVEL2_PHASE)[keyof typeof LEVEL2_PHASE];
  /** The command to copy, if the confirmation has to be started in a terminal of the user. */
  command: string | null;
  /** When the console window was opened, in milliseconds since the Unix epoch. */
  openedAt: number | null;
  failure: Failure | null;
}

const EMPTY: Level2View = {
  status: null,
  phase: LEVEL2_PHASE.idle,
  command: null,
  openedAt: null,
  failure: null,
};

const [views, setViews] = createSignal<ReadonlyMap<number, Level2View>>(new Map());
const timers = new Map<number, ReturnType<typeof setInterval>>();
const deadlines = new Map<number, ReturnType<typeof setTimeout>>();

/** Forgets everything about the switches and stops all waiting (a new session, or a test). */
export function resetLevel2(): void {
  for (const project of [...timers.keys()]) {
    stopPolling(project);
  }
  setViews(new Map());
}

/** Forgets what is known about the switch of a project and stops waiting for it, for a project
 * that was removed: a new project may get its id. */
export function forgetLevel2(project: number): void {
  stopPolling(project);
  setViews((previous) => {
    const next = new Map(previous);
    next.delete(project);
    return next;
  });
}

/** What the screen knows about the switch of a project. */
export function level2Of(project: number): Level2View {
  return views().get(project) ?? EMPTY;
}

function patch(project: number, change: Partial<Level2View>): void {
  setViews((previous) => new Map(previous).set(project, { ...level2Of(project), ...change }));
}

/** The command that confirms level 2 for a project in a terminal. */
export function confirmCommand(project: number): string {
  return `${CONFIRM_COMMAND_PREFIX}${project}`;
}

/** Asks the server for the state of the switch. A failure keeps the state that was shown. */
export async function loadLevel2(project: number): Promise<Level2Status | null> {
  try {
    const status = await runCommand<Level2Status>(LEVEL2_COMMAND.status, { project });
    patch(project, { status });
    return status;
  } catch {
    return null;
  }
}

function stopPolling(project: number): void {
  clearInterval(timers.get(project));
  timers.delete(project);
  clearTimeout(deadlines.get(project));
  deadlines.delete(project);
}

function pollSetting(value: number | undefined, fallback: number): number {
  return (value ?? fallback) * MS_PER_SECOND;
}

/** Asks for the state until the confirmation arrived, or until the time is up. The page never
 * learns the result in any other way: the consent is typed in the console, not here. The end of
 * the time is its own timer, so that a request that never answers cannot keep the wait alive. */
function watchConfirmation(project: number): void {
  stopPolling(project);
  const config = uiConfig();
  const every = pollSetting(config?.level2_poll_seconds, LEVEL2_POLL_SECONDS);
  const limit = pollSetting(config?.level2_poll_max_seconds, LEVEL2_POLL_MAX_SECONDS);
  timers.set(
    project,
    setInterval(() => {
      void loadLevel2(project).then((status) => {
        if (status?.state === LEVEL2_STATE.on) {
          stopPolling(project);
          patch(project, { phase: LEVEL2_PHASE.idle, command: null, openedAt: null });
        }
      });
    }, every),
  );
  deadlines.set(
    project,
    setTimeout(() => {
      stopPolling(project);
      const lost = focusWouldBeLost(project);
      patch(project, { phase: LEVEL2_PHASE.timedOut });
      if (lost) {
        focusSwitch(project);
      }
    }, limit),
  );
}

/** Puts the focus on the switch of the project, so that it is not lost when a notice goes. */
export function focusSwitch(project: number): void {
  document.getElementById(`${LEVEL2_SWITCH_ID_PREFIX}${project}`)?.focus();
}

/** Whether the focus is nowhere (on the page) or inside the level 2 section of the project, which
 * is where it would be lost when the notice goes. A person typing elsewhere keeps the focus. */
function focusWouldBeLost(project: number): boolean {
  const active = document.activeElement;
  const panel = document.getElementById(`${LEVEL2_PANEL_ID_PREFIX}${project}`);
  return active === null || active === document.body || (panel?.contains(active) ?? false);
}

/** Ends the waiting for the console window: nothing was confirmed, the switch is free again. The
 * page cannot see that the window was closed, so this is the way out for the user. */
export function cancelWaiting(project: number): void {
  stopPolling(project);
  patch(project, { phase: LEVEL2_PHASE.cancelled, command: null, openedAt: null, failure: null });
  focusSwitch(project);
}

/** Escape ends the waiting, wherever the focus is (on the switch or in the section). */
export function escapeWaiting(project: number, event: KeyboardEvent): void {
  if (event.key === ESCAPE_KEY && level2Of(project).phase === LEVEL2_PHASE.waiting) {
    event.stopPropagation();
    cancelWaiting(project);
  }
}

/** Shows the command to copy; `failure` says why the window did not open, if it was tried. */
function showCommand(project: number, command: string | null, failure: Failure | null): void {
  patch(project, {
    phase: LEVEL2_PHASE.waiting,
    command: command ?? confirmCommand(project),
    openedAt: null,
    failure,
  });
  watchConfirmation(project);
}

function onOpened(project: number, opened: ConfirmWindow): void {
  if (opened.launch === LAUNCH.copy) {
    showCommand(project, opened.command ?? null, null);
    return;
  }
  patch(project, { phase: LEVEL2_PHASE.waiting, command: null, openedAt: Date.now() });
  watchConfirmation(project);
}

/** What a refused request leaves on the screen. */
function onOpenFailed(project: number, failure: Failure): void {
  const code = failure.code;
  if (code === LEVEL2_ERROR.windowFailed) {
    showCommand(project, null, failure);
  } else if (code === LEVEL2_ERROR.rateLimited) {
    patch(project, { phase: LEVEL2_PHASE.idle, failure });
  } else {
    patch(project, { phase: LEVEL2_PHASE.idle, failure });
    if (code === LEVEL2_ERROR.switchOff) {
      void loadLevel2(project);
    }
  }
}

/** Asks the server to open the console window of the confirmation. The click gives no consent: the
 * owner types the project name in the window (ADR 0071, decision 1). */
export async function switchOn(project: number): Promise<void> {
  if (level2Of(project).phase === LEVEL2_PHASE.opening) {
    return;
  }
  patch(project, { phase: LEVEL2_PHASE.opening, failure: null });
  try {
    const opened = await api<ConfirmWindow>(CONFIRM_WINDOW_PATH, {
      method: METHOD_POST,
      body: { project_id: project },
    });
    onOpened(project, opened);
  } catch (error) {
    onOpenFailed(project, failed(error));
  }
}

/** Asks whether the owner really wants to switch off. */
export function askRevoke(project: number): void {
  patch(project, { phase: LEVEL2_PHASE.askRevoke, failure: null });
}

/** Leaves the question without switching off. */
export function cancelRevoke(project: number): void {
  patch(project, { phase: LEVEL2_PHASE.idle });
}

/** Takes the confirmation back. Results that were checked stay until the next scan. */
export async function switchOff(project: number): Promise<void> {
  patch(project, { phase: LEVEL2_PHASE.revoking, failure: null });
  stopPolling(project);
  try {
    await runCommand(LEVEL2_COMMAND.revoke, { project });
    patch(project, { phase: LEVEL2_PHASE.revoked });
    await loadLevel2(project);
  } catch (error) {
    patch(project, { phase: LEVEL2_PHASE.idle, failure: failed(error) });
  }
}

/** Forgets the notice after a switch-off (the user went on, for instance by scanning again). */
export function dismissRevoked(project: number): void {
  patch(project, { phase: LEVEL2_PHASE.idle });
}
