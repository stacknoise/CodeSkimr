import { createSignal } from "solid-js";
import { ApiError } from "../../shared/api/client";
import { runCommand } from "../../shared/api/commands";
import { DEPTH_ALL, GRAPH_COMMAND, LEVEL } from "./consts";
import type { GraphData, Level } from "./types";

const [data, setData] = createSignal<GraphData | null>(null);
const [failed, setFailed] = createSignal(false);
const [level, setLevel] = createSignal<Level>(LEVEL.layers);
const [depth, setDepth] = createSignal(DEPTH_ALL);
const [focus, setFocus] = createSignal<number | null>(null);
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
/** What the boxes of the map stand for. */
export const graphLevel = level;
/** Chooses what the boxes stand for; the next load asks for it. */
export const setGraphLevel = setLevel;
/** How many segments of a namespace name stay on the namespace level; 0 keeps them all. */
export const graphDepth = depth;
/** Chooses the depth; the next load asks for it. */
export const setGraphDepth = setDepth;
/** The type in the middle of the map on the type level, `null` until one is chosen. */
export const graphFocus = focus;
/** Chooses the type in the middle; the next load asks for it. */
export const setGraphFocus = setFocus;
/** Whether the last load failed. */
export const graphFailed = failed;

/** Loads the map of a project. A failure is kept for the screen; the old map stays. */
export async function loadGraph(project: number): Promise<void> {
  if (level() === LEVEL.types && focus() === null) {
    return;
  }
  const mine = ++latest;
  try {
    const answer = await runCommand<GraphData>(GRAPH_COMMAND, {
      project,
      level: level(),
      depth: depth(),
      focus: focus(),
    });
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

/** Forgets the map; for tests and when the project changes. The type level needs a focus, which
 * is forgotten too, so the level goes back to the layers. */
export function resetGraph(): void {
  latest += 1;
  setData(null);
  setFocus(null);
  setLevel((chosen) => (chosen === LEVEL.types ? LEVEL.layers : chosen));
  setFailed(false);
}
