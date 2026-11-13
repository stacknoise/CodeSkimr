import { createSignal, onCleanup, onMount } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { DROP } from "./consts";
import { readDrop } from "./drop";

/** What a drop on the window does: a path or address goes to `onText`, a folder without a path
 * is explained. Returns whether something is being dragged over the window, and the notice. */
export function useDropSource(onText: (text: string) => void, onDone: () => void) {
  const [dragging, setDragging] = createSignal(false);
  const [notice, setNotice] = createSignal<string | null>(null);
  const over = (event: DragEvent) => {
    event.preventDefault();
    setDragging(true);
  };
  const drop = (event: DragEvent) => {
    event.preventDefault();
    setDragging(false);
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
  });
  onCleanup(() => {
    window.removeEventListener("dragover", over);
    window.removeEventListener("drop", drop);
    window.removeEventListener("dragleave", leave);
  });
  return { dragging, notice, clearNotice: () => setNotice(null) };
}
