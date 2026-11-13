import { createMemo, createSignal } from "solid-js";
import { NOT_FOUND } from "./consts";
import type { ProjectRow } from "./types";

/** The selected row of the list and the moves of the arrow keys. */
export function useSelection(rows: () => ProjectRow[], initial: number | null) {
  const [selected, setSelected] = createSignal<number | null>(initial);
  const ids = createMemo(() => rows().map((row) => row.id));
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
    if (id !== null) {
      run(id);
    }
  };
  return { selected, setSelected, move, act };
}
