import { createMemo, createSignal } from "solid-js";
import { runCommand } from "../../shared/api/commands";
import { activeProject } from "../../shared/project/active";
import { loadOutline } from "../explorer/explorer";
import { resetGraph } from "../graph/graph";
import { failed, type Failure } from "../explorer/failure";
import { projectsFailure, rescanProject } from "../projects/projects";
import { COMMAND } from "./consts";
import { isValid, sameLayers } from "./draft";
import type { LayerEntry, LayersData } from "./types";

const [data, setData] = createSignal<LayersData | null>(null);
const [draft, setDraft] = createSignal<LayerEntry[]>([]);
const [failure, setFailure] = createSignal<Failure | null>(null);
const [saving, setSaving] = createSignal(false);
const [saved, setSaved] = createSignal(false);
/** Counts the loads, so that an answer for a project that is no longer shown is dropped. */
let latest = 0;

/** What the server said about the project: layers, namespaces, whether they can be written. */
export const layersData = data;
/** The layers as the user has put them together so far. */
export const layersDraft = draft;
/** Replaces the draft. */
export const setLayersDraft = setDraft;
/** The failure of the last read or save; `null` if it worked. */
export const layersFailure = failure;
/** Whether the layers are being saved and the project is scanned again. */
export const layersSaving = saving;
/** Whether the last save went through. */
export const layersSaved = saved;

/** The project the editor works on: the one that is open, else the first. */
export const layersProject = createMemo(() => activeProject());

/** Whether the draft differs from what the project has. */
export function layersChanged(): boolean {
  return data() !== null && !sameLayers(draft(), data()!.layers);
}

/** Whether the draft can be saved now. */
export function layersSavable(): boolean {
  return layersChanged() && isValid(draft()) && data()?.editable === true && !saving();
}

/** Reads the layers and the namespaces of a project and starts the draft from the layers. */
export async function loadLayers(project: number): Promise<void> {
  const mine = ++latest;
  setData(null);
  setFailure(null);
  setSaved(false);
  try {
    const read = await runCommand<LayersData>(COMMAND.get, { project });
    if (mine === latest) {
      setData(read);
      setDraft(read.layers);
    }
  } catch (error) {
    if (mine === latest) {
      setFailure(failed(error));
    }
  }
}

/** Forgets what was read, when the screen is left or the project changes. */
export function resetLayers(): void {
  latest += 1;
  setData(null);
  setDraft([]);
  setFailure(null);
  setSaved(false);
}

/** Saves the draft into the project file, scans the project again so the layers apply, and
 * reads everything anew. */
export async function saveLayers(project: number): Promise<void> {
  setSaving(true);
  setFailure(null);
  setSaved(false);
  const layers = draft().map((layer) => ({ ...layer, name: layer.name.trim() }));
  try {
    await runCommand(COMMAND.set, { project, layers });
    await rescanProject(project);
    resetGraph();
    void loadOutline(project);
    await loadLayers(project);
    const scanFailure = projectsFailure();
    setFailure(scanFailure);
    setSaved(scanFailure === null);
  } catch (error) {
    setFailure(failed(error));
  } finally {
    setSaving(false);
  }
}
