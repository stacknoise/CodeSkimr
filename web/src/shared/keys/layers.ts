import { createEffect, createSignal, onCleanup } from "solid-js";

/** How many layers (forms, windows) are open that take the key Esc before anything else does. */
const [openLayers, setOpenLayers] = createSignal(0);
const ONE = 1;

/** Whether a layer was open when the last key went down; read before any handler closes it. */
let openAtKey = false;
if (typeof window !== "undefined") {
  window.addEventListener("keydown", () => (openAtKey = openLayers() > 0), true);
}

/** Registers a layer while `open()` is true: Esc closes it and does not cancel a scan. */
export function useLayer(open: () => boolean): void {
  createEffect(() => {
    if (open()) {
      setOpenLayers((count) => count + ONE);
      onCleanup(() => setOpenLayers((count) => count - ONE));
    }
  });
}

/** Whether a layer was open when the key in the handler that runs now went down. */
export function layerOpenAtKey(): boolean {
  return openAtKey;
}
