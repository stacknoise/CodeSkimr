import { useKeydown } from "../../shared/keys/keydown";
import { CHIPS, KEY, NONE, STEP_DOWN, STEP_UP, type Filter } from "./consts";
import type { Entry } from "./entries";

/** Registers `Ctrl+K` (`Cmd+K` on a Mac) for the whole page, also while a field has the focus. */
export function useSearchShortcut(open: () => void): void {
  useKeydown((event) => {
    if ((event.ctrlKey || event.metaKey) && !event.altKey && event.key.toLowerCase() === KEY.open) {
      event.preventDefault();
      open();
    }
  });
}

/** The entry the choice moves to for an arrow key, or `NONE` for any other key. */
export function moveChoice(key: string, at: number, count: number): number {
  if (count === 0) {
    return NONE;
  }
  if (key === KEY.down) {
    return Math.min(count - 1, at + STEP_DOWN);
  }
  return key === KEY.up ? Math.max(0, at + STEP_UP) : NONE;
}

/** The chip after the current one, wrapping around; `STEP_UP` goes backwards. */
export function nextChip(current: Filter, step: number): Filter {
  const at = CHIPS.findIndex((chip) => chip.filter === current);
  const next = (at + step + CHIPS.length) % CHIPS.length;
  return CHIPS[next]?.filter ?? current;
}

/** What the keys of the overlay ask for. */
export interface SearchKeyActions {
  close: () => void;
  choose: (index: number) => void;
  activate: (entry: Entry) => void;
  chip: (step: number) => void;
  copy: (entry: Entry) => boolean;
  editor: (entry: Entry) => void;
}

/** The keys of the overlay: arrows choose, `Enter` opens, `Tab` changes the filter, `Ctrl+C`
 * copies the name when no text of the input is selected, `Esc` closes. */
export function onSearchKey(
  event: KeyboardEvent,
  entries: Entry[],
  at: number,
  actions: SearchKeyActions,
  hasSelection: boolean,
): void {
  const entry = entries[at];
  const moved = moveChoice(event.key, at, entries.length);
  if (event.key === KEY.close) {
    actions.close();
  } else if (moved !== NONE) {
    actions.choose(moved);
  } else if (event.key === KEY.enter && entry && event.altKey && !event.ctrlKey) {
    actions.editor(entry);
  } else if (event.key === KEY.enter && entry && !event.ctrlKey && !event.altKey) {
    actions.activate(entry);
  } else if (event.key === KEY.tab) {
    actions.chip(event.shiftKey ? STEP_UP : STEP_DOWN);
  } else if (
    (event.ctrlKey || event.metaKey) &&
    event.key.toLowerCase() === KEY.copy &&
    entry &&
    !hasSelection
  ) {
    if (!actions.copy(entry)) {
      return;
    }
  } else {
    return;
  }
  event.preventDefault();
}
