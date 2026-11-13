import { isTyping, useKeydown } from "../../shared/keys/keydown";
import { KEY, NOT_FOUND } from "./consts";

/** What the document-wide keys of the explorer do. */
export interface ExplorerKeys {
  collapseAll: () => void;
  reveal: () => void;
}

/** Registers `Ctrl+Left` (close the whole tree) and `Ctrl+.` (show the selection) while the
 * screen is shown. The plain arrows belong to the tree itself, so the page can still scroll. */
export function useExplorerKeys(keys: ExplorerKeys): void {
  useKeydown((event) => {
    if (!event.ctrlKey || event.altKey || event.metaKey || isTyping(event.target)) {
      return;
    }
    if (event.key === KEY.left) {
      keys.collapseAll();
    } else if (event.key === KEY.reveal) {
      keys.reveal();
    } else {
      return;
    }
    event.preventDefault();
  });
}

/** The index the cursor moves to for a navigation key, or `null` for any other key. With no
 * cursor yet (`NOT_FOUND`), down and end start at the first and the last row. */
export function cursorTarget(key: string, at: number, count: number): number | null {
  const last = count - 1;
  switch (key) {
    case KEY.down:
      return Math.min(last, at + 1);
    case KEY.up:
      return Math.max(0, at === NOT_FOUND ? 0 : at - 1);
    case KEY.home:
      return 0;
    case KEY.end:
      return last;
    default:
      return null;
  }
}
