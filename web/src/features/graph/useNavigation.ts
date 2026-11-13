import { createSignal, onCleanup, onMount } from "solid-js";
import { isTyping } from "../../shared/keys/keydown";
import { BUTTON_MIDDLE, KEY } from "./consts";
import type { View } from "./types";
import { notches, pan, zoomAt } from "./viewport";

/** Mouse navigation of the map: the wheel zooms around the pointer; dragging with the space key
 * held, or with the middle button, pans. Returns the handlers for the pointer and whether a drag
 * is possible or running (for the cursor). */
export function useNavigation(
  area: () => HTMLElement | undefined,
  view: () => View,
  onView: (view: View) => void,
) {
  const [spaceDown, setSpaceDown] = createSignal(false);
  const [dragging, setDragging] = createSignal(false);
  const onWheel = (event: WheelEvent) => {
    event.preventDefault();
    const rect = area()?.getBoundingClientRect();
    const at = { x: event.clientX - (rect?.left ?? 0), y: event.clientY - (rect?.top ?? 0) };
    onView(zoomAt(view(), notches(event.deltaY), at));
  };
  const space = (down: boolean) => (event: KeyboardEvent) => {
    if (event.key === KEY.pan && !isTyping(event.target)) {
      // Space on a focused box would press it when released: it pans instead.
      event.preventDefault();
      setSpaceDown(down);
    }
  };
  const keydown = space(true);
  const keyup = space(false);
  onMount(() => {
    area()?.addEventListener("wheel", onWheel, { passive: false });
    document.addEventListener("keydown", keydown);
    document.addEventListener("keyup", keyup);
  });
  onCleanup(() => {
    area()?.removeEventListener("wheel", onWheel);
    document.removeEventListener("keydown", keydown);
    document.removeEventListener("keyup", keyup);
  });
  return {
    grabbing: () => spaceDown() || dragging(),
    start: (event: PointerEvent) => {
      if (spaceDown() || event.button === BUTTON_MIDDLE) {
        event.preventDefault();
        setDragging(true);
        area()?.setPointerCapture(event.pointerId);
      }
    },
    move: (event: PointerEvent) => {
      if (dragging()) {
        onView(pan(view(), event.movementX, event.movementY));
      }
    },
    stop: () => setDragging(false),
  };
}
