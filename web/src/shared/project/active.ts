import { createSignal } from "solid-js";

const [active, setActive] = createSignal<number | null>(null);

/** Id of the project the other screens show, `null` until the user opens one. */
export const activeProject = active;

/** Makes a project the one that the explorer, the graph and the search work on. */
export function openProject(id: number): void {
  setActive(id);
}
