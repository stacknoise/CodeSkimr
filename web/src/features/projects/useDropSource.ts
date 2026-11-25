import { createSignal, onCleanup, onMount } from "solid-js";
import { t } from "../../shared/i18n/i18n";
import { I18N_KEYS } from "../../shared/i18n/keys";
import { DRAG_EVENT, DROP } from "./consts";
import { readDrop } from "./drop";

/** Whether a drag that began in this page is under way (a line of the list, say): its text is no
 * path to add. `take` ends it, so a drop closes the drag even if `dragend` never comes. */
function useOwnDrag() {
  let inside = false;
  const started = () => (inside = true);
  const ended = () => (inside = false);
  onMount(() => {
    window.addEventListener(DRAG_EVENT.start, started);
    window.addEventListener(DRAG_EVENT.end, ended);
  });
  onCleanup(() => {
    window.removeEventListener(DRAG_EVENT.start, started);
    window.removeEventListener(DRAG_EVENT.end, ended);
  });
  return {
    own: () => inside,
    take: () => {
      const was = inside;
      inside = false;
      return was;
    },
  };
}

/** What a drop on the window does: a path or address goes to `onText`, a folder without a path
 * is explained. Returns whether something is being dragged over the window, and the notice. */
export function useDropSource(onText: (text: string) => void, onDone: () => void) {
  const [dragging, setDragging] = createSignal(false);
  const [notice, setNotice] = createSignal<string | null>(null);
  const drag = useOwnDrag();
  const over = (event: DragEvent) => {
    event.preventDefault();
    setDragging(!drag.own());
  };
  const drop = (event: DragEvent) => {
    event.preventDefault();
    setDragging(false);
    if (drag.take()) {
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
    window.addEventListener(DRAG_EVENT.over, over);
    window.addEventListener(DRAG_EVENT.drop, drop);
    window.addEventListener(DRAG_EVENT.leave, leave);
  });
  onCleanup(() => {
    window.removeEventListener(DRAG_EVENT.over, over);
    window.removeEventListener(DRAG_EVENT.drop, drop);
    window.removeEventListener(DRAG_EVENT.leave, leave);
  });
  return { dragging, notice, clearNotice: () => setNotice(null) };
}
