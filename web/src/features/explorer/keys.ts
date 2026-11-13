import { isTyping, useKeydown } from "../../shared/keys/keydown";
import { KEY, NOT_FOUND } from "./consts";
import type { TreeRow } from "./tree";

/** What the document-wide keys of the explorer do. */
export interface ExplorerKeys {
  collapseAll: () => void;
  reveal: () => void;
  /** Shows the usages of the selected symbol. */
  usages: () => void;
}

/** Registers `Ctrl+Left` (close the whole tree), `Ctrl+.` (show the selection) and `U` (usages of
 * the selection) while the screen is shown. The plain arrows belong to the tree itself, so the page can still scroll. */
export function useExplorerKeys(keys: ExplorerKeys): void {
  useKeydown((event) => {
    if (
      event.key.toLowerCase() === KEY.usages &&
      !event.ctrlKey &&
      !event.altKey &&
      !event.metaKey &&
      !isTyping(event.target)
    ) {
      keys.usages();
      event.preventDefault();
      return;
    }
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

/** What the keys of the tree can do. */
export interface TreeKeyActions {
  onCursor: (key: string) => void;
  onToggle: (row: TreeRow) => void;
  onActivate: (row: TreeRow) => void;
}

/** The keys of the tree: the arrows move the cursor (the row that `aria-activedescendant` names),
 * `Right` and `Left` open and close, `Enter` and `Space` act like a click. */
export function onTreeKey(
  event: KeyboardEvent,
  rows: TreeRow[],
  cursor: number,
  actions: TreeKeyActions,
): void {
  const row = rows[cursor];
  const target = cursorTarget(event.key, cursor, rows.length);
  if (event.ctrlKey || event.altKey || event.metaKey) {
    return;
  }
  if (target !== null && rows[target]) {
    actions.onCursor(rows[target].key);
  } else if (row && event.key === KEY.right && row.expandable && !row.open) {
    actions.onToggle(row);
  } else if (row && event.key === KEY.left && row.open) {
    actions.onToggle(row);
  } else if (row && (event.key === KEY.enter || event.key === KEY.space)) {
    actions.onActivate(row);
  } else {
    return;
  }
  event.preventDefault();
}
