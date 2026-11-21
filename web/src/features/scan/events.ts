import { createSignal } from "solid-js";
import { applyAlertsEvent, parseAlertsEvent } from "../alerts/store";
import { noteLevel2Finished, noteLevel2Progress } from "../projects/level2run";
import { noteScanReport, refreshProjects } from "../projects/projects";
import {
  EVENT,
  EVENTS_PATH,
  LEVEL2_EVENT,
  RECONNECT_MS,
  SCHEME_SEPARATOR,
  SOCKET_SCHEME,
  SOCKET_SCHEME_DEFAULT,
  STAGES,
} from "./consts";
import { applyEvent } from "./run";
import type { Run, Stage, WireEvent } from "./types";

const [runs, setRuns] = createSignal<Run[]>([]);
const [clock, setClock] = createSignal(Date.now());

/** When the last event arrived, in milliseconds: the "now" of a running scan, so that its
 * numbers change when something happened and not on a timer. */
export const lastEventAt = clock;

/** The scans that the stream told of since this page was opened, oldest first. */
export const scanRuns = runs;

function isStage(value: unknown): value is Stage {
  return STAGES.some((stage) => stage === value);
}

/** The event in `text`, or `null` if it is not one this screen knows. Anything else on the stream
 * is ignored: a newer server may send more kinds. */
export function parseEvent(text: string): WireEvent | null {
  let value: unknown;
  try {
    value = JSON.parse(text);
  } catch {
    return null;
  }
  const event = value as Partial<WireEvent> | null;
  if (typeof event?.project !== "number") {
    return null;
  }
  if (event.type === EVENT.progressed) {
    return isStage(event.stage) ? (event as WireEvent) : null;
  }
  const known: readonly string[] = [EVENT.created, EVENT.stopped, EVENT.changed];
  return typeof event.type === "string" && known.includes(event.type) ? (event as WireEvent) : null;
}

/** Level 2 events carry a project and counts or labels; anything else is not one. */
function applyLevel2Event(text: string): boolean {
  let value: unknown;
  try {
    value = JSON.parse(text);
  } catch {
    return false;
  }
  const event = value as Record<string, unknown> | null;
  if (typeof event?.project !== "number") {
    return false;
  }
  if (event.type === LEVEL2_EVENT.progressed) {
    noteLevel2Progress(event.project, {
      language: String(event.language ?? ""),
      stage: String(event.stage ?? ""),
    });
    return true;
  }
  if (event.type === LEVEL2_EVENT.finished) {
    noteLevel2Finished(event.project, {
      checked: Number(event.checked ?? 0),
      skipped: Number(event.skipped ?? 0),
      failed: Number(event.failed ?? 0),
    });
    void refreshProjects();
    return true;
  }
  return false;
}

function handle(text: string): void {
  if (applyLevel2Event(text)) {
    return;
  }
  const alerts = parseAlertsEvent(text);
  if (alerts !== null) {
    applyAlertsEvent(alerts.project, alerts.open);
    return;
  }
  const event = parseEvent(text);
  if (event === null) {
    return;
  }
  if (event.type === EVENT.progressed) {
    noteScanReport(event.project);
  }
  const now = Date.now();
  setRuns((previous) => applyEvent(previous, event, now));
  setClock(now);
  if (event.type === EVENT.created || event.type === EVENT.stopped) {
    void refreshProjects();
  }
}

function address(): string {
  const { protocol, host } = window.location;
  return `${SOCKET_SCHEME[protocol] ?? SOCKET_SCHEME_DEFAULT}${SCHEME_SEPARATOR}${host}${EVENTS_PATH}`;
}

/** Listens to the event stream until the returned function is called. A lost connection is
 * opened again after a pause: a timer that only runs while the stream is down. */
export function listenToScans(): () => void {
  let socket: WebSocket | null = null;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let stopped = false;
  const open = () => {
    socket = new WebSocket(address());
    socket.onmessage = (message: MessageEvent) => {
      if (typeof message.data === "string") {
        handle(message.data);
      }
    };
    socket.onclose = () => {
      socket = null;
      if (!stopped) {
        timer = setTimeout(open, RECONNECT_MS);
      }
    };
  };
  open();
  return () => {
    stopped = true;
    clearTimeout(timer);
    socket?.close();
  };
}

/** Forgets every run; for tests. */
export function resetRuns(): void {
  setRuns([]);
}
