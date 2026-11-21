import { createSignal } from "solid-js";
import { activeProject } from "../../shared/project/active";
import { change } from "../annotations/store";
import { dismissAlert, dismissAllAlerts, readAlerts, readOpenCount } from "./api";
import {
  ALERTS_EVENT,
  LOADING,
  REMOVED_KINDS,
  STATE,
  TAB,
  type AlertKind,
  type Loading,
  type TabId,
} from "./consts";
import type { Alert } from "./types";

const [count, setCount] = createSignal<number | null>(null);
const [shown, setShown] = createSignal(false);
const [tab, setTab] = createSignal<TabId>(TAB.open);
const [open, setOpen] = createSignal<Alert[]>([]);
const [done, setDone] = createSignal<Alert[]>([]);
const [loading, setLoading] = createSignal<Loading>(LOADING.idle);
const [failed, setFailed] = createSignal(false);

/** Counts the reads, so that the answer of an earlier one is dropped. */
let generation = 0;

/** The number on the bell; `null` while it is unknown and for a server without alerts. */
export const openAlertCount = count;
/** Whether the drawer is shown. */
export const drawerShown = shown;
/** The tab of the drawer that is shown. */
export const activeTab = tab;
export const selectTab = setTab;
/** The open alerts and the dismissed ones. */
export const openAlerts = open;
export const doneAlerts = done;
export const alertsLoading = loading;
/** Whether the last dismissal or reassignment failed. */
export const actionFailed = failed;
export const clearActionFailed = () => setFailed(false);

/** Reads the number of open alerts of the project, for the bell. A server without alerts, or the
 * static export, leaves it `null`: the bell is not shown then. */
export async function refreshCount(project: number | null): Promise<void> {
  if (project === null) {
    setCount(null);
    return;
  }
  try {
    const value = await readOpenCount(project);
    if (project === activeProject()) {
      setCount(value);
    }
  } catch {
    setCount(null);
  }
}

/** Reads both lists of the drawer. */
export async function loadAlerts(project: number): Promise<void> {
  generation += 1;
  const mine = generation;
  setLoading((state) => (state === LOADING.ready ? state : LOADING.loading));
  try {
    const [opened, dismissed] = await Promise.all([
      readAlerts(project, STATE.open),
      readAlerts(project, STATE.dismissed),
    ]);
    if (mine === generation) {
      setOpen(opened.items);
      setDone(dismissed.items);
      setCount(opened.open);
      setLoading(LOADING.ready);
    }
  } catch {
    if (mine === generation) {
      setLoading(LOADING.failed);
    }
  }
}

/** Shows the drawer and reads the lists. */
export function showDrawer(project: number): void {
  setShown(true);
  void loadAlerts(project);
}

/** Hides the drawer. */
export function hideDrawer(): void {
  setShown(false);
}

/** The alerts of a tab. "Removed" holds the orphaned targets, too. */
export function alertsOfTab(id: TabId, openList: Alert[], doneList: Alert[]): Alert[] {
  if (id === TAB.done) {
    return doneList;
  }
  if (id === TAB.open) {
    return openList;
  }
  const kinds: readonly AlertKind[] = id === TAB.removed ? REMOVED_KINDS : [id];
  return openList.filter((alert) => kinds.includes(alert.kind));
}

/** What the server told through the event: the number of open alerts of a project. The drawer
 * reads its lists again, because the event carries no text. */
export function applyAlertsEvent(project: number, openNow: number): void {
  if (project !== activeProject()) {
    return;
  }
  setCount(openNow);
  if (shown()) {
    void loadAlerts(project);
  }
}

/** Runs a change of alerts, then reads the lists again; remembers a failure for the drawer. */
export async function changeAlerts(project: number, work: () => Promise<unknown>): Promise<void> {
  setFailed(false);
  try {
    await work();
  } catch {
    setFailed(true);
  }
  await loadAlerts(project);
}

/** Changes the critical mark an alert is about (reassigns or removes it): the counters of the
 * explorer are read again, then the lists of the drawer. */
export async function changeMark(project: number, work: () => Promise<unknown>): Promise<void> {
  setFailed((await change(project, work)) !== null);
  await loadAlerts(project);
}

/** Dismisses one alert. */
export function dismiss(project: number, alert: number): Promise<void> {
  return changeAlerts(project, () => dismissAlert(project, alert));
}

/** Dismisses every open alert of the kinds of a tab. */
export function dismissKinds(project: number, kinds: readonly string[]): Promise<void> {
  return changeAlerts(project, () => dismissAllAlerts(project, kinds));
}

/** The event of the stream that this module knows: `alerts_changed` with the project and the
 * number of open alerts. Anything else is `null`. */
export function parseAlertsEvent(text: string): { project: number; open: number } | null {
  let value: unknown;
  try {
    value = JSON.parse(text);
  } catch {
    return null;
  }
  const event = value as { type?: unknown; project?: unknown; open?: unknown } | null;
  if (
    event?.type === ALERTS_EVENT &&
    typeof event.project === "number" &&
    typeof event.open === "number"
  ) {
    return { project: event.project, open: event.open };
  }
  return null;
}

/** Forgets everything; for tests and a project that was left. */
export function resetAlerts(): void {
  generation += 1;
  setCount(null);
  setShown(false);
  setTab(TAB.open);
  setOpen([]);
  setDone([]);
  setLoading(LOADING.idle);
  setFailed(false);
}
