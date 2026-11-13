import { onCleanup, onMount } from "solid-js";
import { KEY, PRESSABLE_TAGS, STEP_DOWN, STEP_UP, TYPING_TAGS } from "./consts";

/** What the keys of the project list do. */
export interface ProjectKeys {
  newProject: () => void;
  rescan: () => void;
  open: () => void;
  move: (step: number) => void;
}

/** A button or a link gets `Enter` itself: it must not open the selected project instead. */
function pressable(target: EventTarget | null): boolean {
  return (
    target instanceof HTMLElement && (PRESSABLE_TAGS as readonly string[]).includes(target.tagName)
  );
}

function typing(target: EventTarget | null): boolean {
  return (
    target instanceof HTMLElement && (TYPING_TAGS as readonly string[]).includes(target.tagName)
  );
}

/** Registers `N`, `R`, `Enter` and the arrow keys of the project list while the screen is shown
 * (blueprint 15.4). Typing in a field and combinations with Ctrl, Alt or Meta are left alone. */
export function useProjectKeys(keys: ProjectKeys): void {
  const onKey = (event: KeyboardEvent) => {
    if (event.ctrlKey || event.altKey || event.metaKey || typing(event.target)) {
      return;
    }
    if (event.key === KEY.open && pressable(event.target)) {
      return;
    }
    const handled = dispatch(event.key, keys);
    if (handled) {
      event.preventDefault();
    }
  };
  onMount(() => document.addEventListener("keydown", onKey));
  onCleanup(() => document.removeEventListener("keydown", onKey));
}

function dispatch(key: string, keys: ProjectKeys): boolean {
  switch (key.toLowerCase()) {
    case KEY.newProject:
      keys.newProject();
      return true;
    case KEY.rescan:
      keys.rescan();
      return true;
    case KEY.open.toLowerCase():
      keys.open();
      return true;
    case KEY.down.toLowerCase():
      keys.move(STEP_DOWN);
      return true;
    case KEY.up.toLowerCase():
      keys.move(STEP_UP);
      return true;
    default:
      return false;
  }
}
