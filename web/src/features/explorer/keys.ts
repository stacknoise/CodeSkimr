import { isTyping, useKeydown } from "../../shared/keys/keydown";
import { KEY, NOT_FOUND, STEP_DOWN, STEP_UP } from "./consts";

/** What the keys of the explorer do. */
export interface ExplorerKeys {
  move: (step: number) => void;
  open: () => void;
  close: () => void;
  collapseAll: () => void;
  reveal: () => void;
}

/** The index of `key` in the list, clamped to the list after a step; the first one if `key` is
 * not in it. */
export function stepIndex(keys: readonly string[], current: string | null, step: number): number {
  const at = current === null ? NOT_FOUND : keys.indexOf(current);
  if (at === NOT_FOUND) {
    return 0;
  }
  return Math.min(keys.length - 1, Math.max(0, at + step));
}

function plain(key: string, keys: ExplorerKeys): boolean {
  switch (key) {
    case KEY.down:
      keys.move(STEP_DOWN);
      return true;
    case KEY.up:
      keys.move(STEP_UP);
      return true;
    case KEY.right:
      keys.open();
      return true;
    case KEY.left:
      keys.close();
      return true;
    default:
      return false;
  }
}

function withCtrl(key: string, keys: ExplorerKeys): boolean {
  if (key === KEY.left) {
    keys.collapseAll();
    return true;
  }
  if (key === KEY.reveal) {
    keys.reveal();
    return true;
  }
  return false;
}

/** Registers the keys of the tree while the screen is shown: the arrows move and open, Ctrl+Left
 * closes everything, Ctrl+. shows the selection. Typing in a field is left alone. */
export function useExplorerKeys(keys: ExplorerKeys): void {
  const onKey = (event: KeyboardEvent) => {
    if (event.altKey || event.metaKey || isTyping(event.target)) {
      return;
    }
    const handled = event.ctrlKey ? withCtrl(event.key, keys) : plain(event.key, keys);
    if (handled) {
      event.preventDefault();
    }
  };
  useKeydown(onKey);
}
