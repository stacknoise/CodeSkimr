import { createSignal } from "solid-js";
import { ApiError } from "../../shared/api/client";
import { runCommand } from "../../shared/api/commands";
import { DEPTH_ALL, GRAPH_COMMAND, LEVEL, MODULES_COMMAND } from "./consts";
import type { GraphData, Level, ModulesData, ModulesInfo } from "./types";

const [data, setData] = createSignal<GraphData | null>(null);
const [failed, setFailed] = createSignal(false);
const [modules, setModules] = createSignal<ModulesInfo | null>(null);
const [level, setLevel] = createSignal<Level>(LEVEL.layers);
const [depth, setDepth] = createSignal(DEPTH_ALL);
const [focus, setFocus] = createSignal<number | null>(null);
const [namespace, setNamespace] = createSignal<string | null>(null);
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
/** The namespace in the middle of the map on the namespace level, `null` lets the server choose. */
export const graphNamespace = namespace;
/** Chooses the namespace in the middle; the next load asks for it. */
export const setGraphNamespace = setNamespace;
/** The numbers of the level "modules"; `null` on the other levels. */
export const modulesInfo = modules;
/** Whether the last load failed. */
export const graphFailed = failed;

/** The map of one of the levels that `graph.layers` draws. */
async function loadLevel(project: number) {
  const map = await runCommand<GraphData>(GRAPH_COMMAND, {
    project,
    level: level(),
    depth: depth(),
    focus: focus(),
    namespace: namespace(),
  });
  return { map, modules: null };
}

/** The map and the numbers of the level "modules". */
async function loadModules(project: number) {
  const answer = await runCommand<ModulesData>(MODULES_COMMAND, { project, depth: depth() });
  return { map: { ...answer, focus: null }, modules: answer };
}

/** Loads the map of a project. A failure is kept for the screen; the old map stays. */
export async function loadGraph(project: number): Promise<void> {
  if (level() === LEVEL.types && focus() === null) {
    return;
  }
  const mine = ++latest;
  try {
    const picture =
      level() === LEVEL.modules ? await loadModules(project) : await loadLevel(project);
    if (mine === latest) {
      setData(picture.map);
      setModules(picture.modules);
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
  setModules(null);
  setFocus(null);
  setNamespace(null);
  setLevel((chosen) => (chosen === LEVEL.types ? LEVEL.layers : chosen));
  setFailed(false);
}
