import { createSignal, onCleanup, onMount } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { DROP } from "./consts";
import { readDrop } from "./drop";

/** What a drop on the window does: a path or address goes to `onText`, a folder without a path
 * is explained. Returns whether something is being dragged over the window, and the notice. */
export function useDropSource(onText: (text: string) => void, onDone: () => void) {
  const [dragging, setDragging] = createSignal(false);
  // Text that was dragged inside this page (a line of the list, say) is no path to add.
  let inside = false;
  const started = () => (inside = true);
  const ended = () => (inside = false);
  const [notice, setNotice] = createSignal<string | null>(null);
  const over = (event: DragEvent) => {
    event.preventDefault();
    setDragging(!inside);
  };
  const drop = (event: DragEvent) => {
    event.preventDefault();
    setDragging(false);
    if (inside) {
      return;
    }
    const dropped = event.dataTransfer ? readDrop(event.dataTransfer) : { kind: DROP.empty };
    setNotice(dropped.kind === DROP.opaque ? t(I18N_KEYS.PROJECTS_DROP_WITHOUT_PATH) : null);
    if (dropped.kind === DROP.text) {
      onText(dropped.text);
    }
    onDone();
  };
  const leave = () => setDragging(false);
  onMount(() => {
    window.addEventListener("dragover", over);
    window.addEventListener("drop", drop);
    window.addEventListener("dragleave", leave);
    window.addEventListener("dragstart", started);
    window.addEventListener("dragend", ended);
  });
  onCleanup(() => {
    window.removeEventListener("dragover", over);
    window.removeEventListener("drop", drop);
    window.removeEventListener("dragleave", leave);
    window.removeEventListener("dragstart", started);
    window.removeEventListener("dragend", ended);
  });
  return { dragging, notice, clearNotice: () => setNotice(null) };
}
