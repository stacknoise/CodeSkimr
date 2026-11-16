import { createEffect, createMemo, createSignal, onCleanup, untrack } from "solid-js";
import { FALLBACK_VIEWPORT_ROWS, OVERSCAN_ROWS, ROW_HEIGHT_PX } from "./consts";

/** A row with its place in the whole list. */
export interface Placed<T> {
  item: T;
  index: number;
}

/** The window of a long list that is drawn (blueprint 15.5): only the rows in view and a few
 * more are in the document, the scroll bar still has the length of the whole list. */
export function useWindow<T>(items: () => T[], cursorIndex: () => number) {
  const [top, setTop] = createSignal(0);
  const [height, setHeight] = createSignal(0);
  let element: HTMLElement | undefined;
  const viewport = () => (height() > 0 ? height() : FALLBACK_VIEWPORT_ROWS * ROW_HEIGHT_PX);
  const visible = createMemo<Placed<T>[]>(() => {
    const first = Math.max(0, Math.floor(top() / ROW_HEIGHT_PX) - OVERSCAN_ROWS);
    const last = Math.ceil((top() + viewport()) / ROW_HEIGHT_PX) + OVERSCAN_ROWS;
    return items()
      .slice(first, last)
      .map((item, offset) => ({ item, index: first + offset }));
  });
  // Only a new cursor moves the view. The scroll position and the height are read untracked: if
  // they were dependencies, every scroll step would run this again and pull the view back to a
  // cursor row that has left it, so the list could not be scrolled freely.
  createEffect(() => {
    const index = cursorIndex();
    const at = index * ROW_HEIGHT_PX;
    if (element && index >= 0) {
      const [from, size] = untrack(() => [top(), viewport()]);
      if (at < from) {
        element.scrollTop = at;
      } else if (at + ROW_HEIGHT_PX > from + size) {
        element.scrollTop = at + ROW_HEIGHT_PX - size;
      }
    }
  });
  const attach = (target: HTMLElement) => {
    element = target;
    setHeight(target.clientHeight);
    if (typeof ResizeObserver !== "undefined") {
      const observer = new ResizeObserver(() => setHeight(target.clientHeight));
      observer.observe(target);
      onCleanup(() => observer.disconnect());
    }
  };
  return {
    visible,
    attach,
    onScroll: (event: Event) => setTop((event.currentTarget as HTMLElement).scrollTop),
    total: () => items().length * ROW_HEIGHT_PX,
  };
}
