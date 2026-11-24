import { createSignal } from "solid-js";
import { openProject } from "../../shared/project/active";
import { PROJECT_TAB, type ProjectTab } from "./consts";

const [selected, setSelected] = createSignal<number | null>(null);
const [tab, setTab] = createSignal<ProjectTab>(PROJECT_TAB.overview);

/** The project whose page is shown, `null` until one is chosen. */
export const selectedProject = selected;
/** The tab of the project page that is shown. */
export const projectTab = tab;

/** Shows the page of a project; the tab stays unless one is named. */
export function selectProject(id: number | null, next?: ProjectTab): void {
  setSelected(id);
  if (next !== undefined) {
    setTab(next);
  }
}

/** Changes the tab of the page that is shown. */
export function showTab(next: ProjectTab): void {
  setTab(next);
}

/** Opens the project and shows its tab "Scan": a scan of it started, or ends the wait for one. */
export function showScan(project: number): void {
  openProject(project);
  selectProject(project, PROJECT_TAB.scan);
}
