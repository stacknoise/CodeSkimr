import { createMemo } from "solid-js";
import { NOT_FOUND } from "./consts";
import { selectedProject, selectProject } from "./page";
import { removingNow } from "./removing";
import type { ProjectRow } from "./types";

/** The selected row of the list and the moves of the arrow keys. The selection is kept outside
 * the screen, so that it survives a visit of another screen. */
export function useSelection(rows: () => ProjectRow[], initial: number | null) {
  if (selectedProject() === null && initial !== null) {
    selectProject(initial);
  }
  const selected = selectedProject;
  const setSelected = (id: number | null) => selectProject(id);
  const ids = createMemo(() =>
    rows()
      .filter((row) => !removingNow(row))
      .map((row) => row.id),
  );
  const move = (step: number) => {
    const list = ids();
    const at = list.indexOf(selected() ?? NOT_FOUND);
    const next = list[Math.min(list.length - 1, Math.max(0, at + step))];
    if (next !== undefined) {
      setSelected(next);
    }
  };
  /** Runs `run` with the selected id, if there is one. */
  const act = (run: (id: number) => void) => () => {
    const id = selected();
    const row = rows().find((known) => known.id === id);
    if (id !== null && (row === undefined || !removingNow(row))) {
      run(id);
    }
  };
  return { selected, setSelected, move, act };
}
