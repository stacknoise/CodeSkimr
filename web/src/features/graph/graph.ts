import { createSignal } from "solid-js";
import { ApiError } from "../../shared/api/client";
import { runCommand } from "../../shared/api/commands";
import { GRAPH_COMMAND } from "./consts";
import type { GraphData } from "./types";

const [data, setData] = createSignal<GraphData | null>(null);
const [failed, setFailed] = createSignal(false);
/** Counts the loads, so that a slow answer for a project that is no longer shown is dropped. */
let latest = 0;

const [wanted, setWanted] = createSignal<string | null>(null);

/** The layer a screen asked to have selected on the map, until the map takes it. */
export const wantedLayer = wanted;

/** Asks the map to select `layer` when it is shown; a name the map does not know is dropped. */
export function wantLayer(layer: string | null): void {
  setWanted(layer);
}

/** The layers of the project that was loaded last, `null` until the first answer. */
export const graphData = data;
/** Whether the last load failed. */
export const graphFailed = failed;

/** Loads the map of a project. A failure is kept for the screen; the old map stays. */
export async function loadGraph(project: number): Promise<void> {
  const mine = ++latest;
  try {
    const answer = await runCommand<GraphData>(GRAPH_COMMAND, { project });
    if (mine === latest) {
      setData(answer);
      setFailed(false);
    }
  } catch (error) {
    if (mine === latest) {
      setFailed(error instanceof ApiError || error instanceof TypeError);
    }
  }
}

/** Forgets the map; for tests and when the project changes. */
export function resetGraph(): void {
  latest += 1;
  setData(null);
  setFailed(false);
}
